//! (Preperation-) Graph module. The directed graph is represented as an adjacency lists and used for node contraction.
use crate::constants::{OSMId, Weight};
use anyhow::{Context, Ok};
use log::{debug, info};
use osm_reader::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    hash::Hash,
    path::{Path, PathBuf},
};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct NodeIndex<Idx = DefaultIdx>(Idx);

impl NodeIndex {
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

/// Short version of `NodeIndex::new`
pub fn node_index(index: usize) -> NodeIndex {
    NodeIndex::new(index)
}

/// Edge identifier.
#[derive(
    Debug, Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize,
)]
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

/// Represents OSM Node type (<https://wiki.openstreetmap.org/wiki/Node>)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub id: OSMId,
    pub lat: f64,
    pub lon: f64,
}

impl Node {
    pub fn new(id: OSMId, lat: f64, lon: f64) -> Self {
        Node { id, lat, lon }
    }
}

/// A weighted `Edge` which connects a `source` and a `target` node.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge<Idx = DefaultIdx> {
    pub source: NodeIndex<Idx>,
    pub target: NodeIndex<Idx>,
    pub weight: Weight,
}

impl Edge {
    pub fn new(
        source: NodeIndex<DefaultIdx>,
        target: NodeIndex<DefaultIdx>,
        weight: Weight,
    ) -> Self {
        Edge {
            source,
            target,
            weight,
        }
    }
}

/// A directed graph G = (V,E) stored as adjacency lists.
#[derive(Clone, Serialize, Deserialize)]
pub struct Graph<Idx = DefaultIdx> {
    /// Adjancency lists for fast lookup of incoming edges of a node
    pub edges_in: Vec<Vec<EdgeIndex<Idx>>>,
    /// Adjancency lists for fast lookup of outgoing edges of a node
    pub edges_out: Vec<Vec<EdgeIndex<Idx>>>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge<Idx>>,
    pub num_shortcuts: usize,
}

impl Graph {
    /// Creates a new empty graph
    pub fn new() -> Self {
        Self {
            edges_in: Vec::new(),
            edges_out: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            num_shortcuts: 0,
        }
    }

    /// Creates a new empty graph with preallocated memory for `num_nodes` nodes and `num_edges` edges
    pub fn with_capacity(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            edges_in: Vec::with_capacity(num_nodes),
            edges_out: Vec::with_capacity(num_nodes),
            nodes: Vec::with_capacity(num_nodes),
            edges: Vec::with_capacity(num_edges),
            num_shortcuts: 0,
        }
    }

    /// Add a new `edge` to the graph.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type
    /// **Panics** if the source or target node does not exist
    ///
    /// Returns the index of the new created edge.
    pub fn add_edge(&mut self, edge: Edge) -> EdgeIndex {
        self.add_edge_internal(edge, true)
    }

    /// Add a new `shortcut edge` to the graph.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type
    /// **Panics** if the source or target node does not exist
    ///
    /// Returns the index of the new created edge.
    pub fn add_shortcut(&mut self, edge: Edge) -> EdgeIndex {
        self.num_shortcuts += 1;
        self.add_edge_internal(edge, false)
    }

    fn add_edge_internal(&mut self, edge: Edge, allow_weight_update: bool) -> EdgeIndex {
        let edge_idx = EdgeIndex::new(self.edges.len());

        assert!(
            edge.weight > 0.0,
            "Edge must have a positive non-zero weight. Edge weight was {}",
            edge.weight
        );

        assert!(
            EdgeIndex::end() != edge_idx,
            "Maximum number of edges for index type {} exceeded",
            std::any::type_name::<DefaultIdx>()
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

        // If an edge already exists between source and target but the new edge
        // has a lower weight, replace the old edge with the new one (update the weight)
        // Since shortcuts edges are added like normal edges, we can't allow edge updates for shortcuts
        if allow_weight_update {
            for (_, e_idx) in self.edges_out[edge.source.index()].iter().enumerate() {
                let old_edge = &self.edges[e_idx.index()];
                if edge.target == old_edge.target && edge.weight < old_edge.weight {
                    debug!(
                        "Updated edge weight from {} to {}",
                        old_edge.weight, edge.weight
                    );
                    self.edges[e_idx.index()].weight = edge.weight;
                    return *e_idx;
                }
            }
        }

        self.edges_out[edge.source.index()].push(edge_idx);
        self.edges_in[edge.target.index()].push(edge_idx);
        self.edges.push(edge);

        edge_idx
    }

    /// Add multiple `edges` to the graph.
    pub fn add_edges(&mut self, edges: Vec<Edge>) {
        for edge in edges {
            self.add_edge(edge);
        }
    }

    /// Adds a new node to the graph
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let node_idx: NodeIndex = NodeIndex::new(self.nodes.len());

        assert!(
            NodeIndex::end() != node_idx,
            "Maximum number of nodes for index type {} exceeded",
            std::any::type_name::<DefaultIdx>()
        );

        // Create new entry in adjacency list for new node
        self.edges_in.push(Vec::new());
        self.edges_out.push(Vec::new());

        self.nodes.push(node);

        node_idx
    }

    /// Returns node with the given index if it exists
    pub fn node(&self, node_idx: NodeIndex) -> Option<&Node> {
        self.nodes.get(node_idx.index())
    }

    /// Returns an iterator over all nodes of the graph
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    /// Returns an mutable iterator over all nodes of the graph
    pub fn nodes_mut(&mut self) -> impl Iterator<Item = &mut Node> {
        self.nodes.iter_mut()
    }

    /// Returns an iterator over all edges of the graph
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    /// Returns an iterator over all outgoing edges of `node_idx`
    pub fn neighbors_outgoing(
        &self,
        node_idx: NodeIndex,
    ) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        // Ignore shortcuts
        self.edges_out[node_idx.index()]
            .iter()
            // .filter(|edge_idx| edge_idx.index() < self.edges.len() - self.num_shortcuts)
            .map(|edge_idx| (*edge_idx, &self.edges[edge_idx.index()]))
    }

    /// Returns an iterator over all incoming edges of `node_idx`
    pub fn neighbors_incoming(
        &self,
        node_idx: NodeIndex,
    ) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        // Ignore shortcuts
        self.edges_in[node_idx.index()]
            .iter()
            .map(|edge_idx| (*edge_idx, &self.edges[edge_idx.index()]))
    }

    /// Prints some information about the graph
    pub fn print_info(&self) {
        println!(
            "InputGraph:\t#Nodes: {}, #Edges: {}, #Shortcuts: {}",
            self.nodes.len(),
            self.edges.len() - self.num_shortcuts,
            self.num_shortcuts
        );
        println!(
            "Avg. In-Degree: {:.3}",
            crate::statistics::average_in_degree(self)
        );
        println!(
            "Avg. Out-Degree: {:.3}",
            crate::statistics::average_out_degree(self)
        );
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

        let mut edges: Vec<Edge> = Vec::new();
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

    /// Parses a pbf file and returns a graph
    pub fn from_pbf(path_to_pbf: &Path) -> anyhow::Result<Self> {
        info!("Parsing pbf file: {:?}", path_to_pbf);

        let road_graph = RoadGraph::from_pbf(path_to_pbf).context("Could not parse pbf file")?;

        parse_road_graph(road_graph)
    }

    pub fn from_pbf_with_simplification(path_to_pbf: &Path) -> anyhow::Result<Self> {
        info!("Parsing pbf file: {:?}", path_to_pbf);

        let road_graph = RoadGraph::from_pbf_with_simplification(path_to_pbf)
            .context("Could not parse pbf file")?;

        parse_road_graph(road_graph)
    }

    /// Writes all nodes as `nodes.csv` and edges as `edges.csv` to the current directory
    pub fn export_csv(&self) -> anyhow::Result<()> {
        let mut wtr = csv::Writer::from_path("nodes.csv")?;

        debug!("BEGIN writing nodes");
        for node in self.nodes() {
            wtr.serialize(node)?;
        }

        wtr.flush()?;
        debug!("FINISHED writing nodes");

        let mut wtr = csv::Writer::from_path("edges.csv")?;
        wtr.write_record(["source", "target", "weight"])?;
        debug!("BEGIN writing edges");
        for edge in self.edges() {
            wtr.write_record(&[
                edge.source.index().to_string(),
                edge.target.index().to_string(),
                edge.weight.to_string(),
            ])?;
        }

        wtr.flush()?;
        debug!("FINISHED writing edges");
        Ok(())
    }
}

