use crate::constants::{NodeId, Weight};

#[derive(Debug, PartialEq)]
pub struct ShortestPath {
    pub nodes: Vec<NodeId>,
    pub weight: Weight,
}

impl ShortestPath {
    pub fn new(nodes: Vec<NodeId>, weight: Weight) -> Self {
        ShortestPath { nodes, weight }
    }
}
