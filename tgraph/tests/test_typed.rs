#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

#[cfg(test)]
mod tests_typed {
    use std::collections::HashSet;

    use tgraph::typed_graph::library::*;
    use tgraph::typed_graph::*;
    use tgraph_macros::*;

    #[derive(TypedNode, Debug)]
    struct NodeA {
        to: NodeIndex,
    }

    #[derive(IndexEnum)]
    enum NIEnum {
        A(NodeIndex),
        B(NodeIndex),
    }

    #[derive(TypedNode, Debug)]
    struct NodeB {
        a: NodeIndex,
        x: NodeIndex,
    }

    #[derive(NodeEnum, Debug)]
    enum NodeType {
        A(NodeA),
        B(NodeB),
        Empty(NodeEmpty),
    }

    #[derive(TypedNode, Clone, Debug)]
    struct NodeEmpty {
        x: usize,
    }

    #[test]
    fn can_compile() {
        let context = Context::new();
        let mut graph = Graph::<NodeType>::new(&context);
        let mut trans = Transaction::new(&context);
        let n = trans.new_node(NodeType::Empty(NodeEmpty { x: 0 }));
        graph.commit(trans);
        for (idx, n) in NodeEmpty::iter_by_type(&graph) {
            eprintln!("{:?} {:?}", idx, n);
        }

        let mut trans = Transaction::new(&context);
        let b = trans.alloc_node();
        let a = trans.new_node(NodeType::A(NodeA { to: b }));
        trans.fill_back_node(b, NodeType::B(NodeB { a: a, x: n }));

        graph.commit(trans);
        for (idx, n) in graph.iter_nodes() {
            eprintln!("{:?} {:?}", idx, n);
        }
        for (idx, n) in NodeA::iter_by_type(&graph) {
            eprintln!("{:?} {:?}", idx, n);
        }
        for (idx, n) in NodeB::iter_by_type(&graph) {
            eprintln!("{:?} {:?}", idx, n);
        }
        // for (idx, n) in NodeA::iter_by_type(&graph) {}
        // for (idx, n) in NodeB::iter_by_type(&graph) {
        // let b = NodeB::get_by_type(&graph, idx);
        // }
        // for (idx, n) in Edge::iter_by_type(&graph) {}
    }
}
