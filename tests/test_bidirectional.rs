#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod test_bidirectional {
  use std::collections::{BTreeSet, HashSet};
  use std::hash::Hash;

  use ttgraph::*;

  // Test set to set
  #[derive(TypedNode, Debug, Clone)]
  struct GraphNode {
    tos: BTreeSet<NodeIndex>,
    froms: BTreeSet<NodeIndex>,
    data: usize,
  }

  // Test direct to direct
  #[derive(TypedNode, Debug, Clone)]
  struct PairNode {
    the_other: NodeIndex,
  }

  // Test set to direct
  #[derive(TypedNode, Debug, Clone)]
  struct TreeNode {
    children: HashSet<NodeIndex>,
    father: NodeIndex,
  }

  // Test different type
  #[derive(TypedNode, Debug, Clone)]
  struct BoxNode {
    inside: BTreeSet<NodeIndex>,
  }

  #[derive(TypedNode, Debug, Clone)]
  struct DataNode {
    parent: NodeIndex,
    data: usize,
  }

  node_enum! {
    #[derive(Debug)]
    enum NodeType {
      GraphNode(GraphNode),
      PairNode(PairNode),
      TreeNode(TreeNode),
      BoxNode(BoxNode),
      DataNode(DataNode),
    }
    bidirectional!{
      GraphNode.tos <-> GraphNode.froms,
      PairNode.the_other <-> PairNode.the_other,
      TreeNode.father <-> TreeNode.children,
      BoxNode.inside <-> DataNode.parent
    }
  }

  struct GraphUnderTest {
    gn1: NodeIndex,
    gn2: NodeIndex,
    gn3: NodeIndex,
    pn1: NodeIndex,
    pn2: NodeIndex,
    pn3: NodeIndex,
    tn1: NodeIndex,
    tn2: NodeIndex,
    tn3: NodeIndex,
    tn4: NodeIndex,
    bn: NodeIndex,
    dn1: NodeIndex,
    dn2: NodeIndex,
    dn3: NodeIndex,
  }

  // gn1 -> gn2, gn3
  // gn2 -> gn2
  // gn3 -> gn1, gn2
  // pn1 <-> pn2, pn3 dangling
  // tn1 -> tn2, tn3
  // tn2 -> tn4
  // bn -> dn1, dn2, dn3, tn1, gn1, pn3
  fn build_graph() -> (Context, Graph<NodeType>, GraphUnderTest) {
    let ctx = Context::new();
    let mut graph = Graph::new(&ctx);
    let mut trans = Transaction::new(&ctx);

    let gn1 = trans.alloc();
    let gn2 = trans.alloc();
    let gn3 = trans.alloc();
    let pn1 = trans.alloc();
    let pn2 = trans.alloc();
    let pn3 = trans.alloc();
    let tn1 = trans.alloc();
    let tn2 = trans.alloc();
    let tn3 = trans.alloc();
    let tn4 = trans.alloc();
    let bn = trans.alloc();
    let dn1 = trans.alloc();
    let dn2 = trans.alloc();
    let dn3 = trans.alloc();

    trans.fill_back(
      gn1,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter(vec![gn2, gn3]),
        froms: BTreeSet::from_iter(vec![gn3]),
        data: 1,
      }),
    );
    trans.fill_back(
      gn2,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter(vec![gn2]),
        froms: BTreeSet::from_iter(vec![gn3]),
        data: 2,
      }),
    );
    trans.fill_back(
      gn3,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::new(),
        froms: BTreeSet::from_iter(vec![gn1]),
        data: 3,
      }),
    );

    trans.fill_back(pn1, NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }));
    trans.fill_back(pn2, NodeType::PairNode(PairNode { the_other: pn1 }));
    trans.fill_back(pn3, NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }));

    trans.fill_back(
      tn1,
      NodeType::TreeNode(TreeNode {
        children: HashSet::from_iter([tn2]),
        father: NodeIndex::empty(),
      }),
    );
    trans.fill_back(
      tn2,
      NodeType::TreeNode(TreeNode {
        children: HashSet::from_iter([tn4]),
        father: tn1,
      }),
    );
    trans.fill_back(
      tn3,
      NodeType::TreeNode(TreeNode { children: HashSet::new(), father: tn1 }),
    );
    trans.fill_back(
      tn4,
      NodeType::TreeNode(TreeNode {
        children: HashSet::new(),
        father: NodeIndex::empty(),
      }),
    );

    trans.fill_back(
      bn,
      NodeType::BoxNode(BoxNode {
        inside: BTreeSet::from_iter([dn1, dn3, tn1, gn1, pn3]),
      }),
    );
    trans.fill_back(dn1, NodeType::DataNode(DataNode { parent: bn, data: 1 }));
    trans.fill_back(dn2, NodeType::DataNode(DataNode { parent: bn, data: 2 }));
    trans.fill_back(
      dn3,
      NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 3 }),
    );

    graph.commit(trans);
    (
      ctx,
      graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    )
  }

  #[test]
  fn can_build_graph() {
    let (
      ctx,
      graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    ) = build_graph();

    // gn1 -> gn2, gn3
    // gn2 -> gn2
    // gn3 -> gn1, gn2
    // pn1 <-> pn2, pn3 dangling
    // tn1 -> tn2, tn3
    // tn2 -> tn4
    // bn -> dn1, dn2, dn3, tn1, gn1, pn3
    {
      let node = get_node!(graph, NodeType::GraphNode, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3]));
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1]));
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn1).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn2).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3, tn1, gn1, pn3]));
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn3).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 3);
    }

    graph.check_backlinks();
  }

  #[test]
  fn can_add_node() {
    let (
      ctx,
      mut graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    ) = build_graph();

    // gn1 -> gn2, gn3, gn4
    // gn2 -> gn2
    // gn3 -> gn1, gn2, gn4
    // gn4 -> gn2, gn3, gn4
    // pn1 <-> pn2, pn3 <-> pn4
    // tn1 -> tn2, tn3
    // tn2 -> tn4, tn5
    // bn -> dn1, dn2, dn3, dn4, gn1, tn1, pn3
    // bn2 -> dn5
    let mut trans = Transaction::new(&ctx);
    let gn4 = trans.alloc();
    trans.fill_back(
      gn4,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter([gn2, gn3, gn4]),
        froms: BTreeSet::from_iter([gn1, gn3]),
        data: 4,
      }),
    );
    let pn4 = trans.insert(NodeType::PairNode(PairNode { the_other: pn3 }));
    let tn5 = trans
      .insert(NodeType::TreeNode(TreeNode { children: HashSet::new(), father: tn2 }));
    let dn5 = trans.alloc();
    let bn2 =
      trans.insert(NodeType::BoxNode(BoxNode { inside: BTreeSet::from_iter([dn5]) }));
    let dn4 = trans.insert(NodeType::DataNode(DataNode { parent: bn, data: 4 }));
    trans.fill_back(
      dn5,
      NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 5 }),
    );

    graph.commit(trans);
    {
      let node = get_node!(graph, NodeType::GraphNode, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3, gn4]));
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn4]));
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn4).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 4);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn1).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn2).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn3).unwrap();
      assert_eq!(node.the_other, pn4);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn4).unwrap();
      assert_eq!(node.the_other, pn3);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4, tn5]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn5).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3, dn4, tn1, gn1, pn3]));
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn5]));
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn3).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn4).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 4);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn5).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 5);
    }

    graph.check_backlinks();
  }

  #[test]
  fn can_modify_and_update_node() {
    let (
      ctx,
      mut graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    ) = build_graph();

    // gn1 -> gn2, gn4
    // gn2 ->
    // gn3 -> gn2, gn4
    // gn4 -> gn2, gn3, gn4
    // pn2 <-> pn4, pn1, pn3 dangling
    // tn1 -> tn5, tn3
    // tn5 -> tn2, tn4
    // bn -> dn1, dn4, tn1, pn3
    // bn2 -> dn2, dn3, dn5, gn1
    let mut trans = Transaction::new(&ctx);

    let gn4 = trans.alloc();
    mut_node!(trans, NodeType::GraphNode, gn1, |x| {
      x.tos.remove(&gn3);
      x.froms.remove(&gn3);
    });
    mut_node!(trans, NodeType::GraphNode, gn2, |x| {
      x.tos.remove(&gn2);
    });
    update_node!(trans, NodeType::GraphNode, gn3, x, {
      GraphNode {
        tos: BTreeSet::from_iter([gn2]),
        froms: BTreeSet::from_iter([gn4]),
        ..x
      }
    });
    trans.fill_back(
      gn4,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter([gn2, gn4]),
        froms: BTreeSet::from_iter([gn1, gn3]),
        data: 4,
      }),
    );

    let pn4 = trans.insert(NodeType::PairNode(PairNode { the_other: pn2 }));
    let empty = NodeIndex::empty();
    mut_node!(trans, NodeType::PairNode, pn1, move |x| {
      x.the_other = empty;
    });
    mut_node!(trans, NodeType::PairNode, pn2, x, {
      x.the_other = NodeIndex::empty();
    });

    let tn5 = trans.insert(NodeType::TreeNode(TreeNode {
      children: HashSet::from_iter([tn2, tn4]),
      father: tn1,
    }));
    mut_node!(trans, NodeType::TreeNode, tn2, x, {
      x.father = tn5;
      x.children.remove(&tn4);
    });
    mut_node!(trans, NodeType::TreeNode, tn4, x, {
      x.father = tn5;
    });

    let dn4 =
      trans.insert(NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 4 }));
    mut_node!(trans, NodeType::BoxNode, bn, x, {
      x.inside.remove(&dn2);
      x.inside.remove(&dn3);
      x.inside.remove(&gn1);
      x.inside.insert(dn4);
    });
    let bn2 = trans.insert(NodeType::BoxNode(BoxNode {
      inside: BTreeSet::from_iter([dn2, dn3, gn1]),
    }));
    update_node!(trans, NodeType::DataNode, dn2, x, { DataNode { parent: bn2, ..x } });
    mut_node!(trans, NodeType::DataNode, dn3, x, {
      x.parent = NodeIndex::empty();
    });
    let dn5 = trans.insert(NodeType::DataNode(DataNode { parent: bn2, data: 5 }));

    graph.commit(trans);
    {
      let node = get_node!(graph, NodeType::GraphNode, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn4]));
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn4).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 4);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn1).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn2).unwrap();
      assert_eq!(node.the_other, pn4);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn4).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn5, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn2).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn5);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn5);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn5).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn4, tn1, pn3]));
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn2, dn3, dn5, gn1]));
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn2).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn3).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn4).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 4);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn5).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 5);
    }
    graph.check_backlinks();
  }

  #[test]
  fn can_remove_node() {
    let (
      ctx,
      mut graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    ) = build_graph();

    // gn1 -> gn2
    // gn2 -> gn2
    // pn1, pn3 dangling
    // tn1 -> tn3
    // bn -> dn1, dn2, tn1, gn1, pn3
    let mut trans = Transaction::new(&ctx);
    trans.remove(gn3);
    trans.remove(pn2);
    trans.remove(tn2);
    trans.remove(dn3);

    graph.commit(trans);

    {
      let node = get_node!(graph, NodeType::GraphNode, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2]));
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn3);
      assert!(node.is_none());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn1).unwrap();
      assert_eq!(node.the_other, NodeIndex::empty());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn2);
      assert!(node.is_none());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn2);
      assert!(node.is_none());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, NodeIndex::empty());
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, tn1, gn1, pn3]));
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn3);
      assert!(node.is_none());
    }

    graph.check_backlinks();
  }

  #[test]
  fn can_redirect_node() {
    let (
      ctx,
      mut graph,
      GraphUnderTest {
        gn1,
        gn2,
        gn3,
        pn1,
        pn2,
        pn3,
        tn1,
        tn2,
        tn3,
        tn4,
        bn,
        dn1,
        dn2,
        dn3,
      },
    ) = build_graph();

    // Start:
    // gn1 -> gn2, gn3 ; <- gn3
    // gn2 -> gn2      ; <- gn1, gn2, gn3
    // gn3 -> gn1, gn2 ; <- gn1
    // Redirect gn2 -> gn3
    // gn1 -> gn3 ; <- gn3
    // gn2 -> gn3      ; <- gn1, gn3
    // gn3 -> gn1, gn3 ; <- gn1
    // Removed:
    // gn1 -> gn2 ; gn2 -> gn2; gn2 <- gn2; gn3 -> gn2;
    // Add:
    // gn2 -> gn3, gn3 -> gn3
    // result:
    // gn1 -> gn3      ; <- gn3
    // gn2 -> gn3      ; <-
    // gn3 -> gn1, gn3 ; <- gn1, gn2, gn3

    // pn1 <-> pn3, pn2 dangling
    // tn1 -> tn3
    // tn3 -> tn4
    // bn -> tn1, gn1, pn3
    // bn2 -> dn1, dn2, dn3, tn1, gn1, pn3

    let mut trans = Transaction::new(&ctx);

    trans.redirect_links(gn2, gn3);
    trans.redirect_links(pn2, pn3);
    trans.redirect_links(tn2, tn3);
    let bn2 = trans.insert(NodeType::BoxNode(BoxNode { inside: BTreeSet::new() }));
    trans.redirect_links(bn, bn2);

    graph.commit(trans);

    {
      let node = get_node!(graph, NodeType::GraphNode, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::GraphNode, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3]));
      assert_eq!(node.data, 3);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn1).unwrap();
      assert_eq!(node.the_other, pn3);
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn2).unwrap();
      assert_eq!(node.the_other, NodeIndex::empty());
    }
    {
      let node = get_node!(graph, NodeType::PairNode, pn3).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([]));
      assert_eq!(node.father, NodeIndex::empty());
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn3).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = get_node!(graph, NodeType::TreeNode, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn3);
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([tn1, gn1, pn3]));
    }
    {
      let node = get_node!(graph, NodeType::BoxNode, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3]));
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn1).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 1);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn2).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 2);
    }
    {
      let node = get_node!(graph, NodeType::DataNode, dn3).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 3);
    }

    graph.check_backlinks();
  }
}
