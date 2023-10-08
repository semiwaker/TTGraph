#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod tests_typed {
    use std::collections::HashSet;

    use tgraph::typed_graph::library::*;
    use tgraph::typed_graph::*;
    use tgraph_macros::*;

    #[derive(TypedNode)]
    struct NodeA {
        to: NodeIndex,
        from: HashSet<NodeIndex>,
        values: Vec<NodeIndex>,
    }

    #[derive(IndexEnum)]
    enum NIEnum {
        A(NodeIndex),
        B(NodeIndex),
    }

    #[derive(TypedNode)]
    struct NodeB {
        a: NodeIndex,
        x: NIEWrap<NIEnum>,
    }

    #[derive(NodeEnum)]
    enum NodeType {
        A(NodeA),
        B(NodeB),
        Edge(Edge<i32>),
    }

    #[derive(TypedNode, Clone)]
    struct NodeEmpty {
        x: usize,
    }

    #[test]
    fn can_compile() {
        let context = Context::new();
        let mut graph = Graph::<NodeType>::new(&context);
        for (idx, n) in NodeA::iter_by_type(&graph) {}
        for (idx, n) in NodeB::iter_by_type(&graph) {
            let b = NodeB::get_by_type(&graph, idx);
        }
        for (idx, n) in Edge::iter_by_type(&graph) {}
    }
}
