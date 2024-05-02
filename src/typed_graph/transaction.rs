use std::collections::BTreeSet;

use visible::StructFields;

use super::*;
/// The transaction to modify a [`Graph`].
/// It is a operation recorder which have independent lifetime than the graph and does not hold reference to the graph.
/// The transaction and the graph should have been created from the same [`Context`] to ensure correctness.
#[StructFields(pub(crate))]
pub struct Transaction<'a, NodeT: NodeEnum> {
  ctx_id: Uuid,
  alloc_nodes: BTreeSet<NodeIndex>,
  inc_nodes: Arena<NodeIndex, NodeT>,
  dec_nodes: Vec<NodeIndex>,
  mut_nodes: Vec<(NodeIndex, MutFunc<'a, NodeT>)>,
  update_nodes: Vec<(NodeIndex, UpdateFunc<'a, NodeT>)>,
  redirect_all_links_vec: Vec<(NodeIndex, NodeIndex)>,
  redirect_links_vec: Vec<(NodeIndex, NodeIndex)>,
}

impl<'a, NodeT: NodeEnum> Transaction<'a, NodeT> {
  /// Make a empty transaction
  /// Please ensure the [`Graph`] and the [`Transaction`] use the same Context!
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
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// // In fact Transaction::<Node>::new(), but <Node> can be inferenced when commit
  /// let mut trans = Transaction::new(&ctx);
  /// trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// ```
  pub fn new(context: &Context) -> Self {
    let node_dist = Arc::clone(&context.node_dist);
    Transaction {
      ctx_id: context.id,
      alloc_nodes: BTreeSet::new(),
      inc_nodes: Arena::new(node_dist),
      dec_nodes: Vec::new(),
      mut_nodes: Vec::new(),
      update_nodes: Vec::new(),
      redirect_all_links_vec: Vec::new(),
      redirect_links_vec: Vec::new(),
    }
  }

  /// Allocate a new [`NodeIndex`] for a new node, use together with [`fill_back`](Transaction::fill_back)
  /// Useful when there is a cycle.
  /// # Panic
  /// If there is a node that is allocaed, but is not filled back, it is detected and panic to warn the user.
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
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// // Alloc some nodes
  /// let n1 = trans.alloc();
  /// let n2 = trans.alloc();
  /// let n3 = trans.alloc();
  /// // Build a circle
  /// trans.fill_back(n1, Node::A(NodeA{ last: n3, next: n2 }));
  /// trans.fill_back(n2, Node::A(NodeA{ last: n1, next: n3 }));
  /// trans.fill_back(n3, Node::A(NodeA{ last: n2, next: n1 }));
  /// graph.commit(trans);
  /// ```
  pub fn alloc(&mut self) -> NodeIndex {
    let idx = self.inc_nodes.alloc();
    self.alloc_nodes.insert(idx);
    idx
  }

  /// Fill back the data to a [`NodeIndex`] created by [`alloc`](Transaction::alloc)
  pub fn fill_back(&mut self, idx: NodeIndex, data: NodeT) {
    self.inc_nodes.fill_back(idx, data);
    self.alloc_nodes.remove(&idx);
  }

  /// Insert a new node into the graph, returns a [`NodeIndex`] pointing to the new node
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
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// // Get the node back by the returned NodeIndex idx
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  /// ```
  pub fn insert(&mut self, data: NodeT) -> NodeIndex {
    self.inc_nodes.insert(data)
  }

  /// Remove an existing node
  /// Note: nodes created by [`insert`](Transaction::insert) and [`alloc`](Transaction::alloc) in this uncommitted transaction can also be removed.
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
  /// ```
  pub fn remove(&mut self, node: NodeIndex) {
    if self.inc_nodes.remove(node).is_none() && !self.alloc_nodes.remove(&node) {
      self.dec_nodes.push(node);
    }
  }

  /// Mutate a node as with a closure `FnOnce(&mut NodeT)`.
  /// If the type of the node is previously known, use [`mut_node!`](crate::mut_node!) instead.
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
  ///     B(NodeA),
  ///   }
  /// }
  ///
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
  /// ```
  /// # Performance warning
  /// TTGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  /// Try not to create a node with a very large connectivity, or merge multiple operations into once.
  pub fn mutate<F>(&mut self, node: NodeIndex, func: F)
  where
    F: FnOnce(&mut NodeT) + 'a,
  {
    if self.inc_nodes.contains(node) {
      func(self.inc_nodes.get_mut(node).unwrap());
    } else {
      self.mut_nodes.push((node, Box::new(func)));
    }
  }

  /// Update a node with a closure `FnOnce(NodeT) -> NodeT`.
  /// If the type of the node is previously known, use [`update_node!`](crate::update_node!) instead.
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
  ///     B(NodeA),
  ///   }
  /// }
  ///
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
  /// ```
  /// # Performance warning
  /// TTGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  /// Try not to create a node with a very large connectivity, or merge multiple operations into once.
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
  /// Nodes in the [`Graph`] and new nodes in the [`Transaction`] are both redirected
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
  /// let context = Context::new();
  /// let mut graph = Graph::<Node>::new(&context);
  /// let mut trans = Transaction::new(&context);
  /// let a = trans.alloc();
  /// let b = trans.alloc();
  /// let c = trans.alloc();
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
  /// ```
  pub fn redirect_all_links(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_all_links_vec.push((old_node, new_node));
  }

  /// Redirect the connections from old_node to new_node
  /// Only nodes in the [`Graph`] is redirected, new nodes in the [`Transaction`] is not redirected
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
  /// let context = Context::new();
  /// let mut graph = Graph::<Node>::new(&context);
  /// let mut trans = Transaction::new(&context);
  /// let a = trans.alloc();
  /// let b = trans.alloc();
  /// let c = trans.alloc();
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
  /// ```
  pub fn redirect_links(&mut self, old_node: NodeIndex, new_node: NodeIndex) {
    self.redirect_links_vec.push((old_node, new_node));
  }

  /// Merge a graph and all its nodes
  /// The merged graph and this transaction should have the same context, otherwise use [`switch_context`](Graph::switch_context) first.
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
  /// ```
  pub fn merge(&mut self, graph: Graph<NodeT>) {
    assert!(self.ctx_id == graph.ctx_id);
    for (i, n) in graph.into_iter() {
      self.fill_back(i, n);
    }
  }

  /// Give up the transaction. Currently if a transaction is dropped without commit, it does not give a warning or panic. This issue may be fixed in the future.
  /// Currently this method does nothing.
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
  /// let ctx = Context::new();
  /// let mut graph = Graph::<Node>::new(&ctx);
  /// let mut trans = Transaction::<Node>::new(&ctx);
  /// let idx = trans.insert(Node::A(NodeA{data: 1}));
  ///
  /// trans.give_up();
  /// ```
  pub fn give_up(self) {
    drop(self);
  }
}

impl<'a, NodeT: NodeEnum + Debug> Debug for Transaction<'a, NodeT> {
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
