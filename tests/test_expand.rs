use ttgraph::*;
#[derive(TypedNode, Debug)]
#[phantom_group(def)]
struct OpA { 
  x: NodeIndex,
}

node_enum!{
  #[derive(Debug)]
  enum Node{
    OpA(OpA)
  }
}