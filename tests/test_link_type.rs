#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
#[cfg(feature = "debug")]
mod test_link_type {
  use std::collections::{BTreeSet, HashSet};

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

  #[derive(TypedNode)]
  struct NodeC {
    x: NodeIndex,
    y: NodeIndex,
  }

  node_enum! {
    enum Node{
      A(NodeA),
      B(NodeB),
      C(NodeC),
    }
    group!{
      AB{A, B},
      BC{B, C},
    }
    link_type!{
      A.to_a: A,
      A.to_b: B,
      A.x: {A, B},
      B.to_a: A,
      B.to_b: B,
      B.x: BC,
      C.x: AB,
      C.y: {AB, C},
    }
  }

  #[test]
  fn test_link_type() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    let c = trans.alloc();
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
        x: c,
      }),
    );
    trans.fill_back(c, Node::C(NodeC { x: b, y: a }));

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

  #[test]
  #[should_panic]
  fn test_link_type7() {
    let ctx = Context::new();
    let mut graph = Graph::<Node>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let a = trans.alloc();
    let b = trans.alloc();
    let c = trans.alloc();
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
        x: c,
      }),
    );
    trans.fill_back(c, Node::C(NodeC { x: c, y: c }));

    graph.commit(trans);
  }

  #[derive(TypedNode, Debug)]
  struct Left1 {
    #[group(l1)]
    g1: NodeIndex,
    #[group(l2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Left2 {
    #[group(l1)]
    g1: NodeIndex,
    #[group(l2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Left3 {
    #[group(l1)]
    g1: NodeIndex,
    #[group(l2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Left4 {
    #[group(l1)]
    g1: NodeIndex,
    #[group(l2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Right1 {
    #[group(r1)]
    g1: NodeIndex,
    #[group(r2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Right2 {
    #[group(r1)]
    g1: NodeIndex,
    #[group(r2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Right3 {
    #[group(r1)]
    g1: NodeIndex,
    #[group(r2)]
    g2: NodeIndex,
  }

  #[derive(TypedNode, Debug)]
  struct Right4 {
    #[group(r1)]
    g1: NodeIndex,
    #[group(r2)]
    g2: NodeIndex,
  }

  node_enum!{
    enum LR{
      Left1(Left1),
      Left2(Left2),
      Left3(Left3),
      Left4(Left4),
      Right1(Right1),
      Right2(Right2),
      Right3(Right3),
      Right4(Right4),
    }
    group!{
      left { Left1, Left2, Left3, Left4},
      right { Right1, Right2, Right3, Right4},
      LU {Left1, Left2},
      LD {Left3, Left4},
      RU {Right1, Right2},
      RD {Right3, Right4},
    }
    link_type!{
      left.l1: RU,
      left.l2: RD,
      right.r1: LU,
      right.r2: LD,
    }
  }

  #[test]
  fn test_group_link_1() {
    let ctx = Context::new();
    let mut graph = Graph::<LR>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let l1 = trans.alloc();
    let l2 = trans.alloc();
    let l3 = trans.alloc();
    let l4 = trans.alloc();
    let r1 = trans.alloc();
    let r2 = trans.alloc();
    let r3 = trans.alloc();
    let r4 = trans.alloc();

    trans.fill_back(l1, LR::Left1(Left1 { g1: r1, g2: r3 }));
    trans.fill_back(l2, LR::Left2(Left2 { g1: r2, g2: r3 }));
    trans.fill_back(l3, LR::Left3(Left3 { g1: r1, g2: r4 }));
    trans.fill_back(l4, LR::Left4(Left4 { g1: r2, g2: r4 }));

    trans.fill_back(r1, LR::Right1(Right1 { g1: l1, g2: l3 }));
    trans.fill_back(r2, LR::Right2(Right2 { g1: l2, g2: l3 }));
    trans.fill_back(r3, LR::Right3(Right3 { g1: l1, g2: l4 }));
    trans.fill_back(r4, LR::Right4(Right4 { g1: l2, g2: l4 }));

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_group_link_2() {
    let ctx = Context::new();
    let mut graph = Graph::<LR>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let l1 = trans.alloc();
    let l2 = trans.alloc();
    let l3 = trans.alloc();
    let l4 = trans.alloc();
    let r1 = trans.alloc();
    let r2 = trans.alloc();
    let r3 = trans.alloc();
    let r4 = trans.alloc();

    trans.fill_back(l1, LR::Left1(Left1 { g1: r1, g2: r1 }));
    trans.fill_back(l2, LR::Left2(Left2 { g1: r2, g2: r3 }));
    trans.fill_back(l3, LR::Left3(Left3 { g1: r1, g2: r4 }));
    trans.fill_back(l4, LR::Left4(Left4 { g1: r2, g2: r4 }));

    trans.fill_back(r1, LR::Right1(Right1 { g1: l1, g2: l3 }));
    trans.fill_back(r2, LR::Right2(Right2 { g1: l2, g2: l3 }));
    trans.fill_back(r3, LR::Right3(Right3 { g1: l1, g2: l4 }));
    trans.fill_back(r4, LR::Right4(Right4 { g1: l2, g2: l4 }));

    graph.commit(trans);
  }

  #[test]
  #[should_panic]
  fn test_group_link_3() {
    let ctx = Context::new();
    let mut graph = Graph::<LR>::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let l1 = trans.alloc();
    let l2 = trans.alloc();
    let l3 = trans.alloc();
    let l4 = trans.alloc();
    let r1 = trans.alloc();
    let r2 = trans.alloc();
    let r3 = trans.alloc();
    let r4 = trans.alloc();

    trans.fill_back(l1, LR::Left1(Left1 { g1: r1, g2: r3 }));
    trans.fill_back(l2, LR::Left2(Left2 { g1: r2, g2: r3 }));
    trans.fill_back(l3, LR::Left3(Left3 { g1: r1, g2: r4 }));
    trans.fill_back(l4, LR::Left4(Left4 { g1: r2, g2: r4 }));

    trans.fill_back(r1, LR::Right1(Right1 { g1: l1, g2: l3 }));
    trans.fill_back(r2, LR::Right2(Right2 { g1: l2, g2: l3 }));
    trans.fill_back(r3, LR::Right3(Right3 { g1: l1, g2: l4 }));
    trans.fill_back(r4, LR::Right4(Right4 { g1: l4, g2: l4 }));

    graph.commit(trans);
  }
}
