use crate::{constants::Weight, graph::NodeIndex};

#[derive(Debug, PartialEq, Clone)]
pub struct ShortestPath {
    pub nodes: Vec<NodeIndex>,
    pub weight: Weight,
}

impl ShortestPath {
    pub fn new(nodes: Vec<NodeIndex>, weight: Weight) -> Self {
        ShortestPath { nodes, weight }
    }
}
