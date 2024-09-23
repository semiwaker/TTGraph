use ttgraph::*;
#[derive(TypedNode, Debug)]
#[phantom_group(def)]
struct OpA { 
  #[group(y)]
  x: NodeIndex,
}

node_enum!{
  enum Node{
    OpA(OpA)
  }
  link_type!{
    OpA.def: OpA
  }
}