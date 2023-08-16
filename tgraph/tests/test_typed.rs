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
    }

    #[derive(TypedNode)]
    struct NodeB {
        a: NodeIndex,
    }

    #[derive(NodeEnum)]
    enum NodeType {
        A(NodeA),
        B(NodeB),
        Edge(Edge<i32>),
    }

    #[test]
    fn can_compile() {
        let context = Context::new();
        let mut graph = Graph::<NodeType>::new(&context);
        for (idx, n) in NodeA::iter_by_type(&graph) {}
        for (idx, n) in NodeB::iter_by_type(&graph) {}
        for (idx, n) in Edge::iter_by_type(&graph) {}
    }
}
