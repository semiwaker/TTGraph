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

    let x1 = trans.new_node(Nodes::A(Node::default()));
    let x2 = trans.new_node(Nodes::A(Node::default()));
    let x3 = trans.new_node(Nodes::A(Node::default()));
    let x4 = trans.new_node(Nodes::A(Node::default()));
    let x5 = trans.new_node(Nodes::A(Node {
      a: x1,
      b: vec![x2, x3],
      c: x4,
      inserted: x2,
      d: BTreeSet::from([x1, x3]),
    }));

    graph.commit(trans);

    let node = graph.get_node(x5).unwrap();
    assert_eq!(node.get_link_by_group("group1"), vec![x1, x2, x3]);
    assert_eq!(node.get_link_by_group("group2"), vec![x4, x1, x3]);
    assert_eq!(node.get_link_by_group("x"), vec![x1, x2, x3, x1, x3]);
  }
}
