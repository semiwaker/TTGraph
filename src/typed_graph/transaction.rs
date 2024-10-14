use ordermap::OrderSet;

use visible::StructFields;

use super::*;
/// The transaction to modify a [`Graph`].
///
/// It is a operation recorder which have independent lifetime than the graph and does not hold reference to the graph.
///
/// The transaction and the graph should have been created from the same [`Context`] to ensure correctness.
#[StructFields(pub(crate))]
pub struct Transaction<'a, NodeT, Arena = <NodeT as NodeEnum>::GenArena>
where
  NodeT: NodeEnum,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>,
{
  ctx_id: Uuid,
  alloc_nodes: OrderSet<NodeIndex>,
  inc_nodes: Arena,
  dec_nodes: OrderSet<NodeIndex>,
  mut_nodes: Vec<(NodeIndex, MutFunc<'a, NodeT>)>,
  update_nodes: Vec<(NodeIndex, UpdateFunc<'a, NodeT>)>,
  redirect_all_links_vec: Vec<(NodeIndex, NodeIndex)>,
  redirect_links_vec: Vec<(NodeIndex, NodeIndex)>,
}

impl<'a, NodeT, Arena> Transaction<'a, NodeT, Arena>
where
  NodeT: NodeEnum,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>,
{
  /// Make a empty transaction
  ///
  /// Please ensure the [`Graph`] and the [`Transaction`] use the same Context!
  ///
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// // In fact Transaction::<Node>::new(), but <Node> can be inferenced when commit
  /// let mut trans = Transaction::new(&ctx);
  /// trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// # }
  /// ```
  pub fn new(context: &Context) -> Self {
    let node_dist = context.node_dist.clone();
    Transaction {
      ctx_id: context.id,
      alloc_nodes: OrderSet::new(),
      inc_nodes: Arena::new(node_dist),
      dec_nodes: OrderSet::new(),
      mut_nodes: Vec::new(),
      update_nodes: Vec::new(),
      redirect_all_links_vec: Vec::new(),
      redirect_links_vec: Vec::new(),
    }
  }

  /// Allocate a new [`NodeIndex`] for a new node, use together with [`fill_back`](Transaction::fill_back)
  /// 
  /// A discriminant is required to denote the type of the node, which can be acquired by the [`discriminant!`](crate::discriminant!) macro. Use [`alloc_node!`](crate::alloc_node) is simplier.
  ///
  /// Useful when there is a cycle.
  ///
  /// # Panic
  /// If there is a node that is allocated, but is not filled back, it will be detected when commit.
  /// 
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   last: NodeIndex,
  ///   next: NodeIndex,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// // Alloc some nodes
  /// let n1 = trans.alloc(discriminant!(Node::A));
  /// let n2 = alloc_node!(trans, Node::A);
  /// let n3 = alloc_node!(trans, Node::A);
  /// // Build a circle
  /// trans.fill_back(n1, Node::A(NodeA{ last: n3, next: n2 }));
  /// trans.fill_back(n2, Node::A(NodeA{ last: n1, next: n3 }));
  /// trans.fill_back(n3, Node::A(NodeA{ last: n2, next: n1 }));
  /// graph.commit(trans);
  /// # }
  /// ```
  pub fn alloc(&mut self, d: NodeT::Discriminant) -> NodeIndex {
    let idx = self.inc_nodes.alloc(d);
    self.alloc_nodes.insert(idx);
    idx
  }

  /// Similar to [`alloc`](Transaction::alloc) but does not require a type. Use [`fill_back_untyped`](Transaction::fill_back_untyped) to fill back.
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   last: NodeIndex,
  ///   next: NodeIndex,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// // Alloc some nodes
  /// let n1 = trans.alloc_untyped();
  /// let n2 = trans.alloc_untyped();
  /// let n3 = trans.alloc_untyped();
  /// // Build a circle
  /// trans.fill_back_untyped(n1, Node::A(NodeA{ last: n3, next: n2 }));
  /// trans.fill_back_untyped(n2, Node::A(NodeA{ last: n1, next: n3 }));
  /// trans.fill_back_untyped(n3, Node::A(NodeA{ last: n2, next: n1 }));
  /// graph.commit(trans);
  /// # }
  /// ```
  pub fn alloc_untyped(&mut self) -> NodeIndex {
    let idx = self.inc_nodes.alloc_untyped();
    self.alloc_nodes.insert(idx);
    idx
  }

  /// Fill back the data to a [`NodeIndex`] created by [`alloc`](Transaction::alloc)
  pub fn fill_back(&mut self, idx: NodeIndex, data: NodeT) {
    self.inc_nodes.fill_back(idx, data);
    self.alloc_nodes.remove(&idx);
  }

  /// Fill back the data to a [`NodeIndex`] created by [`alloc_untyped`](Transaction::alloc_untyped)
  pub fn fill_back_untyped(&mut self, idx: NodeIndex, data: NodeT) {
    self.inc_nodes.fill_back_untyped(idx, data);
    self.alloc_nodes.remove(&idx);
  }

  /// Insert a new node into the graph, returns a [`NodeIndex`] pointing to the new node
  ///
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// // Get the node back by the returned NodeIndex idx
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  /// # }
  /// ```
  pub fn insert(&mut self, data: NodeT) -> NodeIndex {
    self.inc_nodes.insert(data)
  }

  /// Remove an existing node
  ///
  /// Note: nodes created by [`insert`](Transaction::insert) and [`alloc`](Transaction::alloc) in this uncommitted transaction can also be removed.
  ///
  /// Currently backed by [OrderMap::swap_remove] for performance, the order is changed after remove.
  ///
  /// Example:
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert!(graph.get(idx).is_some());
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.remove(idx);
  /// graph.commit(trans);
  /// // Now the node is removed
  /// assert!(graph.get(idx).is_none());
  /// # }
  /// ```
  pub fn remove(&mut self, node: NodeIndex) {
    if self.inc_nodes.remove(node).is_none() && !self.alloc_nodes.remove(&node) {
      self.dec_nodes.insert(node);
    }
  }

  /// Mutate a node as with a closure `FnOnce(&mut NodeT)`.
  ///
  /// If the type of the node is previously known, use [`mut_node!`](crate::mut_node!) instead.
  ///
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA),
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.mutate(idx, |node|{
  ///   if let Node::A(x) = node{
  ///     x.data = 2
  ///   }
  /// });
  /// graph.commit(trans);
  /// // Now the data field of the node is 2
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 2);
  /// }
  /// ```
  /// # Performance warning
  ///
  /// TTGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  ///
  /// Try not to create a node with a very large connectivity. Merge multiple operations into one.
  pub fn mutate<F>(&mut self, node: NodeIndex, func: F)
  where
    F: FnOnce(&mut NodeT) + 'a,
  {
    if self.inc_nodes.contains(node) {
      func(self.inc_nodes.get_mut(node).unwrap());
    } else {
      self.mut_nodes.push((node, Box::new(func)));
    }  }

  /// Update a node with a closure `FnOnce(NodeT) -> NodeT`.
  ///
  /// If the type of the node is previously known, use [`update_node!`](crate::update_node!) instead.
  ///
  /// The node is taken out of the container and will be put back after updating, so the order is changed.
  ///
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA),
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.update(idx, |node| {
  ///   if let Node::A(x) = node{
  ///     Node::A(NodeA{ data: x.data + 1 })
  ///   } else {
  ///     panic!()
  ///   }
  /// });
  /// graph.commit(trans);
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 2);
  /// # }
  /// ```
  /// # Performance warning
  ///
  /// TTGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  ///
  /// Try not to create a node with a very large connectivity. Merge multiple operations into one.
  pub fn update<F>(&mut self, node: NodeIndex, func: F)
  where
    F: FnOnce(NodeT) -> NodeT + 'a,
  {
    if self.inc_nodes.contains(node) {
      self.inc_nodes.update_with(node, func);
    } else {
      self.update_nodes.push((node, Box::new(func)));
    }
  }

  /// Redirect the connections from old_node to new_node
  ///
  /// Nodes in the [`Graph`] and new nodes in the [`Transaction`] are both redirected
  ///
  /// See [`redirect_links`](Transaction::redirect_links) for counter-example
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// use std::collections::BTreeSet;
  /// #[derive(TypedNode)]
  /// struct NodeA {
  ///   tos: BTreeSet<NodeIndex>,
  /// }
  ///
  /// node_enum! {
  ///   enum Node {
  ///     A(NodeA),
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let context = Context::new();
  /// let mut graph = Graph::<Node>::new(&context);
  /// let mut trans = Transaction::new(&context);
  /// let a = alloc_node!(trans, Node::A);
  /// let b = alloc_node!(trans, Node::A);
  /// let c = alloc_node!(trans, Node::A);
  /// let d = trans.insert(Node::A(NodeA { tos: BTreeSet::new() }));
  /// trans.fill_back(c, Node::A(NodeA { tos: BTreeSet::from_iter([d]) }));
  /// trans.fill_back(b, Node::A(NodeA { tos: BTreeSet::from_iter([c, d]) }));
  /// trans.fill_back(a, Node::A(NodeA { tos: BTreeSet::from_iter([b, c, d]) }));
  /// // Though these new nodes are not commited, they can still be redirected
  /// trans.redirect_all_links(c, b);
  /// trans.redirect_all_links(b, a);
  /// trans.redirect_all_links(d, c);
  /// graph.commit(trans);
  ///
  /// // Graph after redirect
  /// // a -> {a, a, a} = {a}
  /// // b -> {a, a}    = {a}
  /// // c -> {a}       = {a}
  /// // d -> {}
  /// assert_eq!(get_node!(graph, Node::A, a).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, b).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, c).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, d).unwrap().tos, BTreeSet::new());
  /// # }
  /// ```
  pub fn redirect_all_links(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_all_links_vec.push((old_node, new_node));
  }

  /// Redirect the connections from old_node to new_node
  ///
  /// Only nodes in the [`Graph`] is redirected, new nodes in the [`Transaction`] is not redirected
  ///
  /// See [`redirect_all_links`](Transaction::redirect_all_links) for counter-example
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// use std::collections::BTreeSet;
  /// #[derive(TypedNode)]
  /// struct NodeA {
  ///   tos: BTreeSet<NodeIndex>,
  /// }
  ///
  /// node_enum! {
  ///   enum Node {
  ///     A(NodeA),
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let context = Context::new();
  /// let mut graph = Graph::<Node>::new(&context);
  /// let mut trans = Transaction::new(&context);
  /// let a = alloc_node!(trans, Node::A);
  /// let b = alloc_node!(trans, Node::A);
  /// let c = alloc_node!(trans, Node::A);
  /// let d = trans.insert(Node::A(NodeA { tos: BTreeSet::new() }));
  /// trans.fill_back(c, Node::A(NodeA { tos: BTreeSet::from_iter([d]) }));
  /// trans.fill_back(b, Node::A(NodeA { tos: BTreeSet::from_iter([c, d]) }));
  /// trans.fill_back(a, Node::A(NodeA { tos: BTreeSet::from_iter([b, c, d]) }));
  /// graph.commit(trans);
  ///
  /// // Graph before redirect
  /// // a -> {b, c, d}
  /// // b -> {c, d}
  /// // c -> {d}
  /// // d -> {}
  ///
  /// trans = Transaction::new(&context);
  /// // Redirect d -> c -> b -> a
  /// // As result, all nodes will be redirected to a
  /// trans.redirect_links(c, b);
  /// trans.redirect_links(b, a);
  /// trans.redirect_links(d, c);
  /// graph.commit(trans);
  ///
  /// // Graph after redirect
  /// // a -> {a, a, a} = {a}
  /// // b -> {a, a}    = {a}
  /// // c -> {a}       = {a}
  /// // d -> {}
  /// assert_eq!(get_node!(graph, Node::A, a).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, b).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, c).unwrap().tos, BTreeSet::from([a]));
  /// assert_eq!(get_node!(graph, Node::A, d).unwrap().tos, BTreeSet::new());
  /// # }
  /// ```
  pub fn redirect_links(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_links_vec.push((old_node, new_node));
  }

  /// Merge a graph and all its nodes
  ///
  /// The merged graph and this transaction should have the same context, otherwise use [`switch_context`](Graph::switch_context) first.
  ///
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA),
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx1 = Context::new();
  /// let mut graph1 = Graph::<Node>::new(&ctx1);
  /// let mut trans1 = Transaction::new(&ctx1);
  /// let n1 = trans1.insert(Node::A(NodeA{data: 1}));
  /// graph1.commit(trans1);
  ///
  /// let mut graph2 = Graph::<Node>::new(&ctx1);
  /// let mut trans2 = Transaction::new(&ctx1);
  /// let n2 = trans2.insert(Node::A(NodeA{data: 1}));
  /// // graph1 and graph2 have the same context
  /// trans2.merge(graph1);
  /// graph2.commit(trans2);
  /// // Now graph2 have all the nodes in graph1
  /// assert!(graph2.get(n1).is_some());
  ///
  /// let ctx2 = Context::new();
  /// let mut graph3 = Graph::<Node>::new(&ctx2);
  /// let mut trans3 = Transaction::new(&ctx2);
  /// // graph1 and graph2 have different context
  /// trans3.merge(graph2.switch_context(&ctx2));
  /// graph3.commit(trans3);
  /// // Now graph3 have all the nodes in graph2
  /// assert!(graph3.get(n1).is_some());
  /// assert!(graph3.get(n2).is_some());
  /// # }
  /// ```
  pub fn merge(&mut self, graph: Graph<NodeT, Arena>) {
    assert!(self.ctx_id == graph.ctx_id);
    self.inc_nodes.merge(graph.nodes);
  }

  /// Give up the transaction. Currently if a transaction is dropped without commit, it does not give a warning or panic. This issue may be fixed in the future.
  ///
  /// Currently this method does nothing.
  ///
  /// # Example
  /// ```
  /// use ttgraph::*;
  /// #[derive(TypedNode)]
  /// struct NodeA{
  ///   data: usize,
  /// }
  /// node_enum!{
  ///   enum Node{
  ///     A(NodeA)
  ///   }
  /// }
  ///
  /// # fn main() {
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::<Node>::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  ///
  /// trans.give_up();
  /// # }
  /// ```
  pub fn give_up(self) {
    drop(self);
  }
}

impl<'a, NodeT: NodeEnum, Arena> Debug for Transaction<'a, NodeT, Arena>
where
  NodeT: NodeEnum + Debug,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant> + Debug,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(&format!("Transaction<{}>", std::any::type_name::<NodeT>()))
      .field("ctx_id", &self.ctx_id)
      .field("alloc_nodes", &self.alloc_nodes)
      .field("inc_nodes", &self.inc_nodes)
      .field("dec_nodes", &self.dec_nodes)
      .field("mut_nodes", &Vec::from_iter(self.mut_nodes.iter().map(|(x, _)| *x)))
      .field("update_nodes", &Vec::from_iter(self.update_nodes.iter().map(|(x, _)| *x)))
      .field("redirect_all_links", &self.redirect_all_links_vec)
      .field("redirect_links", &self.redirect_links_vec)
      .finish()
  }
}

impl<'a, NodeT: NodeEnum> Extend<NodeT> for Transaction<'a, NodeT> {
  fn extend<T: IntoIterator<Item = NodeT>>(&mut self, iter: T) {
    for x in iter {
      self.insert(x);
    }
  }
}
