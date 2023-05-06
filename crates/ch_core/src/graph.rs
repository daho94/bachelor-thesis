use crate::constants::{OSMId, OsmId, Weight};
use anyhow::Context;
use log::info;
use osm_reader::*;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::{fmt, hash::Hash, path::Path};

/// Default integer typer for node and edge indices
/// Needs to be increased vor very large graphs > u32::max
pub type DefaultIdx = u32;

pub trait IndexType: Copy + Default + Hash + Ord + fmt::Debug {
    fn new(idx: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
}

impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x
    }
    #[inline(always)]
    fn index(&self) -> Self {
        *self
    }
    #[inline(always)]
    fn max() -> Self {
        ::std::usize::MAX
    }
}

impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u32
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max() -> Self {
        ::std::u32::MAX
    }
}

impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u16
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max() -> Self {
        ::std::u16::MAX
    }
}

impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u8
    }
    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }
    #[inline(always)]
    fn max() -> Self {
        ::std::u8::MAX
    }
}

/// Node identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct NodeIndex<Idx = DefaultIdx>(Idx);

impl<Idx: IndexType> NodeIndex<Idx> {
    #[inline]
    pub fn new(x: usize) -> Self {
        NodeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0.index()
    }

    #[inline]
    pub fn end() -> Self {
        NodeIndex(IndexType::max())
    }
}

impl<Idx: IndexType> From<Idx> for NodeIndex<Idx> {
    fn from(ix: Idx) -> Self {
        NodeIndex(ix)
    }
}

/// Edge identifier.
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize)]
pub struct EdgeIndex<Idx = DefaultIdx>(Idx);

impl<Idx: IndexType> From<Idx> for EdgeIndex<Idx> {
    fn from(ix: Idx) -> Self {
        EdgeIndex(ix)
    }
}

impl<Idx: IndexType> EdgeIndex<Idx> {
    #[inline]
    pub fn new(x: usize) -> Self {
        EdgeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0.index()
    }

    /// An invalid `EdgeIndex` used to denote absence of an edge, for example
    /// to end an adjacency list.
    #[inline]
    pub fn end() -> Self {
        EdgeIndex(IndexType::max())
    }
}

/// Represents OSM Node type (https://wiki.openstreetmap.org/wiki/Node)
#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: OSMId,
    pub lat: f64,
    pub lon: f64,
    // TODO: Add contraction number
}

