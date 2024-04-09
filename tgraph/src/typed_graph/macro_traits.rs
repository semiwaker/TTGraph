use visible::StructFields;

use super::*;

/// A helper trait for the graph to trace all links in the nodes
/// # Example
/// ```rust
/// use tgraph::typed_graph::*;
/// use tgraph_macros::*;
/// #[derive(TypedNode)]
/// struct SomeNode {
///   a_link: NodeIndex,
///   another_link: NodeIndex,
///   vec_link: Vec<NodeIndex>,
///   set_link: HashSet<NodeIndex>,
///   bset_link: BTreeSet<NodeIndex>,
///   other_data: usize
///   // ...
/// }
/// ```
pub trait TypedNode: Sized {
  type Source: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  type LinkMirror: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  type Iter: SourceIterator<Self, Source = Self::Source>;

  /// Iterate the links and its source reflection
  fn iter_source(&self) -> Self::Iter;
  /// Iterate the linked node of the specified link
  fn iter_link(&self, link: Self::LinkMirror)
    -> Box<dyn Iterator<Item = NodeIndex> + '_>;
  /// Modify a link by source, return (remove_sucess, add_success)
  fn modify_link(
    &mut self, source: Self::Source, old_idx: NodeIndex, new_idx: NodeIndex,
  ) -> (bool, bool);
  /// Add a link, designed for bidirectional links, return true if the link is actually added
  fn add_link(&mut self, link: Self::LinkMirror, target: NodeIndex) -> bool;
  /// Remove a link, designed for bidirectional links, return true if the link is actually removed
  fn remove_link(&mut self, link: Self::LinkMirror, target: NodeIndex) -> bool;

  /// Get the types of the links
  fn link_types() -> &'static [LinkType];
  /// Get a mirror reflecting the links
  fn link_mirrors() -> &'static [Self::LinkMirror];
  /// Get the name of the links
  fn link_names() -> &'static [&'static str];
  /// Get the links by name
  fn get_link_by_name(
    &self, name: &'static str,
  ) -> Box<dyn Iterator<Item = NodeIndex> + '_>;
  /// Get the links by group name
  fn get_link_by_group(&self, name: &'static str) -> Vec<NodeIndex>;

  // fn data_types() -> [TypeId];
  /// Get the name of the data
  fn data_names() -> &'static [&'static str];
  /// Try to get the reference of a data by name
  fn data_ref_by_name<T: Any>(&self, name: &'static str) -> Option<&T>;

  /// Convert Source to LinkMirror
  fn to_source(input: &Self::LinkMirror) -> Self::Source;

  /// Convert LinkMirror to Source
  fn to_link_mirror(input: &Self::Source) -> Self::LinkMirror;
}

