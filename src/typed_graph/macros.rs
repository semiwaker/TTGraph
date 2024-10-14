// use super::*;

pub use paste::paste;

/// Get a node from the graph, assume it is $var variant of the NodeEnum. Returns `Option<&NodeType>`
///
/// # Example
/// ```
/// use ttgraph::*;
///
/// #[derive(TypedNode)]
/// struct NodeA{
///   a: usize
/// }
///
/// node_enum!{
///   enum MyNodeEnum{
///     A(NodeA)
///   }
/// }
///
/// # fn main() {
/// let ctx = Context::new();
/// let mut graph = Graph::<MyNodeEnum>::new(&ctx);
/// let mut trans = Transaction::new(&ctx);
///
/// let x = trans.insert(MyNodeEnum::A(NodeA{ a: 1 }));
/// graph.commit(trans);
///
/// // a: Option<&NodeA>
/// let a = get_node!(graph, MyNodeEnum::A, x);
/// assert!(a.is_some());
/// assert_eq!(a.unwrap().a, 1);
/// # }
/// ```
#[macro_export]
macro_rules! get_node {
  ($graph: expr, $var: path, $idx: expr) => {
    if let Some($var(x)) = $graph.get($idx) {
      Some(x)
    } else {
      None
    }
  };
}

/// Iterate a type of nodes from the graph, assume they are $var variant of the NodeEnum. `Returns impl Iterator<Item = (NodeIndex, &NodeType)>`
///
/// # Example
/// ```
/// use ttgraph::*;
///
/// #[derive(TypedNode)]
/// struct NodeA{
///   a: usize
/// }
/// #[derive(TypedNode)]
/// struct NodeB{
///   b: usize
/// }
///
/// node_enum!{
///   enum MyNodeEnum{
///     A(NodeA),
///     B(NodeB),
///   }
/// }
///
/// # fn main() {
/// let ctx = Context::new();
/// let mut graph = Graph::<MyNodeEnum>::new(&ctx);
/// let mut trans = Transaction::new(&ctx);
///
/// trans.insert(MyNodeEnum::A(NodeA{ a: 1 }));
/// trans.insert(MyNodeEnum::A(NodeA{ a: 2 }));
/// trans.insert(MyNodeEnum::B(NodeB{ b: 0 }));
/// graph.commit(trans);
///
/// let iterator = iter_nodes!(graph, crate::MyNodeEnum::A);
/// // a: (NodeIndex, &NodeA)
/// for (i, a) in (1..3).zip(iterator) {
///   assert_eq!(i, a.1.a)
/// }
/// # }
/// ```
#[macro_export]
macro_rules! iter_nodes {
  ($graph: expr, $p: path) => {
    $graph.iter_type($crate::discriminant!($p)).map(|(idx, node)| if let $p(x) = node { (idx, x) } else { panic!() })
  };
}

/// Use the [`mutate`](crate::Transaction::mutate) method of the transaction, assume the node is $var variant of the NodeEnum.
/// Panics if the enum does not match.
///
/// # Overloads:
/// + Legacy: `mut_node!(graph, NodeEnum::Variant, idx, x, { ... })`
/// + Normal closure: `mut_node!(graph, NodeEnum::Variant, idx, |x| { ... })`
/// + Move closure: `mut_node!(graph, NodeEnum::Variant, idx, move |x| { ... })`
///
/// # Example
/// ```
/// use ttgraph::*;
///
/// #[derive(TypedNode)]
/// struct NodeA{
///   a: usize
/// }
///
/// node_enum!{
///   enum MyNodeEnum{
///     A(NodeA)
///   }
/// }
///
/// # fn main() {
/// let ctx = Context::new();
/// let mut graph = Graph::<MyNodeEnum>::new(&ctx);
/// let mut trans = Transaction::new(&ctx);
///
/// let id = trans.insert(MyNodeEnum::A(NodeA{ a: 1 }));
/// graph.commit(trans);
///
/// trans = Transaction::new(&ctx);
/// // It is similar to this closure |x: &mut NodeA| {x.a =2 }
/// mut_node!(trans, MyNodeEnum::A, id, |x| {
///   x.a = 2;
/// });
/// graph.commit(trans);
///
/// let a = get_node!(graph, MyNodeEnum::A, id);
/// assert!(a.is_some());
/// assert_eq!(a.unwrap().a, 2);
/// # }
/// ```
#[macro_export]
macro_rules! mut_node {
  ($transaction: expr, $var: path, $idx: expr, $node: ident, $func: block) => {
    $transaction.mutate($idx, |x| {
      if let $var($node) = x {
        $func;
      } else {
        panic!("Type does not match!");
      }
    })
  };
  ($transaction: expr, $var: path, $idx: expr, | $node: ident | $func: block) => {
    $transaction.mutate($idx, |x| {
      if let $var($node) = x {
        $func;
      } else {
        panic!("Type does not match!");
      }
    })
  };
  ($transaction: expr, $var: path, $idx: expr, move | $node: ident | $func: block) => {
    $transaction.mutate($idx, move |x| {
      if let $var($node) = x {
        $func;
      } else {
        panic!("Type does not match!");
      }
    })
  };
}

