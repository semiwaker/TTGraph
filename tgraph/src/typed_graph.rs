//! Typed graph
//! A container for a graph-like data structure, where nodes have distinct types.
//! An edge in this graph is a data field in the node.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use uuid::Uuid;

use crate::arena::*;

pub mod debug;
pub mod display;
pub mod library;
pub mod macro_traits;
pub use macro_traits::*;
pub use tgraph_macros::*;

/// The index of a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeIndex(pub usize);

impl NodeIndex {
  /// Make an empty index
  pub fn empty() -> NodeIndex { NodeIndex(0) }

  /// Check if the index is empty
  pub fn is_empty(&self) -> bool { self.0 == 0 }
}

impl ArenaIndex for NodeIndex {
  fn new(id: usize) -> Self { NodeIndex(id) }
}

/// A graph with typed nodes
/// The graph can only by modified by commiting a transaction, which avoids mutable borrow of the graph
///
/// # Example:
///
/// ```rust
/// let ctx = Context::new();
/// let mut graph = Graph::<T>::new(&ctx);
/// let mut trans = Transaction::new(&ctx);
/// // Does some operations on the transaction
/// graph.commit(trans);
/// ```
#[derive(Clone)]
pub struct Graph<NodeT: NodeEnum> {
  ctx_id: Uuid,
  nodes: Arena<NodeT, NodeIndex>,
  back_links: BTreeMap<NodeIndex, BTreeSet<(NodeIndex, NodeT::SourceEnum)>>,
}

impl<NodeT: NodeEnum> Graph<NodeT> {
  /// Create an empty graph
  pub fn new(context: &Context) -> Self {
    Graph {
      ctx_id: context.id,
      nodes: Arena::new(Arc::clone(&context.node_dist)),
      back_links: BTreeMap::new(),
    }
  }

  /// Get the reference of a node
  pub fn get_node(&self, idx: NodeIndex) -> Option<&NodeT> { self.nodes.get(idx) }

  /// Iterate all nodes in the graph following the order of NodeIndex
  pub fn iter_nodes(&self) -> Iter<'_, NodeIndex, NodeT> { self.nodes.iter() }

  /// Get the number of nodes
  pub fn len(&self) -> usize { self.nodes.len() }

  /// Check if the graph has no node
  pub fn is_empty(&self) -> bool { self.len() == 0 }

  /// Commit an transtion to modify the graph
  /// Operation order:
  /// + Redirect nodes
  /// + Insert new nodes
  /// + Modify nodes
  /// + Update nodes
  /// + Redirect all nodes
  /// + Remove nodes
  pub fn commit(&mut self, t: Transaction<NodeT>) {
    if t.committed {
      return;
    }
    assert!(t.alloc_nodes.is_empty(), "There are unfilled allocated nodes");

    let mut bd = BidirectionLinkContainer::default();

    self.redirect_node_vec(t.redirect_nodes, &mut bd);
    self.merge_nodes(t.inc_nodes, &mut bd);
    for (i, f) in t.mut_nodes {
      self.modify_node(i, f, &mut bd)
    }
    for (i, f) in t.update_nodes {
      self.update_node(i, f, &mut bd)
    }
    self.redirect_node_vec(t.redirect_all_nodes, &mut bd);
    for n in &t.dec_nodes {
      self.remove_node(*n, &mut bd);
    }

    self.apply_bidirectional_links(bd);
  }

  /// Switch the context and relabel the node ids. Useful when there are a lot of removed NodeIndex, and after context switching it will be more concise.
  /// Warning: please ensure there is no uncommitted transactions!
  pub fn switch_context(self, new_ctx: &Context) -> Self {
    let mut new_nodes = Arena::new(Arc::clone(&new_ctx.node_dist));
    let mut id_map = BTreeMap::new();

    for (id, x) in self.nodes {
      id_map.insert(id, new_nodes.insert(x));
    }

    for (id, new_id) in &id_map {
      for (y, s) in &self.back_links[id] {
        new_nodes.get_mut(id_map[&y]).unwrap().modify_link(*s, *id, *new_id);
      }
    }

    let mut result = Graph {
      ctx_id: new_ctx.id,
      nodes: Arena::new(Arc::clone(&new_ctx.node_dist)),
      back_links: BTreeMap::new(),
    };

    let mut bd = BidirectionLinkContainer::default();
    result.merge_nodes(new_nodes, &mut bd);
    result.apply_bidirectional_links(bd);
    result
  }

