#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod tests_typed {
  use std::collections::BTreeSet;

  use ttgraph::*;

  #[derive(TypedNode, Debug)]
  struct NodeA {
    to: NodeIndex,
    name: String,
  }

  #[derive(TypedNode, Debug)]
  struct NodeB {
    a: NodeIndex,
    x: NodeIndex,
    data1: usize,
  }

  node_enum! {
    #[derive(Debug)]
    enum MyNodeEnum {
      A(NodeA),
      B(NodeB),
      Empty(NodeEmpty),
    }
  }

  #[derive(TypedNode, Clone, Debug)]
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
    let b = trans.alloc();
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
    println!(
      "{:?}",
      get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<String>("name")
    );
    println!(
      "{:?}",
      get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<usize>("name")
    );
    println!(
      "{:?}",
      get_node!(graph, MyNodeEnum::A, a).unwrap().data_ref_by_name::<String>("data1")
    );
    println!(
      "{:?}",
      get_node!(graph, MyNodeEnum::B, b).unwrap().data_ref_by_name::<usize>("data1")
    );
    println!("{:?}", graph.get(b).unwrap().data_ref_by_name::<usize>("data1"));
    // for (idx, n) in Edge::iter_by_type(graph) {}
  }

  #[derive(TypedNode, Debug)]
  struct CNode {
    tos: BTreeSet<NodeIndex>,
  }

  node_enum! {
    #[derive(Debug)]
    enum TestNode {
      CNode(CNode),
    }
  }

  #[test]
  fn redirect_test() {
    let context = Context::new();
    let mut graph = Graph::<TestNode>::new(&context);
    let mut trans = Transaction::new(&context);

    let a = trans.alloc();
    let b = trans.alloc();
    let c = trans.alloc();
    let d = trans.insert(TestNode::CNode(CNode { tos: BTreeSet::new() }));
    trans.fill_back(c, TestNode::CNode(CNode { tos: BTreeSet::from_iter([d]) }));
    trans.fill_back(b, TestNode::CNode(CNode { tos: BTreeSet::from_iter([c, d]) }));
    trans.fill_back(a, TestNode::CNode(CNode { tos: BTreeSet::from_iter([b, c, d]) }));

    graph.commit(trans);

    println!("{:?}", graph);
    trans = Transaction::new(&context);

    trans.redirect_links(c, b);
    trans.redirect_links(b, a);
    trans.redirect_links(d, c);

    graph.commit(trans);

    println!("{:?}", graph);
  }

  #[test]
  fn uncommit_test() {
    let context = Context::new();
    let mut graph = Graph::<TestNode>::new(&context);
    let mut trans = Transaction::<TestNode>::new(&context);

    let a = trans.alloc();
    let b = trans.alloc();
    let c = trans.alloc();
    let d = trans.insert(TestNode::CNode(CNode { tos: BTreeSet::new() }));
    trans.fill_back(c, TestNode::CNode(CNode { tos: BTreeSet::from_iter([d]) }));
    trans.fill_back(b, TestNode::CNode(CNode { tos: BTreeSet::from_iter([c, d]) }));
    trans.fill_back(a, TestNode::CNode(CNode { tos: BTreeSet::from_iter([b, c, d]) }));
  }
}
