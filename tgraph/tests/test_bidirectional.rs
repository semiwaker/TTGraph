#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod test_bidirectional {
  use std::collections::{BTreeSet, HashSet};
  use std::hash::Hash;

  use tgraph::typed_graph::*;

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

    let gn1 = trans.alloc_node();
    let gn2 = trans.alloc_node();
    let gn3 = trans.alloc_node();
    let pn1 = trans.alloc_node();
    let pn2 = trans.alloc_node();
    let pn3 = trans.alloc_node();
    let tn1 = trans.alloc_node();
    let tn2 = trans.alloc_node();
    let tn3 = trans.alloc_node();
    let tn4 = trans.alloc_node();
    let bn = trans.alloc_node();
    let dn1 = trans.alloc_node();
    let dn2 = trans.alloc_node();
    let dn3 = trans.alloc_node();

    trans.fill_back_node(
      gn1,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter(vec![gn2, gn3]),
        froms: BTreeSet::from_iter(vec![gn3]),
        data: 1,
      }),
    );
    trans.fill_back_node(
      gn2,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter(vec![gn2]),
        froms: BTreeSet::from_iter(vec![gn3]),
        data: 2,
      }),
    );
    trans.fill_back_node(
      gn3,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::new(),
        froms: BTreeSet::from_iter(vec![gn1]),
        data: 3,
      }),
    );

    trans.fill_back_node(
      pn1,
      NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }),
    );
    trans.fill_back_node(pn2, NodeType::PairNode(PairNode { the_other: pn1 }));
    trans.fill_back_node(
      pn3,
      NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }),
    );

    trans.fill_back_node(
      tn1,
      NodeType::TreeNode(TreeNode {
        children: HashSet::from_iter([tn2]),
        father: NodeIndex::empty(),
      }),
    );
    trans.fill_back_node(
      tn2,
      NodeType::TreeNode(TreeNode {
        children: HashSet::from_iter([tn4]),
        father: tn1,
      }),
    );
    trans.fill_back_node(
      tn3,
      NodeType::TreeNode(TreeNode { children: HashSet::new(), father: tn1 }),
    );
    trans.fill_back_node(
      tn4,
      NodeType::TreeNode(TreeNode {
        children: HashSet::new(),
        father: NodeIndex::empty(),
      }),
    );

    trans.fill_back_node(
      bn,
      NodeType::BoxNode(BoxNode {
        inside: BTreeSet::from_iter([dn1, dn3, tn1, gn1, pn3]),
      }),
    );
    trans.fill_back_node(dn1, NodeType::DataNode(DataNode { parent: bn, data: 1 }));
    trans.fill_back_node(dn2, NodeType::DataNode(DataNode { parent: bn, data: 2 }));
    trans.fill_back_node(
      dn3,
      NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 3 }),
    );

    graph.commit(trans);
    (ctx, graph, GraphUnderTest {
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
    })
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
      let node = GraphNode::get_by_type(&graph, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3]));
      assert_eq!(node.data, 2);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1]));
      assert_eq!(node.data, 3);
    }
    {
      let node = PairNode::get_by_type(&graph, pn1).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = PairNode::get_by_type(&graph, pn2).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = PairNode::get_by_type(&graph, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = BoxNode::get_by_type(&graph, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3, tn1, gn1, pn3]));
    }
    {
      let node = DataNode::get_by_type(&graph, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = DataNode::get_by_type(&graph, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = DataNode::get_by_type(&graph, dn3).unwrap();
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
    let gn4 = trans.alloc_node();
    trans.fill_back_node(
      gn4,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter([gn2, gn3, gn4]),
        froms: BTreeSet::from_iter([gn1, gn3]),
        data: 4,
      }),
    );
    let pn4 = trans.new_node(NodeType::PairNode(PairNode { the_other: pn3 }));
    let tn5 = trans
      .new_node(NodeType::TreeNode(TreeNode { children: HashSet::new(), father: tn2 }));
    let dn5 = trans.alloc_node();
    let bn2 =
      trans.new_node(NodeType::BoxNode(BoxNode { inside: BTreeSet::from_iter([dn5]) }));
    let dn4 = trans.new_node(NodeType::DataNode(DataNode { parent: bn, data: 4 }));
    trans.fill_back_node(
      dn5,
      NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 5 }),
    );

    graph.commit(trans);
    {
      let node = GraphNode::get_by_type(&graph, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3, gn4]));
      assert_eq!(node.data, 2);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn4]));
      assert_eq!(node.data, 3);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn4).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 4);
    }
    {
      let node = PairNode::get_by_type(&graph, pn1).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = PairNode::get_by_type(&graph, pn2).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = PairNode::get_by_type(&graph, pn3).unwrap();
      assert_eq!(node.the_other, pn4);
    }
    {
      let node = PairNode::get_by_type(&graph, pn4).unwrap();
      assert_eq!(node.the_other, pn3);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4, tn5]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn5).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn2);
    }
    {
      let node = BoxNode::get_by_type(&graph, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3, dn4, tn1, gn1, pn3]));
    }
    {
      let node = BoxNode::get_by_type(&graph, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn5]));
    }
    {
      let node = DataNode::get_by_type(&graph, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = DataNode::get_by_type(&graph, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = DataNode::get_by_type(&graph, dn3).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 3);
    }
    {
      let node = DataNode::get_by_type(&graph, dn4).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 4);
    }
    {
      let node = DataNode::get_by_type(&graph, dn5).unwrap();
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

    let gn4 = trans.alloc_node();
    GraphNode::mut_by_type(&mut trans, gn1, |x| {
      x.tos.remove(&gn3);
      x.froms.remove(&gn3);
    });
    GraphNode::mut_by_type(&mut trans, gn2, |x| {
      x.tos.remove(&gn2);
    });
    GraphNode::update_by_type(&mut trans, gn3, |x| GraphNode {
      tos: BTreeSet::from_iter([gn2]),
      froms: BTreeSet::from_iter([gn4]),
      ..x
    });
    trans.fill_back_node(
      gn4,
      NodeType::GraphNode(GraphNode {
        tos: BTreeSet::from_iter([gn2, gn4]),
        froms: BTreeSet::from_iter([gn1, gn3]),
        data: 4,
      }),
    );

    let pn4 = PairNode::new_by_type(&mut trans, PairNode { the_other: pn2 });
    PairNode::mut_by_type(&mut trans, pn1, |x| {
      x.the_other = NodeIndex::empty();
    });
    PairNode::mut_by_type(&mut trans, pn2, |x| {
      x.the_other = NodeIndex::empty();
    });

    let tn5 = trans.new_node(NodeType::TreeNode(TreeNode {
      children: HashSet::from_iter([tn2, tn4]),
      father: tn1,
    }));
    TreeNode::mut_by_type(&mut trans, tn2, |x| {
      x.father = tn5;
      x.children.remove(&tn4);
    });
    TreeNode::mut_by_type(&mut trans, tn4, |x| {
      x.father = tn5;
    });

    let dn4 = trans
      .new_node(NodeType::DataNode(DataNode { parent: NodeIndex::empty(), data: 4 }));
    BoxNode::mut_by_type(&mut trans, bn, |x| {
      x.inside.remove(&dn2);
      x.inside.remove(&dn3);
      x.inside.remove(&gn1);
      x.inside.insert(dn4);
    });
    let bn2 = trans.new_node(NodeType::BoxNode(BoxNode {
      inside: BTreeSet::from_iter([dn2, dn3, gn1]),
    }));
    DataNode::update_by_type(&mut trans, dn2, |x| DataNode { parent: bn2, ..x });
    DataNode::mut_by_type(&mut trans, dn3, |x| {
      x.parent = NodeIndex::empty();
    });
    let dn5 = trans.new_node(NodeType::DataNode(DataNode { parent: bn2, data: 5 }));

    graph.commit(trans);
    {
      let node = GraphNode::get_by_type(&graph, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 1);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 2);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn4]));
      assert_eq!(node.data, 3);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn4).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2, gn3, gn4]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn3, gn4]));
      assert_eq!(node.data, 4);
    }
    {
      let node = PairNode::get_by_type(&graph, pn1).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = PairNode::get_by_type(&graph, pn2).unwrap();
      assert_eq!(node.the_other, pn4);
    }
    {
      let node = PairNode::get_by_type(&graph, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = PairNode::get_by_type(&graph, pn4).unwrap();
      assert_eq!(node.the_other, pn2);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn5, tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn2).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn5);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn5);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn5).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn2, tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = BoxNode::get_by_type(&graph, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn4, tn1, pn3]));
    }
    {
      let node = BoxNode::get_by_type(&graph, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn2, dn3, dn5, gn1]));
    }
    {
      let node = DataNode::get_by_type(&graph, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = DataNode::get_by_type(&graph, dn2).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 2);
    }
    {
      let node = DataNode::get_by_type(&graph, dn3).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 3);
    }
    {
      let node = DataNode::get_by_type(&graph, dn4).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 4);
    }
    {
      let node = DataNode::get_by_type(&graph, dn5).unwrap();
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
    trans.remove_node(gn3);
    trans.remove_node(pn2);
    trans.remove_node(tn2);
    trans.remove_node(dn3);

    graph.commit(trans);

    {
      let node = GraphNode::get_by_type(&graph, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 1);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn2]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2]));
      assert_eq!(node.data, 2);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn3);
      assert!(node.is_none());
    }
    {
      let node = PairNode::get_by_type(&graph, pn1).unwrap();
      assert_eq!(node.the_other, NodeIndex::empty());
    }
    {
      let node = PairNode::get_by_type(&graph, pn2);
      assert!(node.is_none());
    }
    {
      let node = PairNode::get_by_type(&graph, pn3).unwrap();
      assert!(node.the_other.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn2);
      assert!(node.is_none());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn3).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, NodeIndex::empty());
    }
    {
      let node = BoxNode::get_by_type(&graph, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, tn1, gn1, pn3]));
    }
    {
      let node = DataNode::get_by_type(&graph, dn1).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 1);
    }
    {
      let node = DataNode::get_by_type(&graph, dn2).unwrap();
      assert_eq!(node.parent, bn);
      assert_eq!(node.data, 2);
    }
    {
      let node = DataNode::get_by_type(&graph, dn3);
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

    trans.redirect_node(gn2, gn3);
    trans.redirect_node(pn2, pn3);
    trans.redirect_node(tn2, tn3);
    let bn2 = trans.new_node(NodeType::BoxNode(BoxNode { inside: BTreeSet::new() }));
    trans.redirect_node(bn, bn2);

    graph.commit(trans);

    {
      let node = GraphNode::get_by_type(&graph, gn1).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.data, 1);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn2).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![]));
      assert_eq!(node.data, 2);
    }
    {
      let node = GraphNode::get_by_type(&graph, gn3).unwrap();
      assert_eq!(node.tos, BTreeSet::from_iter(vec![gn1, gn3]));
      assert_eq!(node.froms, BTreeSet::from_iter(vec![gn1, gn2, gn3]));
      assert_eq!(node.data, 3);
    }
    {
      let node = PairNode::get_by_type(&graph, pn1).unwrap();
      assert_eq!(node.the_other, pn3);
    }
    {
      let node = PairNode::get_by_type(&graph, pn2).unwrap();
      assert_eq!(node.the_other, NodeIndex::empty());
    }
    {
      let node = PairNode::get_by_type(&graph, pn3).unwrap();
      assert_eq!(node.the_other, pn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn1).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn3]));
      assert!(node.father.is_empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn2).unwrap();
      assert_eq!(node.children, HashSet::from_iter([]));
      assert_eq!(node.father, NodeIndex::empty());
    }
    {
      let node = TreeNode::get_by_type(&graph, tn3).unwrap();
      assert_eq!(node.children, HashSet::from_iter([tn4]));
      assert_eq!(node.father, tn1);
    }
    {
      let node = TreeNode::get_by_type(&graph, tn4).unwrap();
      assert!(node.children.is_empty());
      assert_eq!(node.father, tn3);
    }
    {
      let node = BoxNode::get_by_type(&graph, bn).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([tn1, gn1, pn3]));
    }
    {
      let node = BoxNode::get_by_type(&graph, bn2).unwrap();
      assert_eq!(node.inside, BTreeSet::from_iter([dn1, dn2, dn3]));
    }
    {
      let node = DataNode::get_by_type(&graph, dn1).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 1);
    }
    {
      let node = DataNode::get_by_type(&graph, dn2).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 2);
    }
    {
      let node = DataNode::get_by_type(&graph, dn3).unwrap();
      assert_eq!(node.parent, bn2);
      assert_eq!(node.data, 3);
    }

    graph.check_backlinks();
  }
}