  /// Check if the backlinks are connected correctly, just for debug
  #[doc(hidden)]
  pub fn check_backlinks(&self) {
    let mut back_links: BTreeMap<NodeIndex, BTreeSet<(NodeIndex, NodeT::SourceEnum)>> =
      BTreeMap::new();
    for (x, n) in &self.nodes {
      back_links.entry(*x).or_default();
      for (y, s) in n.iter_source() {
        back_links.entry(y).or_default().insert((*x, s));
        let links = self
          .back_links
          .get(&y)
          .unwrap_or_else(|| panic!("Node {} have no backlink!", x.0));
        assert!(links.contains(&(*x, s)));
      }
    }
    assert_eq!(back_links, self.back_links);
  }

  fn merge_nodes(
    &mut self, nodes: Arena<NodeT, NodeIndex>, bd: &mut BidirectionLinkContainer<NodeT>,
  ) {
    for (x, n) in &nodes {
      self.add_back_links(*x, n);
    }
    for (id, node) in &nodes {
      for (ys, lms) in node.get_bidiretional_links() {
        bd.add(*id, ys, &lms);
      }
    }
    self.nodes.merge(nodes);
  }

  fn remove_node(&mut self, x: NodeIndex, bd: &mut BidirectionLinkContainer<NodeT>) {
    self.remove_bidirectional_link(x, bd);
    let n = self.nodes.remove(x).expect("Remove a non-existing node!");
    self.remove_back_links(x, &n);
    for (y, s) in self.back_links.remove(&x).unwrap() {
      self.nodes.get_mut(y).unwrap().modify_link(s, x, NodeIndex::empty());
    }
  }

  fn modify_node<F>(
    &mut self, x: NodeIndex, f: F, bd: &mut BidirectionLinkContainer<NodeT>,
  ) where F: FnOnce(&mut NodeT) {
    self.remove_bidirectional_link(x, bd);
    for (y, s) in self.nodes.get(x).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(x, s));
    }

    f(self.nodes.get_mut(x).unwrap());

    self.add_bidirectional_link(x, bd);
    for (y, s) in self.nodes.get(x).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().insert((x, s));
    }
  }

  fn update_node<F>(
    &mut self, x: NodeIndex, f: F, bd: &mut BidirectionLinkContainer<NodeT>,
  ) where F: FnOnce(NodeT) -> NodeT {
    self.remove_bidirectional_link(x, bd);
    for (y, s) in self.nodes.get(x).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(x, s));
    }

    self.nodes.update_with(x, f);

    self.add_bidirectional_link(x, bd);
    for (y, s) in self.nodes.get(x).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().insert((x, s));
    }
  }

  fn add_bidirectional_link(
    &mut self, x: NodeIndex, bd: &mut BidirectionLinkContainer<NodeT>,
  ) {
    let to_add = self.nodes.get(x).unwrap().get_bidiretional_links();
    for (ys, lms) in to_add {
      bd.add(x, ys, &lms);
    }
  }

  fn remove_bidirectional_link(
    &mut self, x: NodeIndex, bd: &mut BidirectionLinkContainer<NodeT>,
  ) {
    let to_remove = self.nodes.get(x).unwrap().get_bidiretional_links();
    for (ys, lms) in to_remove {
      bd.remove(x, ys, &lms);
    }
  }

  fn redirect_node(
    &mut self, old_node: NodeIndex, new_node: NodeIndex,
    bd: &mut BidirectionLinkContainer<NodeT>,
  ) {
    let old_link = self.back_links.remove(&old_node).unwrap();
    self.back_links.insert(old_node, BTreeSet::new());

    let new_link = self.back_links.entry(new_node).or_default();
    for (y, s) in old_link {
      new_link.insert((y, s));
      let side_effect = self.nodes.get_mut(y).unwrap().modify_link(s, old_node, new_node);
      if !side_effect.add.is_empty() {
        bd.add_one(y, side_effect.add, &side_effect.link_mirrors);
      }
      if !side_effect.remove.is_empty() {
        bd.remove_one(y, side_effect.remove, &side_effect.link_mirrors);
      }
    }
  }

  fn redirect_node_vec(
    &mut self, replacements: Vec<(NodeIndex, NodeIndex)>,
    bd: &mut BidirectionLinkContainer<NodeT>,
  ) {
    let mut fa = BTreeMap::new();

    for (old, new) in &replacements {
      fa.entry(*old).or_insert(*old);
      fa.entry(*new).or_insert(*new);
    }

    for (old, new) in &replacements {
      let mut x = *new;
      while fa[&x] != x {
        x = fa[&x];
      }
      assert!(x != *old, "Loop redirection detected!");
      *fa.get_mut(old).unwrap() = x;
    }

    for (old, new) in &replacements {
      let mut x = *new;
      let mut y = fa[&x];
      while x != y {
        x = y;
        y = fa[&y];
      }

      self.redirect_node(*old, x, bd);

      x = *new;
      while fa[&x] != y {
        let z = fa[&x];
        *fa.get_mut(&x).unwrap() = y;
        x = z;
      }
    }
  }

  fn apply_bidirectional_links(&mut self, bd: BidirectionLinkContainer<NodeT>) {
    for (x, y, l) in bd.to_remove {
      if self.nodes.contains(x) && self.nodes.contains(y) {
        if self.nodes.get_mut(y).unwrap().remove_link(l, x) {
          self.remove_back_link(y, x, NodeT::to_source_enum(&l));
        }
      }
    }
    for (x, y, l) in bd.to_add {
      if self.nodes.contains(x) && self.nodes.contains(y) {
        if self.nodes.get_mut(y).unwrap().add_link(l, x) {
          self.add_back_link(y, x, NodeT::to_source_enum(&l));
        }
      }
    }
  }

  fn add_back_link(&mut self, x: NodeIndex, y: NodeIndex, src: NodeT::SourceEnum) {
    self.back_links.entry(y).or_default().insert((x, src));
  }

  fn add_back_links(&mut self, x: NodeIndex, n: &NodeT) {
    self.back_links.entry(x).or_default();
    for (y, s) in n.iter_source() {
      self.back_links.entry(y).or_default().insert((x, s));
    }
  }

  fn remove_back_link(&mut self, x: NodeIndex, y: NodeIndex, src: NodeT::SourceEnum) {
    self.back_links.get_mut(&y).unwrap().remove(&(x, src));
  }

  fn remove_back_links(&mut self, x: NodeIndex, n: &NodeT) {
    for (y, s) in n.iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(x, s));
    }
  }
}

