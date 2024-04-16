#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod test_bidirectional {
  use std::collections::BTreeSet;

  use tgraph::*;

  #[derive(TypedNode, Debug)]
  struct Node {
    #[group(group1, x)]
    a: NodeIndex,
    #[group(group1, x)]
    b: Vec<NodeIndex>,
    #[group(group2)]
    c: NodeIndex,
    inserted: NodeIndex,
    #[group(group2, x)]
    d: BTreeSet<NodeIndex>,
  }

  impl Default for Node {
    fn default() -> Self {
      Node {
        a: NodeIndex::empty(),
        b: Vec::default(),
        c: NodeIndex::empty(),
        inserted: NodeIndex::empty(),
        d: BTreeSet::default(),
      }
    }
  }

  node_enum! {
    #[derive(Debug)]
    enum Nodes{
      A(Node)
    }
  }

  #[test]
  fn test_link_group() {
    let ctx = Context::new();
    let mut graph = Graph::<Nodes>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let x1 = trans.insert(Nodes::A(Node::default()));
    let x2 = trans.insert(Nodes::A(Node::default()));
    let x3 = trans.insert(Nodes::A(Node::default()));
    let x4 = trans.insert(Nodes::A(Node::default()));
    let x5 = trans.insert(Nodes::A(Node {
      a: x1,
      b: vec![x2, x3],
      c: x4,
      inserted: x2,
      d: BTreeSet::from([x1, x3]),
    }));

    graph.commit(trans);

    let node = graph.get(x5).unwrap();
    assert_eq!(node.get_links_by_group("group1"), vec![x1, x2, x3]);
    assert_eq!(node.get_links_by_group("group2"), vec![x4, x1, x3]);
    assert_eq!(node.get_links_by_group("x"), vec![x1, x2, x3, x1, x3]);
  }

  #[derive(TypedNode, Debug)]
  struct NodeA {
    a: usize,
  }
  #[derive(TypedNode, Debug)]
  struct NodeB {
    b: usize,
  }
  #[derive(TypedNode, Debug)]
  struct NodeC {
    c: usize,
  }
  #[derive(TypedNode, Debug)]
  struct NodeD {
    d: usize,
  }

  node_enum! {
    #[derive(Debug)]
    enum MultiNodes{
      A(NodeA),
      B(NodeB),
      C(NodeC),
      D(NodeD),
    }
    group!{
      first{A, B},
      second{C, D},
      third{A, D},
      one{B},
      all{A, B, C, D},
    }
  }

  #[test]
  fn test_node_group() {
    let ctx = Context::new();
    let mut graph = Graph::<MultiNodes>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.insert(MultiNodes::A(NodeA { a: 1 }));
    let b = trans.insert(MultiNodes::B(NodeB { b: 2 }));
    let c = trans.insert(MultiNodes::C(NodeC { c: 3 }));
    let d = trans.insert(MultiNodes::D(NodeD { d: 4 }));

    graph.commit(trans);

    assert_eq!(Vec::from_iter(graph.iter_group("first").map(|(x, _)| x)), vec![a, b]);
    assert_eq!(Vec::from_iter(graph.iter_group("second").map(|(x, _)| x)), vec![c, d]);
    assert_eq!(Vec::from_iter(graph.iter_group("third").map(|(x, _)| x)), vec![a, d]);
    assert_eq!(Vec::from_iter(graph.iter_group("one").map(|(x, _)| x)), vec![b]);
    assert_eq!(Vec::from_iter(graph.iter_group("all").map(|(x, _)| x)), vec![a, b, c, d]);
  }
}
