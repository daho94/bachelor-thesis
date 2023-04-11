use crate::constants::{NodeId, Weight};
use anyhow::Context;
use osm_reader::*;
use serde::Deserialize;
use std::path::Path;

pub struct Graph {
    pub edges: Vec<Edge>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
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
        }
    }

    pub fn connected_edges(&self, node: NodeId) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.from == node)
            .map(|edge| edge.clone())
            .collect()
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

        Ok(graph)
    }

    pub fn from_pbf(path_to_pbf: &Path) -> anyhow::Result<Self> {
        let road_graph = RoadGraph::from_pbf(path_to_pbf).context("Could not parse pbf file")?;
        let mut graph = Graph::new();

        for (from, to, weight) in road_graph.get_arcs() {
            let edge = Edge::new(*from as usize, *to as usize, *weight);
            graph.add_edge(edge);
        }

        for (id, [lat, lon]) in road_graph.get_nodes() {
            let node = Node::new(*id as usize, *lat, *lon);
            graph.add_node(node);
        }

        Ok(graph)
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

    fn read_from_pbf() {
        let graph = Graph::from_pbf(Path::new("test_data/minimal.osm.pbf")).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
