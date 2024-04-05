// Common used types of nodes for typed_graph

use tgraph_macros::TypedNode;
use visible::StructFields;

use super::*;

extern crate self as tgraph;

#[derive(TypedNode)]
#[StructFields(pub)]
pub struct Edge<T: Any> {
  data: T,
  from: NodeIndex,
  to: NodeIndex,
}

#[derive(TypedNode)]
#[StructFields(pub)]
pub struct Region<T: Any> {
  data: T,
  nodes: BTreeSet<NodeIndex>,
}
