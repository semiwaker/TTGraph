use std::collections::BTreeSet;

use visible::StructFields;

use super::*;
/// The transaction to modify a [`Graph`].
/// It is a operation recorder which have independent lifetime than the graph and does not hold reference to the graph.
/// The transaction and the graph should have been created from the same [`Context`] to ensure correctness.
#[StructFields(pub)]
pub struct Transaction<'a, NodeT: NodeEnum> {
  ctx_id: Uuid,
  alloc_nodes: BTreeSet<NodeIndex>,
  inc_nodes: Arena<NodeT, NodeIndex>,
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
  /// use tgraph::*;
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
  /// trans.new_node(Node::A(NodeA{data: 1}));
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

  /// Allocate a new [`NodeIndex`] for a new node, use together with [`fill_back_node`](Transaction::fill_back_node)
  /// Useful when there is a cycle.
  /// # Panic
  /// If there is a node that is allocaed, but is not filled back, it is detected and panic to warn the user.
  /// # Example
  /// ```
  /// use tgraph::*;
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
  /// let n1 = trans.alloc_node();
  /// let n2 = trans.alloc_node();
  /// let n3 = trans.alloc_node();
  /// // Build a circle
  /// trans.fill_back_node(n1, Node::A(NodeA{ last: n3, next: n2 }));
  /// trans.fill_back_node(n2, Node::A(NodeA{ last: n1, next: n3 }));
  /// trans.fill_back_node(n3, Node::A(NodeA{ last: n2, next: n1 }));
  /// graph.commit(trans);
  /// ```
  pub fn alloc_node(&mut self) -> NodeIndex {
    let idx = self.inc_nodes.alloc();
    self.alloc_nodes.insert(idx);
    idx
  }

  /// Fill back the data to a [`NodeIndex`] created by [`alloc_node`](Transaction::alloc_node)
  pub fn fill_back_node(&mut self, idx: NodeIndex, data: NodeT) {
    self.inc_nodes.fill_back(idx, data);
    self.alloc_nodes.remove(&idx);
  }

  /// Put a new node into the graph, returns a [`NodeIndex`] pointing to the new node
  /// # Example
  /// ```
  /// use tgraph::*;
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
  /// let idx = trans.new_node(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// // Get the node back by the returned NodeIndex idx
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  /// ```
  pub fn new_node(&mut self, data: NodeT) -> NodeIndex {
    self.inc_nodes.insert(data)
  }

  /// Remove an existing node by the [`NodeIndex`]
  /// Note: nodes created by [`new_node`](Transaction::new_node) and [`alloc_node`](Transaction::alloc_node) in this uncommitted transaction can also be removed.
  /// Example:
  /// ```
  /// use tgraph::*;
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
  /// let idx = trans.new_node(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert!(graph.get_node(idx).is_some());
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.remove_node(idx);
  /// graph.commit(trans);
  /// // Now the node is removed
  /// assert!(graph.get_node(idx).is_none());
  /// ```
  pub fn remove_node(&mut self, node: NodeIndex) {
    if self.inc_nodes.remove(node).is_none() && !self.alloc_nodes.remove(&node) {
      self.dec_nodes.push(node);
    }
  }

  /// Mutate a node as with a closure `FnOnce(&mut NodeT)`.
  /// If the type of the node is previously known, use [`mut_node!`](crate::mut_node!) instead.
  /// # Example
  /// ```
  /// use tgraph::*;
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
  /// let idx = trans.new_node(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.mut_node(idx, |node|{
  ///   if let Node::A(x) = node{
  ///     x.data = 2
  ///   }
  /// });
  /// graph.commit(trans);
  /// // Now the data field of the node is 2
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 2);
  /// ```
  /// # Performance warning
  /// TGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  /// Try not to create a node with a very large connectivity, or merge multiple operations into once.
  pub fn mut_node<F>(&mut self, node: NodeIndex, func: F)
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
  /// use tgraph::*;
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
  /// let idx = trans.new_node(Node::A(NodeA{data: 1}));
  /// graph.commit(trans);
  /// assert_eq!(get_node!(graph, Node::A, idx).unwrap().data, 1);
  ///
  /// trans = Transaction::new(&ctx);
  /// trans.update_node(idx, |node| {
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
  /// TGraph does not know which link the user modified, so it always assumes all old links are removed and new links are added.
  /// Try not to create a node with a very large connectivity, or merge multiple operations into once.
  pub fn update_node<F>(&mut self, node: NodeIndex, func: F)
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
  /// use tgraph::*;
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
  /// let a = trans.alloc_node();
  /// let b = trans.alloc_node();
  /// let c = trans.alloc_node();
  /// let d = trans.new_node(Node::A(NodeA { tos: BTreeSet::new() }));
  /// trans.fill_back_node(c, Node::A(NodeA { tos: BTreeSet::from_iter([d]) }));
  /// trans.fill_back_node(b, Node::A(NodeA { tos: BTreeSet::from_iter([c, d]) }));
  /// trans.fill_back_node(a, Node::A(NodeA { tos: BTreeSet::from_iter([b, c, d]) }));
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
  /// use tgraph::*;
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
  /// let a = trans.alloc_node();
  /// let b = trans.alloc_node();
  /// let c = trans.alloc_node();
  /// let d = trans.new_node(Node::A(NodeA { tos: BTreeSet::new() }));
  /// trans.fill_back_node(c, Node::A(NodeA { tos: BTreeSet::from_iter([d]) }));
  /// trans.fill_back_node(b, Node::A(NodeA { tos: BTreeSet::from_iter([c, d]) }));
  /// trans.fill_back_node(a, Node::A(NodeA { tos: BTreeSet::from_iter([b, c, d]) }));
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
  /// use tgraph::*;
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
  /// let n1 = trans1.new_node(Node::A(NodeA{data: 1}));
  /// graph1.commit(trans1);
  ///
  /// let mut graph2 = Graph::<Node>::new(&ctx1);
  /// let mut trans2 = Transaction::new(&ctx1);
  /// let n2 = trans2.new_node(Node::A(NodeA{data: 1}));
  /// // graph1 and graph2 have the same context
  /// trans2.merge_graph(graph1);
  /// graph2.commit(trans2);
  /// // Now graph2 have all the nodes in graph1
  /// assert!(graph2.get_node(n1).is_some());
  ///
  /// let ctx2 = Context::new();
  /// let mut graph3 = Graph::<Node>::new(&ctx2);
  /// let mut trans3 = Transaction::new(&ctx2);
  /// // graph1 and graph2 have different context
  /// trans3.merge_graph(graph2.switch_context(&ctx2));
  /// graph3.commit(trans3);
  /// // Now graph3 have all the nodes in graph2
  /// assert!(graph3.get_node(n1).is_some());
  /// assert!(graph3.get_node(n2).is_some());
  /// ```
  pub fn merge_graph(&mut self, graph: Graph<NodeT>) {
    assert!(self.ctx_id == graph.ctx_id);
    for (i, n) in graph.into_iter() {
      self.fill_back_node(i, n);
    }
  }

  /// Give up the transaction. Currently if a transaction is dropped without commit, it does not give a warning. This issue may be fixed in the future.
  /// # Example
  /// ```
  /// use tgraph::*;
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
  /// let idx = trans.new_node(Node::A(NodeA{data: 1}));
  ///
  /// trans.give_up();
  /// ```
  pub fn give_up(self) {
    drop(self);
  }
}
