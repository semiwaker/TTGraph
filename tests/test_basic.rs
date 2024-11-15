#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod tests_typed {
  use ::ordermap::OrderSet;
  use serde::{Deserialize, Serialize};

  use ttgraph::{
    serialize::{deserialize_graph, GraphSerializer},
    *,
  };

  #[derive(TypedNode, Debug, Serialize, Deserialize)]
  struct NodeA {
    to: NodeIndex,
    name: String,
  }

  #[derive(TypedNode, Debug, Serialize, Deserialize)]
  struct NodeB {
    a: NodeIndex,
    x: NodeIndex,
    data1: usize,
  }

  node_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    enum MyNodeEnum {
      A(NodeA),
      B(NodeB),
      Empty(NodeEmpty),
    }
  }

  #[derive(TypedNode, Clone, Debug, Serialize, Deserialize)]
  struct NodeEmpty {
    x: usize,
  }

  #[test]
  fn can_compile() {
    let context = Context::new();
    let mut graph = Graph::<MyNodeEnum>::new(&context);
    let mut trans = Transaction::new(&context);
    let n = trans.insert(MyNodeEnum::Empty(NodeEmpty { x: 0 }));
    graph.commit(trans);
    for (idx, n) in iter_nodes!(graph, MyNodeEnum::Empty) {
      eprintln!("{:?} {:?}", idx, n);
    }

    let mut trans = Transaction::new(&context);
    let b = alloc_node!(trans, MyNodeEnum::B);
    let a = trans.insert(MyNodeEnum::A(NodeA { to: b, name: "A".to_string() }));
    trans.fill_back(b, MyNodeEnum::B(NodeB { a, x: n, data1: 3 }));

    graph.commit(trans);
    for (idx, n) in graph.iter() {
      eprintln!("{:?} {:?}", idx, n);
    }
    for (idx, n) in iter_nodes!(graph, MyNodeEnum::A) {
      eprintln!("{:?} {:?}", idx, n);
    }
    for (idx, n) in iter_nodes!(graph, MyNodeEnum::B) {
      eprintln!("{:?} {:?}", idx, n);
    }
    println!("{:?}", graph);
    println!("{:?}", NodeB::link_types());
    println!("{:?}", NodeB::link_mirrors());
    println!("{:?}", NodeB::link_names());
    println!("{:?}", NodeA::data_names());
    println!("{:?}", NodeB::data_names());
    println!("{:?}", get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<String>("name"));
    println!("{:?}", get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<usize>("name"));
    println!("{:?}", get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<String>("data1"));
    println!("{:?}", get_node!(graph, MyNodeEnum::B, b).unwrap().data_ref_by_name::<usize>("data1"));
    println!("{:?}", graph.get(b).unwrap().data_ref_by_name::<usize>("data1"));
    println!("{:?}", discriminant!(MyNodeEnum::A));
  }

  #[derive(TypedNode, Debug, Serialize, Deserialize)]
  struct CNode {
    tos: OrderSet<NodeIndex>,
  }

  node_enum! {
    #[derive(Debug, Serialize, Deserialize)]
    enum TestNode {
      CNode(CNode),
    }
  }

  #[test]
  fn redirect_test() {
    let context = Context::new();
    let mut graph = Graph::<TestNode>::new(&context);
    let mut trans = Transaction::new(&context);

    let a = alloc_node!(trans, TestNode::CNode);
    let b = alloc_node!(trans, TestNode::CNode);
    let c = alloc_node!(trans, TestNode::CNode);
    let d = trans.insert(TestNode::CNode(CNode { tos: OrderSet::new() }));
    trans.fill_back(c, TestNode::CNode(CNode { tos: OrderSet::from_iter([d]) }));
    trans.fill_back(b, TestNode::CNode(CNode { tos: OrderSet::from_iter([c, d]) }));
    trans.fill_back(a, TestNode::CNode(CNode { tos: OrderSet::from_iter([b, c, d]) }));

    graph.commit(trans);

    println!("{:?}", graph);
    trans = Transaction::new(&context);

    trans.redirect_links(c, b);
    trans.redirect_links(b, a);
    trans.redirect_links(d, c);

    graph.commit(trans);

    println!("{:?}", graph);

    let serialized = serde_json::to_string(&graph).unwrap();
    println!("{}", serialized);
    println!("{}", serde_json::to_string(&GraphSerializer::<TestNode>::from(graph)).unwrap());

    let deserialized: GraphSerializer<TestNode> = serde_json::from_str(&serialized).unwrap();
    let (ctx2, graph2) = deserialize_graph(deserialized);
    println!("{:?}", graph2);
  }

  #[test]
  fn uncommit_test() {
    let context = Context::new();
    let mut graph = Graph::<TestNode>::new(&context);
    let mut trans = Transaction::<TestNode>::new(&context);

    let a = alloc_node!(trans, TestNode::CNode);
    let b = alloc_node!(trans, TestNode::CNode);
    let c = alloc_node!(trans, TestNode::CNode);
    let d = trans.insert(TestNode::CNode(CNode { tos: OrderSet::new() }));
    trans.fill_back(c, TestNode::CNode(CNode { tos: OrderSet::from_iter([d]) }));
    trans.fill_back(b, TestNode::CNode(CNode { tos: OrderSet::from_iter([c, d]) }));
    trans.fill_back(a, TestNode::CNode(CNode { tos: OrderSet::from_iter([b, c, d]) }));
  }

  #[derive(Debug, TypedNode)]
  struct VecNode {
    x: Vec<NodeIndex>,
  }

  node_enum! {
    #[derive(Debug)]
    enum VecNodeEnum{
      VecNode(VecNode)
    }
  }
  #[test]
  fn test_vec_empty() {
    let ctx = Context::new();
    let mut graph = Graph::new(&ctx);
    let mut trans = Transaction::new(&ctx);
    let a = alloc_node!(trans, VecNodeEnum::VecNode);
    let b = trans.insert(VecNodeEnum::VecNode(VecNode { x: vec![a] }));
    trans.fill_back(a, VecNodeEnum::VecNode(VecNode { x: Vec::new() }));
    graph.commit(trans);
    println!("{:?}", graph);

    let mut trans = Transaction::new(&ctx);
    mut_node!(trans, VecNodeEnum::VecNode, b, |b| { b.x.push(NodeIndex::empty()) });
    graph.commit(trans);
    println!("{:?}", graph);
  }
}
