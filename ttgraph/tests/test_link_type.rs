#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
#[cfg(feature = "debug")]
mod test_link_type {
  use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
  };

  use ttgraph::*;

  #[derive(TypedNode)]
  struct NodeA {
    to_a: NodeIndex,
    to_b: BTreeSet<NodeIndex>,
    x: NodeIndex,
  }

  #[derive(TypedNode)]
  struct NodeB {
    to_a: Vec<NodeIndex>,
    to_b: HashSet<NodeIndex>,
    x: NodeIndex,
  }

  node_enum! {
    enum Node{
      A(NodeA),
      B(NodeB),
    }
    link_type!{
      A.to_a: A,
      A.to_b: B,
      A.x: {A, B},
      B.to_a: A,
      B.to_b: B,
    }
  }

  #[test]
  fn test_link_type() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: a,
        to_b: BTreeSet::from([b]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a],
        to_b: HashSet::from([b]),
        x: b,
      }),
    );

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_link_type2() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: b,
        to_b: BTreeSet::from([b]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a],
        to_b: HashSet::from([b]),
        x: b,
      }),
    );

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_link_type3() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: a,
        to_b: BTreeSet::from([a]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a],
        to_b: HashSet::from([b]),
        x: b,
      }),
    );

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_link_type4() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: a,
        to_b: BTreeSet::from([b]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a, b],
        to_b: HashSet::from([b]),
        x: b,
      }),
    );

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_link_type5() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: a,
        to_b: BTreeSet::from([b]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a],
        to_b: HashSet::from([a, b]),
        x: b,
      }),
    );

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_link_type6() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    trans.fill_back(
      a,
      Node::A(NodeA {
        to_a: a,
        to_b: BTreeSet::from([b]),
        x: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      b,
      Node::B(NodeB {
        to_a: vec![a],
        to_b: HashSet::from([b]),
        x: b,
      }),
    );

    graph.commit(trans);

    trans = Transaction::new(&ctx);
    mut_node!(trans, Node::A, a, x, {
      x.to_b.insert(a);
    });
    graph.commit(trans);
  }
}