impl Node {
    pub fn new(id: OsmId, lat: f64, lon: f64) -> Self {
        Node { id, lat, lon }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Edge<Idx = DefaultIdx> {
    pub source: NodeIndex<Idx>,
    pub target: NodeIndex<Idx>,
    pub weight: Weight,
    // Used to recursively unpack shortcuts
    pub shortcut_for: Option<[EdgeIndex<Idx>; 2]>,
}
pub struct Graph<Idx = DefaultIdx> {
    pub edges_in: Vec<Vec<EdgeIndex<Idx>>>,
    pub edges_out: Vec<Vec<EdgeIndex<Idx>>>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge<Idx>>,
    // node_index: FxHashMap<NodeId, usize>,
}

impl<Idx: IndexType> Edge<Idx> {
    pub fn new(source: NodeIndex<Idx>, target: NodeIndex<Idx>, weight: Weight) -> Self {
        Edge {
            source,
            target,
            weight,
            shortcut_for: None,
        }
    }

    pub fn new_shortcut(
        source: NodeIndex<Idx>,
        target: NodeIndex<Idx>,
        weight: Weight,
        shortcut_for: [EdgeIndex<Idx>; 2],
    ) -> Self {
        Edge {
            source,
            target,
            weight,
            shortcut_for: Some(shortcut_for),
        }
    }
}

impl<Idx: IndexType> Graph<Idx> {
    pub fn new() -> Self {
        Self {
            edges_in: Vec::new(),
            edges_out: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn with_capacity(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            edges_in: Vec::with_capacity(num_nodes),
            edges_out: Vec::with_capacity(num_nodes),
            nodes: Vec::with_capacity(num_nodes),
            edges: Vec::with_capacity(num_edges),
        }
    }

    // pub fn connected_edges(&self, node: OsmId) -> impl Iterator<Item = &Edge<Idx>> {
    //     todo!()
    // }

    /// Add a new `edge` to the graph.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type
    /// **Panics** if the source or target node does not exist
    ///
    /// Returns the index of the new created edge.
    pub fn add_edge(&mut self, edge: Edge<Idx>) -> EdgeIndex<Idx> {
        let edge_idx = EdgeIndex::new(self.edges.len());

        assert!(
            EdgeIndex::end() != edge_idx,
            "Maximum number of edges for index type {} exceeded",
            std::any::type_name::<Idx>()
        );
        assert!(
            edge.source.index() < self.nodes.len(),
            "Source node index ({}) does not exist",
            edge.source.index()
        );
        assert!(
            edge.target.index() < self.nodes.len(),
            "Target node index ({}) does not exist",
            edge.target.index()
        );

        self.edges_out[edge.source.index()].push(edge_idx);
        self.edges_in[edge.target.index()].push(edge_idx);
        self.edges.push(edge);

        edge_idx
    }

    /// Adds a new node to the graph
    pub fn add_node(&mut self, node: Node) -> NodeIndex<Idx> {
        let node_idx: NodeIndex<Idx> = NodeIndex::new(self.nodes.len());

        assert!(
            NodeIndex::end() != node_idx,
            "Maximum number of nodes for index type {} exceeded",
            std::any::type_name::<Idx>()
        );

        // Create new entry in adjacency list for new node
        self.edges_in.push(Vec::new());
        self.edges_out.push(Vec::new());

        self.nodes.push(node);
        node_idx
    }

    pub fn node(&self, node_idx: NodeIndex<Idx>) -> Option<&Node> {
        self.nodes.get(node_idx.index())
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn edges(&self) -> &[Edge<Idx>] {
        &self.edges
    }

    pub fn neighbors_outgoing(&self, node_idx: NodeIndex<Idx>) -> impl Iterator<Item = &Edge<Idx>> {
        self.edges_out[node_idx.index()]
            .iter()
            .map(|edge_idx| &self.edges[edge_idx.index()])
    }

    pub fn from_csv(path_to_nodes: &Path, path_to_edges: &Path) -> anyhow::Result<Self> {
        let mut nodes = Vec::new();
        let mut node_index: FxHashMap<usize, usize> = FxHashMap::default();

        let mut reader = csv::Reader::from_path(path_to_nodes)?;
        for (i, result) in reader.deserialize().enumerate() {
            let node: Node = result.context("Failed to parse Node")?;
            node_index.insert(node.id, i);
            nodes.push(node);
        }

        let mut edges: Vec<Edge<Idx>> = Vec::new();
        let mut reader = csv::Reader::from_path(path_to_edges)?;
        for result in reader.deserialize() {
            let edge: Edge<DefaultIdx> = result.context("Failed to parse Edge")?;
            edges.push(Edge::new(
                NodeIndex::new(node_index[&edge.source.index()]),
                NodeIndex::new(node_index[&edge.target.index()]),
                edge.weight,
            ));
        }

        // Build the graph
        let mut g = Graph::with_capacity(nodes.len(), edges.len());
        for node in nodes {
            g.add_node(node);
        }

        for edge in edges {
            g.add_edge(edge);
        }

        Ok(g)
    }

    pub fn from_pbf(path_to_pbf: &Path) -> anyhow::Result<Self> {
        info!("Parsing pbf file: {:?}", path_to_pbf);

        let road_graph = RoadGraph::from_pbf(path_to_pbf).context("Could not parse pbf file")?;

        let mut node_index: FxHashMap<i64, usize> =
            FxHashMap::with_capacity_and_hasher(road_graph.get_nodes().len(), Default::default());

        let mut g = Graph::with_capacity(road_graph.get_nodes().len(), road_graph.get_arcs().len());

        for (i, (id, [lat, lon])) in road_graph.get_nodes().iter().enumerate() {
            let node = Node::new(*id as usize, *lat, *lon);
            node_index.insert(*id, i);
            g.add_node(node);
        }

        for (from, to, weight) in road_graph.get_arcs() {
            let edge: Edge<Idx> = Edge::new(
                NodeIndex::new(node_index[from]),
                NodeIndex::new(node_index[to]),
                *weight,
            );
            g.add_edge(edge);
        }

        info!("Finished parsing pbf file");
        Ok(g)
    }
}

impl<Idx: IndexType> Default for Graph<Idx> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_from_csv() {
        let graph: Graph<DefaultIdx> = Graph::from_csv(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/nodes.csv"),
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/edges.csv"),
        )
        .unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges_out[0].len(), 1);
        assert_eq!(graph.edges_out[1].len(), 0);
        assert_eq!(graph.edges_in[0].len(), 0);
        assert_eq!(graph.edges_in[1].len(), 1);
    }

    #[test]
    fn read_from_pbf() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/minimal.osm.pbf");
        let graph: Graph<DefaultIdx> = Graph::from_pbf(&path).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges_out.len(), 2);
    }
}