/// A helper trait to declare a enum of all typed nodes
/// # Example
/// ```rust
/// use tgraph::typed_graph::*;
/// use tgraph_macros::*;
/// #[derive(TypedNode)]
/// struct A{
///   a: NodeIndex,
/// }
///
/// #[derive(TypedNode)]
/// struct B{
///   b: NodeIndex,
/// }
///
/// #[derive(NodeEnum)]
/// struct Node{
///   NodeTypeA(A),
///   AnotherNodeType(B),
/// }
/// ```
pub trait NodeEnum {
  type SourceEnum: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  type LinkMirrorEnum: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  type NodeTypeMirror: Copy + Clone + Eq + PartialEq + Debug + Hash + PartialOrd + Ord;
  /// Iterate the links and its source reflection
  fn iter_source(&self) -> Box<dyn Iterator<Item = (NodeIndex, Self::SourceEnum)>>;
  /// Iterate the links and its link reflection
  fn iter_link(
    &self, link: Self::LinkMirrorEnum,
  ) -> Box<dyn Iterator<Item = NodeIndex> + '_>;
  /// Modify a link by source
  fn modify_link(
    &mut self, source: Self::SourceEnum, old_idx: NodeIndex, new_idx: NodeIndex,
  ) -> BidirectionalSideEffect<Self::LinkMirrorEnum>;
  /// Add a link, designed for bidirectional links
  fn add_link(&mut self, link: Self::LinkMirrorEnum, target: NodeIndex) -> bool;
  /// Remove a link, designed for bidirectional links
  fn remove_link(&mut self, link: Self::LinkMirrorEnum, target: NodeIndex) -> bool;
  /// Check if the link and the node is of the same type
  fn check_link(&self, link: Self::LinkMirrorEnum) -> bool;
  /// Get the links by name
  fn get_link_by_name(
    &self, name: &'static str,
  ) -> Box<dyn Iterator<Item = NodeIndex> + '_>;
  /// Get the links by group name
  fn get_link_by_group(&self, name: &'static str) -> Vec<NodeIndex>;

  /// Try to get the reference of a data by name
  fn data_ref_by_name<T: Any>(&self, name: &'static str) -> Option<&T>;

  /// Convert SourceEnum to LinkMirrorEnum
  fn to_source_enum(input: &Self::LinkMirrorEnum) -> Self::SourceEnum;

  /// Convert LinkMirrorEnum to SourceEnum
  fn to_link_mirror_enum(input: &Self::SourceEnum) -> Self::LinkMirrorEnum;

  /// Get the links that are required to be added or removed according to the bidirectional connection
  /// Returns: (Vec<y>, Vec<link>), link of y should connect to this node
  fn get_bidiretional_links(&self) -> BidirectionalLinks<Self::LinkMirrorEnum>;

  /// Get the opposite links of the specified link
  /// Returns: Vec<link>, these types of links are the opposite side of the specified link
  /// Returns nothing if the link type does not match the node type
  fn get_bidiretional_link_mirrors_of(
    &self, link: Self::LinkMirrorEnum,
  ) -> Vec<Self::LinkMirrorEnum>;
}

pub type BidirectionalLinks<LinkMirrorT> = Vec<(Vec<NodeIndex>, Vec<LinkMirrorT>)>;

/// Auto implemented trait for TypedNode to query the graph
/// Only works if all node types are distinct!
pub trait QueryByType<'a>
where Self: 'a
{
  type NodeEnumT: NodeEnum;
  /// Iterate all nodes of this type in the graph
  fn iter_by_type(
    graph: &'a Graph<Self::NodeEnumT>,
  ) -> impl Iterator<Item = (NodeIndex, &Self)>;
  /// Get a node from the graph, assume it is of this type
  fn get_by_type(graph: &'a Graph<Self::NodeEnumT>, idx: NodeIndex) -> Option<&Self>;
}

/// Auto implemented trait for TypedNode to do transaction
/// Only works if all node types are distinct!
pub trait TransactionByType
where Self: Sized
{
  type NodeEnumT: NodeEnum;
  /// Insert a new node, infer the variant by type
  fn new_by_type(trans: &mut Transaction<Self::NodeEnumT>, node: Self) -> NodeIndex;
  /// Use a `fn(&mut Self)` to modify the node, panic if type does not match!
  fn mut_by_type<'a, FuncT>(
    trans: &mut Transaction<'a, Self::NodeEnumT>, idx: NodeIndex, func: FuncT,
  ) where FuncT: FnOnce(&mut Self) + 'a;
  /// Use a `fn(Self) -> Self` to update the node, panic if type does not match!
  fn update_by_type<'a, FuncT>(
    trans: &mut Transaction<'a, Self::NodeEnumT>, idx: NodeIndex, func: FuncT,
  ) where FuncT: FnOnce(Self) -> Self + 'a;
}

#[StructFields(pub)]
#[derive(Clone, Debug)]
pub struct BidirectionalSideEffect<LinkMirrorT> {
  link_mirrors: Vec<LinkMirrorT>,
  add: NodeIndex,
  remove: NodeIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum LinkType {
  Point, // Single NodeIndex
  HSet,  // HashSet
  BSet,  // BTreeSet
  Vec,   // Vec,
         // Enum
}

// IndexEnum is not stable

// pub trait IndexEnum {
//   fn modify(&mut self, new_idx: NodeIndex);
//   fn index(&self) -> NodeIndex;
// }

// pub struct NIEWrap<T: IndexEnum> {
//   pub value: T,
// }
