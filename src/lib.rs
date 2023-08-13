use visible::StructFields;

use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::sync::Arc;

use std::collections::{hash_set, HashSet};

mod arena;
pub use arena::*;

mod display;
pub use display::*;

mod iter;
pub use iter::*;

#[StructFields(pub)]
#[derive(Debug, Clone)]
pub struct Node<NDataT> {
    idx: NodeIndex,
    data: NDataT,
    in_edges: HashSet<EdgeIndex>,
    out_edges: HashSet<EdgeIndex>,
}

#[StructFields(pub)]
#[derive(Debug, Clone)]
pub struct Edge<EDataT> {
    idx: EdgeIndex,
    data: EDataT,
    from: NodeIndex,
    to: NodeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(usize);

impl ArenaIndex for NodeIndex {
    fn new(id: usize) -> Self {
        NodeIndex(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeIndex(usize);
impl ArenaIndex for EdgeIndex {
    fn new(id: usize) -> Self {
        EdgeIndex(id)
    }
}

#[derive(Debug, Clone)]
struct Graph<NDataT, EDataT> {
    nodes: Arena<Node<NDataT>, NodeIndex>,
    edges: Arena<Edge<EDataT>, EdgeIndex>,
}

#[derive(Debug, Clone)]
pub struct TGraph<NDataT, EDataT> {
    context: Context,
    graph: RefCell<Graph<NDataT, EDataT>>,
}

impl<NDataT, EDataT> TGraph<NDataT, EDataT> {
    pub fn new(context: Context) -> TGraph<NDataT, EDataT> {
        TGraph {
            graph: RefCell::new(Graph::new(&context)),
            context: context,
        }
    }
    pub fn transaction(&self) -> Transaction<NDataT, EDataT> {
        Transaction::new(self, &self.context)
    }
}

impl<NDataT, EDataT> Graph<NDataT, EDataT> {
    pub fn new(context: &Context) -> Graph<NDataT, EDataT> {
        Graph {
            nodes: Arena::new(&context.node_dist),
            edges: Arena::new(&context.edge_dist),
        }
    }

    fn remove_node(&mut self, n: NodeIndex) {
        self.nodes.remove(n);
    }

    fn remove_edge(&mut self, e: EdgeIndex) {
        self.edges.remove(e);
    }

    fn modify_node<F>(&mut self, i: NodeIndex, f: F)
    where
        F: FnOnce(&mut NDataT),
    {
        f(&mut self.nodes.get_mut(i).unwrap().data);
    }
    fn modify_edge<F>(&mut self, i: EdgeIndex, f: F)
    where
        F: FnOnce(&mut EDataT),
    {
        f(&mut self.edges.get_mut(i).unwrap().data);
    }

    fn update_node<F>(&mut self, i: NodeIndex, f: F)
    where
        F: FnOnce(NDataT) -> NDataT,
    {
        self.nodes.update_with(i, |x| Node {
            data: f(x.data),
            ..x
        });
    }

    fn update_edge<F>(&mut self, i: EdgeIndex, f: F)
    where
        F: FnOnce(EDataT) -> EDataT,
    {
        self.edges.update_with(i, |x| Edge {
            data: f(x.data),
            ..x
        });
    }
}

pub struct Transaction<'tg, 'a, NDataT, EDataT> {
    tgraph: &'tg TGraph<NDataT, EDataT>,
    committed: bool,
    manual: bool,
    inc_nodes: Arena<Node<NDataT>, NodeIndex>,
    inc_edges: Arena<Edge<EDataT>, EdgeIndex>,
    dec_nodes: Vec<NodeIndex>,
    dec_edges: Vec<EdgeIndex>,
    mut_nodes: Vec<(NodeIndex, Box<dyn FnOnce(&mut NDataT) + 'a>)>,
    mut_edges: Vec<(EdgeIndex, Box<dyn FnOnce(&mut EDataT) + 'a>)>,
    update_nodes: Vec<(NodeIndex, Box<dyn FnOnce(NDataT) -> NDataT + 'a>)>,
    update_edges: Vec<(EdgeIndex, Box<dyn FnOnce(EDataT) -> EDataT + 'a>)>,
}

impl<'tg, 'a, NDataT, EDataT> Transaction<'tg, 'a, NDataT, EDataT> {
    fn new(
        tgraph: &'tg TGraph<NDataT, EDataT>,
        context: &Context,
    ) -> Transaction<'tg, 'a, NDataT, EDataT> {
        Transaction {
            tgraph,
            committed: false,
            manual: false,
            inc_nodes: Arena::new(&context.node_dist),
            inc_edges: Arena::new(&context.edge_dist),
            dec_nodes: Vec::new(),
            dec_edges: Vec::new(),
            mut_nodes: Vec::new(),
            mut_edges: Vec::new(),
            update_nodes: Vec::new(),
            update_edges: Vec::new(),
        }
    }

    pub fn new_node(&mut self, data: NDataT) -> NodeIndex {
        self.inc_nodes.insert_with(|idx| Node {
            idx,
            data,
            in_edges: HashSet::new(),
            out_edges: HashSet::new(),
        })
    }
    pub fn new_edge(&mut self, data: EDataT, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.inc_edges.insert_with(|idx| Edge {
            idx,
            data,
            from,
            to,
        })
    }

    pub fn remove_node(&mut self, node: NodeIndex) {
        if self.inc_nodes.contains(node) {
            self.inc_nodes.remove(node);
        } else {
            self.dec_nodes.push(node);
        }
    }
    pub fn remove_edge(&mut self, edge: EdgeIndex) {
        if self.inc_edges.contains(edge) {
            self.inc_edges.remove(edge);
        } else {
            self.dec_edges.push(edge);
        }
    }

    pub fn mut_node<F>(&mut self, node: NodeIndex, func: F)
    where
        F: FnOnce(&mut NDataT) + 'a,
    {
        if self.inc_nodes.contains(node) {
            func(&mut self.inc_nodes.get_mut(node).unwrap().data);
        } else {
            self.mut_nodes.push((node, Box::new(func)));
        }
    }

    pub fn mut_edge<F>(&mut self, edge: EdgeIndex, func: F)
    where
        F: FnOnce(&mut EDataT) + 'a,
    {
        if self.inc_edges.contains(edge) {
            func(&mut self.inc_edges.get_mut(edge).unwrap().data);
        } else {
            self.mut_edges.push((edge, Box::new(func)));
        }
    }

    pub fn update_node<F>(&mut self, node: NodeIndex, func: F)
    where
        F: FnOnce(NDataT) -> NDataT + 'a,
    {
        if self.inc_nodes.contains(node) {
            self.inc_nodes.update_with(node, |x| Node {
                data: func(x.data),
                ..x
            });
        } else {
            self.update_nodes.push((node, Box::new(func)));
        }
    }

    pub fn update_edge<F>(&mut self, edge: EdgeIndex, func: F)
    where
        F: FnOnce(EDataT) -> EDataT + 'a,
    {
        if self.inc_edges.contains(edge) {
            self.inc_edges.update_with(edge, |x| Edge {
                data: func(x.data),
                ..x
            });
        } else {
            self.update_edges.push((edge, Box::new(func)));
        }
    }

    pub fn commit(&mut self) {
        if self.committed {
            return;
        }
        self.committed = true;
        let mut graph = self.tgraph.graph.borrow_mut();

        self.addback(&mut graph);
        while let Some((i, f)) = self.mut_nodes.pop() {
            graph.modify_node(i, f)
        }
        while let Some((i, f)) = self.mut_edges.pop() {
            graph.modify_edge(i, f)
        }
        while let Some((i, f)) = self.update_nodes.pop() {
            graph.update_node(i, f)
        }
        while let Some((i, f)) = self.update_edges.pop() {
            graph.update_edge(i, f)
        }
        for n in &self.dec_nodes {
            graph.remove_node(*n);
        }
        for e in &self.dec_edges {
            graph.remove_edge(*e);
        }
    }

    pub fn giveup(&mut self) {
        self.committed = true;
    }

    pub fn set_manual(&mut self) {
        self.manual = true;
    }
    pub fn set_auto(&mut self) {
        self.manual = false;
    }

    fn addback(&mut self, graph: &mut RefMut<Graph<NDataT, EDataT>>) {
        graph.nodes.merge(&mut self.inc_nodes);
        for (idx, e) in &self.inc_edges {
            graph.nodes[e.from].out_edges.insert(*idx);
            graph.nodes[e.to].in_edges.insert(*idx);
        }
        graph.edges.merge(&mut self.inc_edges);
    }
}

impl<'tg, 'a, NDataT, EDataT> Drop for Transaction<'tg, 'a, NDataT, EDataT> {
    fn drop(&mut self) {
        if !self.manual {
            self.commit();
        }
    }
}

#[derive(Debug)]
pub struct Context {
    node_dist: Arc<IdDistributer>,
    edge_dist: Arc<IdDistributer>,
}
impl Context {
    pub fn new() -> Context {
        Context {
            node_dist: Arc::new(IdDistributer::new()),
            edge_dist: Arc::new(IdDistributer::new()),
        }
    }
}
impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            node_dist: Arc::clone(&self.node_dist),
            edge_dist: Arc::clone(&self.edge_dist),
        }
    }
}
