use crate::constants::{NodeId, Weight};
use anyhow::Context;
use osm_reader::*;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

pub struct GraphBuilder {
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn add_edge(mut self, edge: Edge) -> Self {
        self.edges.push(edge);
        self
    }

    pub fn add_nodes(mut self, nodes: Vec<Node>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn build(self) -> Graph {
        let mut graph = Graph::new();

        graph.nodes = self.nodes;
        graph.edges = self.edges;

        graph.create_node_index();
        graph.create_adj_list();

        graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Graph {
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
    pub adj_list: Vec<Vec<Edge>>,
    node_index: HashMap<NodeId, usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: NodeId,
    pub lat: f64,
    pub lon: f64,
}

impl Node {
    pub fn new(id: NodeId, lat: f64, lon: f64) -> Self {
        Node { id, lat, lon }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub weight: Weight,
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId, weight: Weight) -> Self {
        Edge { from, to, weight }
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            edges: Vec::new(),
            nodes: Vec::new(),
            node_index: Default::default(),
            adj_list: Default::default(),
        }
    }

    pub fn connected_edges(&self, node: NodeId) -> &[Edge] {
        self.adj_list[self.node_index[&node]].as_slice()
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn from_csv(path_to_nodes: &Path, path_to_edges: &Path) -> anyhow::Result<Self> {
        let mut graph = Graph::new();

        let mut reader = csv::Reader::from_path(path_to_nodes)?;
        for result in reader.deserialize() {
            let node: Node = result.context("Failed to parse Node")?;
            graph.nodes.push(node);
        }

        let mut reader = csv::Reader::from_path(path_to_edges)?;
        for result in reader.deserialize() {
            let edge: Edge = result.context("Failed to parse Edge")?;
            graph.add_edge(edge);
        }

        graph.create_node_index();
        graph.create_adj_list();

        Ok(graph)
    }

    fn create_node_index(&mut self) {
        for (i, node) in self.nodes.iter().enumerate() {
            self.node_index.insert(node.id, i);
        }
    }

    fn create_adj_list(&mut self) {
        self.adj_list = vec![Vec::new(); self.nodes.len()];

        for edge in self.edges.iter() {
            let from = self.node_index[&edge.from];
            self.adj_list[from].push(edge.clone());
        }
    }

    pub fn from_pbf(path_to_pbf: &Path) -> anyhow::Result<Self> {
        let road_graph = RoadGraph::from_pbf(path_to_pbf).context("Could not parse pbf file")?;
        let mut graph = Graph::new();

        let mut edges = Vec::with_capacity(road_graph.get_arcs().len());
        for (from, to, weight) in road_graph.get_arcs() {
            let edge = Edge::new(*from as usize, *to as usize, *weight);
            edges.push(edge);
        }
        graph.edges = edges;

        let mut nodes = Vec::with_capacity(road_graph.get_nodes().len());
        for (id, [lat, lon]) in road_graph.get_nodes() {
            let node = Node::new(*id as usize, *lat, *lon);
            nodes.push(node);
        }
        graph.nodes = nodes;

        graph.create_node_index();
        graph.create_adj_list();

        Ok(graph)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_from_csv() {
        let graph = Graph::from_csv(
            Path::new("test_data/nodes.csv"),
            Path::new("test_data/edges.csv"),
        )
        .unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn read_from_pbf() {
        let graph = Graph::from_pbf(Path::new("test_data/minimal.osm.pbf")).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 2);
    }
}
