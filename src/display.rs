use super::*;
use std::fmt::Display;

impl<NDataT: Debug> Display for Node<NDataT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ idx: {}, data: {:?}, in_edges: ",
            self.idx.id, self.data
        )?;
        let mut dl = f.debug_list();
        for i in &self.in_edges {
            dl.entry(&i.id);
        }
        dl.finish()?;
        write!(f, ", out_edges: ")?;
        let mut dl = f.debug_list();
        for i in &self.out_edges {
            dl.entry(&i.id);
        }
        dl.finish()?;
        write!(f, "}}")?;
        std::fmt::Result::Ok(())
    }
}

impl<EDataT: Debug> Display for Edge<EDataT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Edge")
            .field("idx", &self.idx.id)
            .field("data", &self.data)
            .field("from", &self.from.id)
            .field("to", &self.to.id)
            .finish()
    }
}

impl<NDataT: Debug, EDataT: Debug> Display for Graph<NDataT, EDataT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n  nodes: [\n")?;
        for (_, n) in &self.nodes {
            write!(f, "     {},\n", n)?;
        }
        write!(f, "  ],\n  edges: [\n")?;
        for (_, n) in &self.edges {
            write!(f, "     {},\n", n)?;
        }
        write!(f, "  ]\n}}")?;
        std::fmt::Result::Ok(())
    }
}

impl<NDataT: Debug, EDataT: Debug> Display for TGraph<NDataT, EDataT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.graph.borrow(), f)
    }
}
