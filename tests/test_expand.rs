#![allow(dead_code)]

use ttgraph::*;
#[derive(TypedNode, Debug)]
#[phantom_group(def)]
struct OpA {
  x: NodeIndex,
}

node_enum! {
  #[derive(Debug)]
  enum Node{
    OpA(OpA)
  }
}

fn f() {
  discriminant!(crate::Node::OpA);
}
