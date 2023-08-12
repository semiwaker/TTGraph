use visible::StructFields;

use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
// extern crate generational_arena;
// use generational_arena::{Arena, Index};

use std::collections::{HashMap, HashSet};

mod arena;
pub use arena::*;

mod display;
pub use display::*;

#[StructFields(pub)]
#[derive(Debug, Clone)]
pub struct Node<NDataT> {
    idx: Index,
    data: NDataT,
    in_edges: HashSet<Index>,
    out_edges: HashSet<Index>,
}

#[StructFields(pub)]
#[derive(Debug, Clone)]
pub struct Edge<EDataT> {
    idx: Index,
    data: EDataT,
    from: Index,
    to: Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewIndex {
    Exist(Index),
    New(Index),
}
impl NewIndex {
    fn convert(self, map: &HashMap<Index, Index>) -> Index {
        match self {
            NewIndex::Exist(idx) => idx,
            NewIndex::New(idx) => map[&idx],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIndex(NewIndex);
impl NodeIndex {
    pub fn exist(i: Index) -> NodeIndex {
        NodeIndex(NewIndex::Exist(i))
    }
    pub fn convert(self, convertor: &IndexConverter) -> Index {
        self.0.convert(&convertor.node_map)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeIndex(NewIndex);
impl EdgeIndex {
    pub fn exist(i: Index) -> NodeIndex {
        NodeIndex(NewIndex::Exist(i))
    }
    pub fn convert(self, convertor: &IndexConverter) -> Index {
        self.0.convert(&convertor.edge_map)
    }
}

#[StructFields(pub)]
pub struct NewNode<NDataT> {
    data: NDataT,
    in_edges: Vec<NewIndex>,
    out_edges: Vec<NewIndex>,
}

#[StructFields(pub)]
pub struct NewEdge<EDataT> {
    data: EDataT,
    from: NewIndex,
    to: NewIndex,
}

#[derive(Debug, Clone)]
struct Graph<NDataT, EDataT> {
    nodes: Arena<Node<NDataT>>,
    edges: Arena<Edge<EDataT>>,
}

#[derive(Debug, Clone)]
pub struct TGraph<NDataT, EDataT> {
    graph: RefCell<Graph<NDataT, EDataT>>,
}

impl<NDataT, EDataT> TGraph<NDataT, EDataT> {
    pub fn new() -> TGraph<NDataT, EDataT> {
        TGraph {
            graph: RefCell::new(Graph::new()),
        }
    }
    pub fn transaction(&self) -> Transaction<NDataT, EDataT> {
        Transaction::new(self)
    }

    pub fn get_node(&self, idx: Index) -> Option<NodeIndex> {
        if self.graph.borrow().nodes.contains(idx) {
            Some(NodeIndex(NewIndex::Exist(idx)))
        } else {
            None
        }
    }
    pub fn get_edge(&self, idx: Index) -> Option<EdgeIndex> {
        if self.graph.borrow().edges.contains(idx) {
            Some(EdgeIndex(NewIndex::Exist(idx)))
        } else {
            None
        }
    }

    pub fn convert_node(&self, idx: NodeIndex, cvt: &IndexConverter) -> NodeIndex {
        self.get_node(idx.convert(cvt)).unwrap()
    }
    pub fn convert_edge(&self, idx: EdgeIndex, cvt: &IndexConverter) -> EdgeIndex {
        self.get_edge(idx.convert(cvt)).unwrap()
    }
}

impl<NDataT, EDataT> Graph<NDataT, EDataT> {
    pub fn new() -> Graph<NDataT, EDataT> {
        Graph {
            nodes: Arena::new(),
            edges: Arena::new(),
        }
    }

    fn remove_node(&mut self, n: Index) {
        self.nodes.remove(n);
    }

    fn remove_edge(&mut self, e: Index) {
        self.edges.remove(e);
    }

    fn modify_node<F>(&mut self, i: Index, f: F)
    where
        F: FnOnce(&mut NDataT),
    {
        f(&mut self.nodes.get_mut(i).unwrap().data);
    }
    fn modify_edge<F>(&mut self, i: Index, f: F)
    where
        F: FnOnce(&mut EDataT),
    {
        f(&mut self.edges.get_mut(i).unwrap().data);
    }

    fn update_node<F>(&mut self, i: Index, f: F)
    where
        F: FnOnce(NDataT) -> NDataT,
    {
        self.nodes.update_with(i, |x| Node {
            data: f(x.data),
            ..x
        });
    }

    fn update_edge<F>(&mut self, i: Index, f: F)
    where
        F: FnOnce(EDataT) -> EDataT,
    {
        self.edges.update_with(i, |x| Edge {
            data: f(x.data),
            ..x
        });
    }
}

fn convert_index(input: Vec<NewIndex>, map: &HashMap<Index, Index>) -> HashSet<Index> {
    input.into_iter().map(|x| x.convert(map)).collect()
}

pub struct Transaction<'tg, 'a, NDataT, EDataT> {
    tgraph: &'tg TGraph<NDataT, EDataT>,
    committed: bool,
    manual: bool,
    inc_nodes: Arena<NewNode<NDataT>>,
    inc_edges: Arena<NewEdge<EDataT>>,
    dec_nodes: Vec<Index>,
    dec_edges: Vec<Index>,
    mut_nodes: Vec<(Index, Box<dyn FnOnce(&mut NDataT) + 'a>)>,
    mut_edges: Vec<(Index, Box<dyn FnOnce(&mut EDataT) + 'a>)>,
    update_nodes: Vec<(Index, Box<dyn FnOnce(NDataT) -> NDataT + 'a>)>,
    update_edges: Vec<(Index, Box<dyn FnOnce(EDataT) -> EDataT + 'a>)>,
}

impl<'tg, 'a, NDataT, EDataT> Transaction<'tg, 'a, NDataT, EDataT> {
    fn new(tgraph: &'tg TGraph<NDataT, EDataT>) -> Transaction<'tg, 'a, NDataT, EDataT> {
        Transaction {
            tgraph,
            committed: false,
            manual: false,
            inc_nodes: Arena::new(),
            inc_edges: Arena::new(),
            dec_nodes: Vec::new(),
            dec_edges: Vec::new(),
            mut_nodes: Vec::new(),
            mut_edges: Vec::new(),
            update_nodes: Vec::new(),
            update_edges: Vec::new(),
        }
    }

    pub fn new_node(&mut self, data: NDataT) -> NodeIndex {
        NodeIndex(NewIndex::New(self.inc_nodes.insert(NewNode {
            data,
            in_edges: Vec::new(),
            out_edges: Vec::new(),
        })))
    }
    pub fn new_edge(&mut self, data: EDataT, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        let idx = NewIndex::New(self.inc_edges.insert(NewEdge {
            data,
            from: from.0,
            to: to.0,
        }));
        EdgeIndex(idx)
    }

    pub fn remove_node(&mut self, node: NodeIndex) {
        match node.0 {
            NewIndex::Exist(idx) => self.dec_nodes.push(idx),
            NewIndex::New(idx) => drop(
                self.inc_nodes
                    .remove(idx)
                    .expect("TGraph: try to remove non-existent node"),
            ),
        };
    }
    pub fn remove_edge(&mut self, edge: EdgeIndex) {
        match edge.0 {
            NewIndex::Exist(idx) => self.dec_edges.push(idx),
            NewIndex::New(idx) => drop(
                self.inc_edges
                    .remove(idx)
                    .expect("TGraph: try to remove non-existent edge"),
            ),
        };
    }

    pub fn mut_node<F>(&mut self, node: NodeIndex, func: F)
    where
        F: FnOnce(&mut NDataT) + 'a,
    {
        match node.0 {
            NewIndex::Exist(idx) => self.mut_nodes.push((idx, Box::new(func))),
            NewIndex::New(idx) => func(&mut self.inc_nodes.get_mut(idx).unwrap().data),
        }
    }

    pub fn mut_edge<F>(&mut self, edge: EdgeIndex, func: F)
    where
        F: FnOnce(&mut EDataT) + 'a,
    {
        match edge.0 {
            NewIndex::Exist(idx) => self.mut_edges.push((idx, Box::new(func))),
            NewIndex::New(idx) => func(&mut self.inc_edges.get_mut(idx).unwrap().data),
        }
    }

    pub fn update_node<F>(&mut self, node: NodeIndex, func: F)
    where
        F: FnOnce(NDataT) -> NDataT + 'a,
    {
        match node.0 {
            NewIndex::Exist(idx) => self.update_nodes.push((idx, Box::new(func))),
            NewIndex::New(idx) => self.inc_nodes.update_with(idx, |x| NewNode {
                data: func(x.data),
                ..x
            }),
        }
    }

    pub fn update_edge<F>(&mut self, edge: EdgeIndex, func: F)
    where
        F: FnOnce(EDataT) -> EDataT + 'a,
    {
        match edge.0 {
            NewIndex::Exist(idx) => self.update_edges.push((idx, Box::new(func))),
            NewIndex::New(idx) => self.inc_edges.update_with(idx, |x| NewEdge {
                data: func(x.data),
                ..x
            }),
        }
    }

    pub fn commit(&mut self) -> Option<IndexConverter> {
        if self.committed {
            return None;
        }
        self.committed = true;
        let mut graph = self.tgraph.graph.borrow_mut();

        let result = Some(self.addback(&mut graph));
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
        result
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

    fn addback(&mut self, graph: &mut RefMut<Graph<NDataT, EDataT>>) -> IndexConverter {
        let node_map = graph.nodes.alloc_for_merge(&self.inc_nodes);
        let edge_map = graph.edges.alloc_for_merge(&self.inc_edges);

        let node_idx: Vec<Index> = self
            .inc_nodes
            .iter()
            .map(|(i, _)| Index { id: *i })
            .collect();
        let edge_idx: Vec<Index> = self
            .inc_edges
            .iter()
            .map(|(i, _)| Index { id: *i })
            .collect();
        for idx in node_idx {
            let n = self.inc_nodes.remove(idx).unwrap();
            let new_idx = node_map[&idx];
            graph.nodes.fill_back(
                new_idx,
                Node {
                    idx: new_idx,
                    data: n.data,
                    in_edges: convert_index(n.in_edges, &edge_map),
                    out_edges: convert_index(n.out_edges, &edge_map),
                },
            );
        }
        for idx in edge_idx {
            let e = self.inc_edges.remove(idx).unwrap();
            let new_idx = edge_map[&idx];
            let from = e.from.convert(&node_map);
            let to = e.to.convert(&node_map);
            graph.nodes[from].out_edges.insert(new_idx);
            graph.nodes[to].in_edges.insert(new_idx);
            graph.edges.fill_back(
                new_idx,
                Edge {
                    idx: new_idx,
                    data: e.data,
                    from,
                    to,
                },
            );
        }
        IndexConverter { node_map, edge_map }
    }
}

impl<'tg, 'a, NDataT, EDataT> Drop for Transaction<'tg, 'a, NDataT, EDataT> {
    fn drop(&mut self) {
        if !self.manual {
            self.commit();
        }
    }
}

pub struct IndexConverter {
    node_map: HashMap<Index, Index>,
    edge_map: HashMap<Index, Index>,
}
