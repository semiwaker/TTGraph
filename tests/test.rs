#[cfg(test)]
mod tests {
    use tgraph::*;

    #[test]
    fn can_compile() {
        let graph = TGraph::<i64, i64>::new();
        let mut trans = graph.transaction();
        let n1 = trans.new_node(1);
        let n2 = trans.new_node(2);
        let e1 = trans.new_edge(-1, n1, n2);
        let cvt = trans.commit().unwrap();
        println!("{}", graph);

        let n1 = graph.convert_node(n1, &cvt);
        let n2 = graph.convert_node(n2, &cvt);
        let e1 = graph.convert_edge(e1, &cvt);
        {
            let mut _trans = graph.transaction();
            let n3 = _trans.new_node(3);
            let n4 = _trans.new_node(4);
            _trans.new_edge(-2, n1, n3);
            _trans.new_edge(-3, n4, n2);
            _trans.new_edge(-4, n4, n3);
            _trans.remove_edge(e1);
        }
        println!("{}", graph);
        {
            let mut _trans = graph.transaction();
            _trans.remove_node(n1);
            _trans.mut_node(n2, |x| *x = *x * 5);
        }
        println!("{}", graph);

        let mut trans = graph.transaction();
        let n5 = trans.new_node(5);
        trans.new_edge(-5, n2, n5);
        let cvt = trans.commit().unwrap();
        let n5 = graph.convert_node(n5, &cvt);
        {
            let mut _trans = graph.transaction();
            _trans.update_node(n5, |x| x * 3);
        }
        println!("{}", graph);
    }
}
