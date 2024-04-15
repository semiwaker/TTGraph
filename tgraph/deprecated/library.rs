// Common used types of nodes for typed_graph

use tgraph_macros::TypedNode;
use visible::StructFields;

use super::*;

extern crate self as tgraph;

#[derive(TypedNode, Debug)]
#[StructFields(pub)]
pub struct BidirectionalEdge<T: Any> {
  from: NodeIndex,
  to: NodeIndex,
  data: T,
}

#[derive(TypedNode, Debug)]
#[StructFields(pub)]
pub struct Node<T: Any> {
  tos: Vec<NodeIndex>,
  froms: Vec<NodeIndex>,
  data: T,
}

node_enum! {
  #[derive(Debug)]
  enum BidirectionalGraph<NodeDataT: Any, EdgeDataT: Any> {
    Node(Node<NodeDataT>),
    Edge(BidirectionalEdge<EdgeDataT>),
  }
  bidirectional!{
    Node.tos <-> Edge.from,
    Node.froms <-> Edge.to
  }
}

#[derive(TypedNode, Debug)]
#[StructFields(pub)]
pub struct Region<T: Any> {
  data: T,
  nodes: BTreeSet<NodeIndex>,
}
