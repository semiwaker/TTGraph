#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod test_bidirectional {
  use std::collections::{BTreeSet, HashSet};

  use tgraph::typed_graph::*;
  use tgraph_macros::*;

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

    // gn1 -> gn2, gn3
    // gn2 -> gn2
    // gn3 -> gn1, gn2
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

    // pn1 <-> pn2, pn3 dangling
    trans.fill_back_node(
      pn1,
      NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }),
    );
    trans.fill_back_node(pn2, NodeType::PairNode(PairNode { the_other: pn1 }));
    trans.fill_back_node(
      pn3,
      NodeType::PairNode(PairNode { the_other: NodeIndex::empty() }),
    );

    // tn1 -> tn2, tn3
    // tn2 -> tn4

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

    // bn -> dn1, dn2, dn3, tn1, gn1, pn3
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
}