// Helper struct for bidirectional links
struct BidirectionLinkContainer<NodeT: NodeEnum> {
  to_add: BTreeSet<(NodeIndex, NodeIndex, NodeT::LinkMirrorEnum)>,
  to_remove: BTreeSet<(NodeIndex, NodeIndex, NodeT::LinkMirrorEnum)>,
}
impl<NodeT: NodeEnum> BidirectionLinkContainer<NodeT> {
  fn add_one(&mut self, x: NodeIndex, y: NodeIndex, lms: &Vec<NodeT::LinkMirrorEnum>) {
    for l in lms {
      if self.to_remove.contains(&(x, y, *l)) {
        self.to_remove.remove(&(x, y, *l));
      } else {
        self.to_add.insert((x, y, *l));
      }
    }
  }

  fn add(&mut self, x: NodeIndex, ys: Vec<NodeIndex>, lms: &Vec<NodeT::LinkMirrorEnum>) {
    for y in ys {
      self.add_one(x, y, lms)
    }
  }

  fn remove_one(&mut self, x: NodeIndex, y: NodeIndex, lms: &Vec<NodeT::LinkMirrorEnum>) {
    for l in lms {
      if self.to_add.contains(&(x, y, *l)) {
        self.to_add.remove(&(x, y, *l));
      } else {
        self.to_remove.insert((x, y, *l));
      }
    }
  }

  fn remove(
    &mut self, x: NodeIndex, ys: Vec<NodeIndex>, lms: &Vec<NodeT::LinkMirrorEnum>,
  ) {
    for y in ys {
      self.remove_one(x, y, lms);
    }
  }
}
impl<NodeT: NodeEnum> Default for BidirectionLinkContainer<NodeT> {
  fn default() -> Self {
    BidirectionLinkContainer {
      to_add: BTreeSet::new(),
      to_remove: BTreeSet::new(),
    }
  }
}

impl<T: NodeEnum> IntoIterator for Graph<T> {
  type IntoIter = IntoIter<NodeIndex, T>;
  type Item = (NodeIndex, T);

  fn into_iter(self) -> Self::IntoIter { self.nodes.into_iter() }
}

pub type MutFunc<'a, T> = Box<dyn FnOnce(&mut T) + 'a>;
pub type UpdateFunc<'a, T> = Box<dyn FnOnce(T) -> T + 'a>;

