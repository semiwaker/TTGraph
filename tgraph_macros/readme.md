# TGraph macro

## TypedNode

```rust
#[TypedNode]
struct MyNode{
  x: NodeIndex, // Direct link
  ys: HashSet<NodeIndex> // Set link
  ys2: BTreeSet<NodeIndex> // Set link
  // z: NIEWrap<NIEnum> // Enum link
  u: Vec<NodeIndex> // Vec link
  other_data: usize // other data, can have any type
}
```

Generics and visibility is taken into consideration.
*Though, if the generic is NodeIndex, it is not detected.*

### Link Reflection

**Definition:**

+ Link: a struct member with `NodeIndex`
+ Source: a link with more data to specify the exact connection. Currently the only difference is `Vec<NodeIndex>`, a source have a usize to indicate the position of the vec.

Generated Link (names are camel-cased):
```rust
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#vis enum MyNodeLinkMirror{
  X,
  Ys,
  Ys2,
  // Z,
  U
}
```

Generated Source (names are camel-cased):

```rust
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#vis enum MyNodeSource{
  X,
  Ys,
  Ys2,
  // Z,
  U(usize)
}

impl MyNodeSource{
  fn to_link_mirror(&self) -> MyNodeLinkMirror {
    // ...
  }
}
```

Source Iterator:

```rust
struct MyNodeSourceIterator{
  sources: Vec<(NodeIndex, NodeNameSource)>,
  cur: usize
}
impl tgraph::typed_graph::SourceIterator<NodeName> for NodeNameIterator{ 
  // ...
}

impl std::iter::Iterator for MyNodeIterator { }
```

### TypedNode trait

```rust
impl tgraph::typed_graph::TypedNode for MyNode {
  type Source = MyNodeSource;
  type LinkMirror = MyNodeLinkMirror;
  type Iter = MyNodeSourceIterator;
  fn iter_source(&self) -> Self::Iter { }
  fn iter_link(&self, link: Self::LinkMirror) -> Box<dyn Iterator<Item=NodeIndex>>;
  fn modify_link(&mut self, source: Self::Source, old_idx:NodeIndex, new_idx: NodeIndex) {
    match source{
      Self::Source::X => self.x = new_idx, // Direct link
      Self::Source::Ys => { // Set link
        self.ys.remove(old_idx);
        self.ys.insert(new_idx);
      }
      Self::Source::Ys2 => { // Set link
        self.ys2.remove(old_idx);
        self.ys2.insert(new_idx);
      }
      // MyNodeSource::Z => { // enum link
      //     tgraph::typed_graph::IndexEnum::modify(&mut self.z.value, new_idx);
      // }
      Self::Source::U(idx) => { // vec link
        self.u[idx] = new_idx;
      }
    }
  }
  fn add_link(&mut self, link: Self::LinkMirror, target: tgraph::typed_graph::NodeIndex) {
    match link{
      Self::Link::X => {
        if self.x.is_empty() {
          self.x = target;
        } else if self.x != target {
          panic!("Add link on an existing point link!");
        }
      },
      Self::Link::Ys => {
        self.ys.insert(target);
      },
      Self::Link::Ys2 => {
        self.ys2.insert(target);
      },
      Self::Link::U => {
        panic!("Add link on Vec<NodeIndex> is not supported!");
      },
    }
  }
  fn remove_link(&mut self, link: Self::LinkMirror, target: tgraph::typed_graph:: NodeIndex) {
    match link{
      Self::Link::X => {
        if self.x.is_empty() {
          panic!("Remove link on an empty point link!");
        } else {
          self.x = tgraph::typed_graph::NodeIndex::empty();
        }
      },
      Self::Link::Ys => {
        if !self.ys.remove(target) {
          panic!("Remove an non-existing link!");
        }
      },
      Self::Link::Ys2 => {
        if !self.ys.remove(target) {
          panic!("Remove an non-existing link!");
        }
      },
      Self::Link::U => {
        panic!("Remove link on Vec<NodeIndex> is not supported!");
      },
    }
  }

  fn link_types() -> &'static [LinkType] {
    &[LinkType::Point, LinkType::HSet, LinkType::BSet, LinkType::Vec]
  }
  fn link_mirrors() -> &'static [Self::LinkMirror] {
    &[MyNodeLinkMirror::X, MyNodeLinkMirror::Ys, MyNodeLinkMirror::Ys2, MyNodeLinkMirrot::U]
  }
  fn link_names() -> &'static [&'static str] {
    &["x", "ys", "ys2", "u"]
  }

  // fn data_types() -> &'static [std::any::TypeId] {
  //     &[std::any::TypeId::of::<usize>]
  // }
  fn data_names() -> &'static [&'static str] {
    &["other_data"]
  }
  fn data_ref_by_name<TGDataRefT: Any>(&self, name: &'static str) -> Option<&TGDataRefT> {
    match name {
      "other_data": std::Any::downcast_ref::<TGDataRefT>(self.other_data),
      _ => None
    } 
  }
}
```

<!-- ## NodeIndexEnum

Deprecated

```rust
#[derive(IndexEnum)]
enum NIEnum {
    A(NodeIndex),
    B(NodeIndex),
}
```

