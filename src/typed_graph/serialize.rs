//! Helper structs and functions to serialize and deserialize a [`Graph`]
//! [`GraphSerializer`] is a helper struct which only contains the nessesary data of the [`Graph`], dropping all other data that can be reconstructed.
//! Use [`from::<Graph>()`](GraphSerializer::from) or directly serialize the graph, and use [`deserialize_graph()`] to reconstruct the context and graph together.
//! # Notes:
//! + [`Context`] is not serializable or deserializable due to it contains [`Arc`] and atomic counters. [`deserialize_graph()`] constructs a new [`Context`] that is compatible instead.
//! + If there are multiple deserialized graphs using the same context before they are serialized, use [`switch_context()`](Graph::switch_context) to merge the newly created contexts.
//! + [`Transaction`] is also not serializable or deserializable, due to it contains closures. Also, it is not reasonable to serialize uncommitted transactions.
//! # Example
//! ```rust
//! use ttgraph::{*, serialize::*};
//! use serde::{Serialize, Deserialize};
//! #[derive(TypedNode, Serialize, Deserialize)]
//! struct NodeA{
//!   data: usize,
//! }
//! node_enum!{
//!   #[derive(Serialize, Deserialize)]
//!   enum Node{
//!     A(NodeA)
//!   }
//! }
//!
//! # fn main() {
//! let ctx = Context::new();
//! let mut graph = Graph::<Node>::new(&ctx);
//! let mut trans = Transaction::new(&ctx);
//! let idx = trans.insert(Node::A(NodeA{
//!   data: 1
//! }));
//! graph.commit(trans);
//!
//! let serialized = serde_json::to_string(&graph).unwrap();
//! let deserialized: GraphSerializer<Node> = serde_json::from_str(&serialized).unwrap();
//! let (ctx2, graph2) = deserialize_graph(deserialized);
//! # }
//! ```

use super::*;
use serde::{
  de::Deserialize,
  ser::{Serialize, SerializeSeq, SerializeStruct},
};

/// Helper struct to serialzie and deserialzie a [`Graph`]
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphSerializer<NodeT>
where
  NodeT: NodeEnum,
{
  ctx_id: Uuid,
  nodes: Vec<(NodeIndex, NodeT)>,
}

impl<NodeT, Arena> From<Graph<NodeT, Arena>> for GraphSerializer<NodeT>
where
  NodeT: NodeEnum,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>,
{
  fn from(value: Graph<NodeT, Arena>) -> GraphSerializer<NodeT> {
    GraphSerializer {
      ctx_id: value.ctx_id,
      nodes: Vec::from_iter(value.into_iter()),
    }
  }
}

struct NodeSerialize<'a, NodeT, Arena>(Arena::Iter<'a>)
where
  NodeT: NodeEnum + Serialize + 'static,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>;

impl<'a, NodeT, Arena> Serialize for NodeSerialize<'a, NodeT, Arena>
where
  NodeT: NodeEnum + Serialize + 'static,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut nodes = serializer.serialize_seq(Some(self.0.len()))?;
    for (i, n) in self.0.clone() {
      nodes.serialize_element(&(i, n))?;
    }
    nodes.end()
  }
}

impl<NodeT, Arena> Serialize for Graph<NodeT, Arena>
where
  NodeT: NodeEnum + Serialize + 'static,
  Arena: CateArena<V = NodeT, D = NodeT::Discriminant>,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut graph = serializer.serialize_struct("Graph", 2)?;
    graph.serialize_field("ctx_id", &self.ctx_id)?;
    graph.serialize_field("nodes", &NodeSerialize::<NodeT, Arena>(self.iter()))?;
    graph.end()
  }
}

/// Helper function to deserialize a graph and construct a context for the graph
pub fn deserialize_graph<'de, NodeT: NodeEnum + Deserialize<'de>>(
  input: GraphSerializer<NodeT>,
) -> (Context, Graph<NodeT>) {
  let cnt = input.nodes.iter().map(|(idx, _)| idx.0).max().unwrap_or_else(|| 0);
  let ctx = Context::from_id(input.ctx_id, cnt);
  let graph = Graph::do_deserialize(&ctx, input.nodes);
  (ctx, graph)
}