fn parse_road_graph(road_graph: RoadGraph) -> Result<Graph, anyhow::Error> {
    let mut node_index: FxHashMap<i64, usize> =
        FxHashMap::with_capacity_and_hasher(road_graph.get_nodes().len(), Default::default());

    let mut g = Graph::with_capacity(road_graph.get_nodes().len(), road_graph.get_arcs().len());

    for (i, (id, [lat, lon])) in road_graph.get_nodes().iter().enumerate() {
        let node = Node::new(*id as usize, *lat, *lon);
        node_index.insert(*id, i);
        g.add_node(node);
    }

    for Arc {
        source,
        target,
        weight,
    } in road_graph.get_arcs()
    {
        if *weight <= 0.0 {
            continue;
        }
        let edge: Edge = Edge::new(
            NodeIndex::new(node_index[source]),
            NodeIndex::new(node_index[target]),
            *weight,
        );
        g.add_edge(edge);
    }

    info!("Finished parsing pbf file");
    info!(
        "Graph has {} nodes and {} edges",
        g.nodes.len(),
        g.edges.len()
    );
    Ok(g)
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to create a edge from source to target with a weight
/// ```
/// // Returns two edges. One for each direction
/// let edges = edge!(0 , 1, 3.0);
/// // Returns unidirectional edge
/// let edge = edge!(0 => 1, 3.0);
/// ```
#[macro_export]
macro_rules! edge {
    ($source:expr => $target:expr, $weight:expr) => {
        $crate::graph::Edge::new($source.into(), $target.into(), $weight)
    };
    ($source:expr , $target:expr, $weight:expr) => {
        vec![
            $crate::graph::Edge::new($source.into(), $target.into(), $weight),
            $crate::graph::Edge::new($target.into(), $source.into(), $weight),
        ]
    };
}

/// Macro to create a node with a given id, lat, lon
#[macro_export]
macro_rules! node {
    ($id:expr, $lat:expr, $lon:expr) => {
        $crate::graph::Node::new($id.into(), $lat, $lon)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_from_csv() {
        let graph: Graph = Graph::from_csv(
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
        let graph: Graph = Graph::from_pbf(&path).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges_out.len(), 2);
    }

    #[test]
    fn add_duplicate_edges() {
        let mut g = Graph::new();
        let a = g.add_node(Node::new(0, 0.0, 0.0));
        let b = g.add_node(Node::new(1, 0.0, 0.0));

        let edge1 = g.add_edge(edge!(a => b, 2.0));
        let _edge2 = g.add_edge(edge!(a => b, 1.0));

        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[edge1.index()].weight, 1.0);
    }
}