/// The transaction to modify a graph
pub struct Transaction<'a, NodeT: NodeEnum> {
  committed: bool,
  ctx_id: Uuid,
  alloc_nodes: BTreeSet<NodeIndex>,
  inc_nodes: Arena<NodeT, NodeIndex>,
  dec_nodes: Vec<NodeIndex>,
  mut_nodes: Vec<(NodeIndex, MutFunc<'a, NodeT>)>,
  update_nodes: Vec<(NodeIndex, UpdateFunc<'a, NodeT>)>,
  redirect_all_nodes: Vec<(NodeIndex, NodeIndex)>,
  redirect_nodes: Vec<(NodeIndex, NodeIndex)>,
}

impl<'a, NodeT: NodeEnum> Transaction<'a, NodeT> {
  /// Make a empty transaction
  /// Please ensure the Graph and the Transaction use the same Context!
  pub fn new(context: &Context) -> Self {
    let node_dist = Arc::clone(&context.node_dist);
    Transaction {
      committed: false,
      ctx_id: context.id,
      alloc_nodes: BTreeSet::new(),
      inc_nodes: Arena::new(node_dist),
      dec_nodes: Vec::new(),
      mut_nodes: Vec::new(),
      update_nodes: Vec::new(),
      redirect_all_nodes: Vec::new(),
      redirect_nodes: Vec::new(),
    }
  }

  /// Allocate a new NodeIndex for a new node
  /// Useful when there is a cycle
  pub fn alloc_node(&mut self) -> NodeIndex {
    let idx = self.inc_nodes.alloc();
    self.alloc_nodes.insert(idx);
    idx
  }

  /// Fill back the data to a pre-allocated NodeIndex
  pub fn fill_back_node(&mut self, idx: NodeIndex, data: NodeT) {
    self.inc_nodes.fill_back(idx, data);
    self.alloc_nodes.remove(&idx);
  }

  /// Put a new node into the graph
  pub fn new_node(&mut self, data: NodeT) -> NodeIndex { self.inc_nodes.insert(data) }

  /// Remove an existing node
  pub fn remove_node(&mut self, node: NodeIndex) {
    if self.inc_nodes.remove(node).is_none() && !self.alloc_nodes.remove(&node) {
      self.dec_nodes.push(node);
    }
  }

  /// Mutate a node as &mut
  /// # Example
  /// ```rust
  /// trans.mut_node(idx, |x| x.y=z);
  /// ```
  pub fn mut_node<F>(&mut self, node: NodeIndex, func: F)
  where F: FnOnce(&mut NodeT) + 'a {
    if self.inc_nodes.contains(node) {
      func(self.inc_nodes.get_mut(node).unwrap());
    } else {
      self.mut_nodes.push((node, Box::new(func)));
    }
  }

  /// Update a node by consuming the old one
  /// # Example
  /// ```rust
  /// trans.mut_node(idx, |x| NodeType { a = x.a });
  /// ```
  pub fn update_node<F>(&mut self, node: NodeIndex, func: F)
  where F: FnOnce(NodeT) -> NodeT + 'a {
    if self.inc_nodes.contains(node) {
      self.inc_nodes.update_with(node, func);
    } else {
      self.update_nodes.push((node, Box::new(func)));
    }
  }

  /// Redirect the connections from old_node to new_node
  /// Nodes in the Graph and new nodes in the Transaction are both redirected
  pub fn redirect_all_node(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_all_nodes.push((old_node, new_node));
  }

  /// Redirect the connections from old_node to new_node
  /// Only nodes in the Graph is redirected, new nodes in the transaction is not redirected
  pub fn redirect_node(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_nodes.push((old_node, new_node));
  }

  /// Merge a graph and all its nodes
  /// The merged graph and this transaction should have then same context
  pub fn merge_graph(&mut self, graph: Graph<NodeT>) {
    assert!(self.ctx_id == graph.ctx_id);
    for (i, n) in graph.into_iter() {
      self.fill_back_node(i, n);
    }
  }

  /// Give up the transaction, have no effect when merging this transaction.
  pub fn giveup(&mut self) { self.committed = true; }
}

/// Context for typed graph
/// Transactions and graph must have the same context to ensure the correctness of NodeIndex
#[derive(Debug)]
pub struct Context {
  id: Uuid,
  node_dist: Arc<IdDistributer>,
}
impl Context {
  pub fn new() -> Context {
    Context {
      id: Uuid::new_v4(),
      node_dist: Arc::new(IdDistributer::new()),
    }
  }
}
impl Default for Context {
  fn default() -> Self { Self::new() }
}
impl Clone for Context {
  fn clone(&self) -> Self {
    Context {
      id: self.id,
      node_dist: Arc::clone(&self.node_dist),
    }
  }
}

pub trait SourceIterator<T: TypedNode>:
  Iterator<Item = (NodeIndex, Self::Source)>
{
  type Source: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  fn new(node: &T) -> Self;
}
