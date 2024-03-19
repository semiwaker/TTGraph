//! Typed graph
//! A container for a graph-like data structure, where nodes have distinct types.
//! An edge in this graph is a data field in the node.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use uuid::Uuid;

use crate::arena::*;

pub mod debug;
pub mod display;
pub mod library;

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

    self.redirect_node_vec(t.redirect_nodes);
    self.merge_nodes(t.inc_nodes);
    for (i, f) in t.mut_nodes {
      self.modify_node(i, f)
    }
    for (i, f) in t.update_nodes {
      self.update_node(i, f)
    }
    self.redirect_node_vec(t.redirect_all_nodes);
    for n in &t.dec_nodes {
      self.remove_node(*n);
    }
  }

  fn merge_nodes(&mut self, nodes: Arena<NodeT, NodeIndex>) {
    for (x, n) in &nodes {
      self.add_back_link(*x, n);
    }
    self.nodes.merge(nodes);
  }

  fn remove_node(&mut self, idx: NodeIndex) {
    let n = self.nodes.remove(idx).unwrap();
    self.remove_back_link(idx, &n);
    for (y, s) in self.back_links.remove(&idx).unwrap() {
      self.nodes.get_mut(y).unwrap().modify(s, idx, NodeIndex::empty());
    }
  }

  fn modify_node<F>(&mut self, i: NodeIndex, f: F)
  where F: FnOnce(&mut NodeT) {
    for (y, s) in self.nodes.get(i).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(i, s));
    }

    f(self.nodes.get_mut(i).unwrap());

    for (y, s) in self.nodes.get(i).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().insert((i, s));
    }
  }

  fn update_node<F>(&mut self, i: NodeIndex, f: F)
  where F: FnOnce(NodeT) -> NodeT {
    for (y, s) in self.nodes.get(i).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(i, s));
    }

    self.nodes.update_with(i, f);

    for (y, s) in self.nodes.get(i).unwrap().iter_source() {
      self.back_links.get_mut(&y).unwrap().insert((i, s));
    }
  }

  fn redirect_node(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    let old_link = self.back_links.remove(&old_node).unwrap();
    self.back_links.insert(old_node, BTreeSet::new());

    let new_link = self.back_links.entry(new_node).or_default();
    for (y, s) in old_link {
      new_link.insert((y, s));
      self.nodes.get_mut(y).unwrap().modify(s, old_node, new_node);
    }
  }

  fn redirect_node_vec(&mut self, replacements: Vec<(NodeIndex, NodeIndex)>) {
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

      self.redirect_node(*old, x);

      x = *new;
      while fa[&x] != y {
        let z = fa[&x];
        *fa.get_mut(&x).unwrap() = y;
        x = z;
      }
    }
  }

  fn add_back_link(&mut self, x: NodeIndex, n: &NodeT) {
    self.back_links.entry(x).or_default();
    for (y, s) in n.iter_source() {
      self.back_links.entry(y).or_default().insert((x, s));
    }
  }

  fn remove_back_link(&mut self, x: NodeIndex, n: &NodeT) {
    for (y, s) in n.iter_source() {
      self.back_links.get_mut(&y).unwrap().remove(&(x, s));
    }
  }
}

pub trait ContextSwitch {
  /// Switch the context and relabel the node ids. Useful when there are a lot of removed NodeIndex, and after context switching it will be more concise.
  /// Warning: please ensure there is no uncommitted transactions!!!
  fn switch_context(&self, new_context: &Context) -> Self;
}

impl<T: NodeEnum + Clone> ContextSwitch for Graph<T> {
  fn switch_context(&self, new_context: &Context) -> Self {
    let mut new_nodes = Arena::new(Arc::clone(&new_context.node_dist));
    let mut id_map = BTreeMap::new();
    let mut node_map = BTreeMap::new();
    let mut new_backlink = BTreeMap::new();

    for (id, x) in &self.nodes {
      id_map.insert(*id, new_nodes.alloc());
      node_map.insert(*id, x.clone());
    }

    for (id, _) in &self.nodes {
      let new_id = id_map[id];
      let mut backlink = BTreeSet::new();
      for (y, s) in &self.back_links[id] {
        node_map.get_mut(y).unwrap().modify(*s, *id, new_id);
        backlink.insert((id_map[y], *s));
      }
      new_backlink.insert(new_id, backlink);
    }

    for (id, x) in node_map {
      new_nodes.fill_back(id, x);
    }

    Graph {
      ctx_id: new_context.id,
      nodes: new_nodes,
      back_links: new_backlink,
    }
  }
}

impl<T: NodeEnum> IntoIterator for Graph<T> {
  type IntoIter = IntoIter<NodeIndex, T>;
  type Item = (NodeIndex, T);

  fn into_iter(self) -> Self::IntoIter { self.nodes.into_iter() }
}

type MutFunc<'a, T> = Box<dyn FnOnce(&mut T) + 'a>;
type UpdateFunc<'a, T> = Box<dyn FnOnce(T) -> T + 'a>;

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
  /// The merged graph can have a different context than the one this transaction used
  pub fn merge_graph(&mut self, graph: Graph<NodeT>) {
    let graph_ctx_id = graph.ctx_id;
    for (i, n) in graph.into_iter() {
      if self.ctx_id != graph_ctx_id {
        self.new_node(n);
      } else {
        self.fill_back_node(i, n);
      }
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

/// A helper trait for the graph to trace all links in the nodes
/// # Example
/// ```rust
/// use tgraph::typed_graph::*;
/// use tgraph_macros::*;
/// #[derive(TypedNode)]
/// struct SomeNode {
///   a_link: NodeIndex,
///   another_link: NodeIndex,
///   vec_link: Vec<NodeIndex>,
///   set_link: HashSet<NodeIndex>,
///   bset_link: BTreeSet<NodeIndex>,
///   other_data: usize
///   // ...
/// }
/// ```
pub trait TypedNode: Sized {
  type Source: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  type Iter: SourceIterator<Self, Source = Self::Source>;
  fn iter_source(&self) -> Self::Iter;
  fn modify(&mut self, source: Self::Source, old_idx: NodeIndex, new_idx: NodeIndex);
}

/// A helper trait to declare a enum of all typed nodes
/// # Example
/// ```rust
/// use tgraph::typed_graph::*;
/// use tgraph_macros::*;
/// #[derive(TypedNode)]
/// struct A{
///   a: NodeIndex,
/// }
/// 
/// #[derive(TypedNode)]
/// struct B{
///   b: NodeIndex,
/// }
/// 
/// #[derive(NodeEnum)]
/// struct Node{
///   NodeTypeA(A),
///   AnotherNodeType(B),
/// }
/// ```
pub trait NodeEnum {
  type SourceEnum: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  fn iter_source(&self) -> Box<dyn Iterator<Item = (NodeIndex, Self::SourceEnum)>>;
  fn modify(&mut self, source: Self::SourceEnum, old_idx: NodeIndex, new_idx: NodeIndex);
}

pub trait IndexEnum {
  fn modify(&mut self, new_idx: NodeIndex);
  fn index(&self) -> NodeIndex;
}

pub struct NIEWrap<T: IndexEnum> {
  pub value: T,
}