/// Use the [`update`](crate::Transaction::update) method of the transaction, assume the node is $var variant of the NodeEnum.
/// Panics if the enum does not match.
///
/// # Overloads:
/// + Legacy: `update_node!(graph, NodeEnum::Variant, idx, x, { ... })`
/// + Normal closure: `update_node!(graph, NodeEnum::Variant, idx, |x| { ... })`
/// + Move closure: `update_node!(graph, NodeEnum::Variant, idx, move |x| { ... })`
///
/// # Example
/// ```
/// use ttgraph::*;
///
/// #[derive(TypedNode)]
/// struct NodeA{
///   a: usize
/// }
///
/// node_enum!{
///   enum MyNodeEnum{
///     A(NodeA)
///   }
/// }
///
/// # fn main() {
/// let ctx = Context::new();
/// let mut graph = Graph::<MyNodeEnum>::new(&ctx);
/// let mut trans = Transaction::new(&ctx);
///
/// let id = trans.insert(MyNodeEnum::A(NodeA{ a: 1 }));
/// graph.commit(trans);
///
/// trans = Transaction::new(&ctx);
/// // It is similar to this closure |x: NodeA| { NodeA{ a: x.a + 1 } }
/// update_node!(trans, MyNodeEnum::A, id, |x| {
///   NodeA {
///     a: x.a + 1,
///   }
/// });
/// graph.commit(trans);
///
/// let a = get_node!(graph, MyNodeEnum::A, id);
/// assert!(a.is_some());
/// assert_eq!(a.unwrap().a, 2);
/// # }
/// ```
#[macro_export]
macro_rules! update_node {
  ($transaction: expr, $var: path, $idx: expr, $node: ident, $func: block) => {
    $transaction.update($idx, |x| {
      if let $var($node) = x {
        $var($func)
      } else {
        panic!("Type does not match!");
      }
    })
  };
  ($transaction: expr, $var: path, $idx: expr, | $node: ident | $func: block) => {
    $transaction.update($idx, |x| {
      if let $var($node) = x {
        $var($func)
      } else {
        panic!("Type does not match!");
      }
    })
  };
  ($transaction: expr, $var: path, $idx: expr, move | $node: ident | $func: block) => {
    $transaction.update($idx, move |x| {
      if let $var($node) = x {
        $var($func)
      } else {
        panic!("Type does not match!");
      }
    })
  };
}

/// Allocate a new [`NodeIndex`](crate::NodeIndex) for a new node of certain type. `alloc_node!(transaction, Node::Type)`
///
/// See [`alloc`](crate::Transaction::alloc) for more information
#[macro_export]
macro_rules! alloc_node {
  ($transaction: expr, $p: path) => {
    $transaction.alloc($crate::discriminant!($p))
  };
}

// /// Get a discriminant for a type
// /// # Example
// /// ```rust
// /// use ttgraph::*;
// ///
// /// #[derive(TypedNode)]
// /// struct NodeA{ }
// ///
// /// #[derive(TypedNode)]
// /// struct NodeB{ }
// ///
// /// node_enum!{
// ///   enum MyNodeEnum{
// ///     A(NodeA),
// ///     B(NodeB),
// ///   }
// /// }
// ///
// /// # fn main() {
// /// println!("{:?}", discriminant!(MyNodeEnum::A)); // A
// /// println!("{:?}", discriminant!(MyNodeEnum::B)); // B
// /// # }
// /// ```
// #[macro_export]
// macro_rules! discriminant {
//   ($($ps:tt)+, $field:tt) => {
//     $crate::macros::paste! {
//       [<$($ps)::* Discriminant>]::$field
//     }
//   };
//   ($($a:tt)::+) => {
//     $crate::discriminant!($($a),*)
//   };
// }
