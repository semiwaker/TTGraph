#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod tests_typed {
  use std::collections::BTreeSet;

  use tgraph::*;

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
    let n = trans.new_node(MyNodeEnum::Empty(NodeEmpty { x: 0 }));
    graph.commit(trans);
    for (idx, n) in iter_nodes!(graph, MyNodeEnum::Empty) {
      eprintln!("{:?} {:?}", idx, n);
    }

    let mut trans = Transaction::new(&context);
    let b = trans.alloc_node();
    let a = trans.new_node(MyNodeEnum::A(NodeA { to: b, name: "A".to_string() }));
    trans.fill_back_node(b, MyNodeEnum::B(NodeB { a, x: n, data1: 3 }));

    graph.commit(trans);
    for (idx, n) in graph.iter_nodes() {
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
    println!("{:?}", graph.get_node(b).unwrap().data_ref_by_name::<usize>("data1"));
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

    let a = trans.alloc_node();
    let b = trans.alloc_node();
    let c = trans.alloc_node();
    let d = trans.new_node(TestNode::CNode(CNode { tos: BTreeSet::new() }));
    trans.fill_back_node(c, TestNode::CNode(CNode { tos: BTreeSet::from_iter([d]) }));
    trans.fill_back_node(b, TestNode::CNode(CNode { tos: BTreeSet::from_iter([c, d]) }));
    trans
      .fill_back_node(a, TestNode::CNode(CNode { tos: BTreeSet::from_iter([b, c, d]) }));

    graph.commit(trans);

    println!("{}", graph);
    trans = Transaction::new(&context);

    trans.redirect_node(c, b);
    trans.redirect_node(b, a);
    trans.redirect_node(d, c);

    graph.commit(trans);

    println!("{}", graph);
  }
}