### NodeIndexEnum trait

Assume modify does not change the variant of the index enum

```rust
impl tgraph::typed_graph::IndexEnum for NIEnum {
    fn modify(&mut self, new_idx: NodeIndex) {
        *self = match self {
            NIEnum::A(idx) => NIEnum::A(new_idx),
            NIEnum::B(idx) => NIEnum::B(new_idx),
        };
    }
    fn index(&self) -> NodeIndex {
        match self {
            NIEnum::A(idx) => idx.clone(),
            NIEnum::B(idx) => idx.clone(),
        }
    }
}
``` -->

## NodeEnum

```rust
#[derive(NodeEnum)]
enum MyNodeEnum {
    A(NodeA),
    B(NodeB),
}
```

Variants should have exactly the form `Name(Type)`


### Aggregated mirrors for all types of nodes in the graph

```rust
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
enum MyNodeEnumSourceEnum {
  A(<NodeA as TypedNode>::Source),
  B(<NodeB as TypedNode>::Source),
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
enum MyNodeEnumLinkMirrorEnum {
  A(<NodeA as TypedNode>::LinkMirror),
  B(<NodeB as TypedNode>::LinkMirror),
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
enum MyNodeEnumNodeTypeMirror {
  A(<NodeA as TypedNode>::LinkMirror),
  B(<NodeB as TypedNode>::LinkMirror),
}
```

### NodeEnum trait
```rust
impl NodeEnum for MyNodeEnum {
    type SourceEnum = MyNodeEnumSourceEnum;
    type LinkMirrorEnum = MyNodeEnumLinkMirrorEnum;
    type NodeTypeMirror = MyNodeEnumNodeTypeMirror;
    fn iter_source(&self) -> Box<dyn Iterator<Item = (NodeIndex, Self::SourceEnum)>> {
      match self {
        Self::A(x) => {
          Box::new(
            <NodeA as TypedNode>::iter_source(&x).map(|(idx, src)| (idx, Self::SourceEnum::A(src))),
          )
        }
        Self::B(x) => {
          Box::new(
            <NodeB as TypedNode>::iter_source(&x).map(|(idx, src)| (idx, Self::SourceEnum::B(src))),
          )
        }
      }
    }
    fn modify_link(
      &mut self,
      source: Self::SourceEnum,
      old_idx: NodeIndex,
      new_idx: NodeIndex,
    ) {
      match self {
        Self::A(x) => {
          if let Self::SourceEnum::A(src) = source {
            <NodeA as TypedNode>::modify(x, src, old_idx, new_idx)
          } else {
            panic!("Unmatched node type and source type!")
          }
        }
        Self::B(x) => {
          if let Self::SourceEnum::B(src) = source {
            <NodeB as TypedNode>::modify(x, src, old_idx, new_idx)
          } else {
            panic!("Unmatched node type and source type!"),
          }
        }
      }
    }
    fn add_link(&mut self, link: Self::LinkMirrorEnum, target: NodeIndex) {
      match self {
        Self::A(x) => {
          if let Self::LinkMirrorEnum::A(src) = source {
            <NodeA as TypedNode>::add_link(x, src, target);
          } else {
            // panic!("Unmatched node type and source type!")
          }
        }
        // ...
      }
    }
    fn remove_link(&mut self, link: Self::LinkMirrorEnum, target: NodeIndex) {
      // ...
    }

    fn data_ref_by_name<T: Any>(&self, name: &'static str) -> Option<&T> {
      match self{
        Self::A(x) => <NodeA as TypedNode>::data_ref_by_name(x, name),
        // ...
      }
    }
}
```


### Generated Trait

A helper trait is generated for each type of node to do the `by-type` operations.

```rust
// Generated helper trait
trait TGGenTraitNodeType<'a, IterT> {
    fn iter_by_type(graph: &'a tgraph::typed_graph::Graph<NodeType>) -> IterT;
    fn get_by_type(graph: &'a tgraph::typed_graph::Graph<NodeType>, idx: tgraph::typed_graph::NodeIndex)
       -> Option<&Self>;
}

// Impl For NodeA
impl<'a> TGGenTraitNodeType<'a, IterA<'a>> for NodeA {
    fn iter_by_type(graph: &'a tgraph::typed_graph::Graph<NodeType>) -> TGGenIterA<'a> {
        TGGenIterA { it: graph.iter_nodes() }
    }
    fn get_by_type(graph: &'a tgraph::typed_graph::Graph<NodeType>, idx: tgraph::typed_graph::NodeIndex)
       -> Option<&NodeA>
    {
        // ...
    }
}

// Generated Iterator for A(NodeA)
struct TGGenIterA<'a> {
    it: tgraph::typed_graph::Iter<'a, NodeType>,
}
impl<'a> std::iter::Iterator for TGGenIterA<'a> {
    type Item = (NodeIndex, &'a NodeA);
    fn next(&mut self) -> Option<Self::Item> {
        self.it
            .next()
            .and_then(|(idx, node)| {
                if let NodeType::A(x) = &node { Some((*idx, x)) } else { None }
            })
        // Iterate and filter
    }
}
impl<'a> std::iter::FusedIterator for IterA<'a> {}
```