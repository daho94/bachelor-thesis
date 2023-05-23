use crate::{
    constants::Weight,
    graph::{IndexType, NodeIndex},
};

#[derive(Debug, PartialEq)]
pub struct ShortestPath<Idx: IndexType> {
    pub nodes: Vec<NodeIndex<Idx>>,
    pub weight: Weight,
}

impl<Idx: IndexType> ShortestPath<Idx> {
    pub fn new(nodes: Vec<NodeIndex<Idx>>, weight: Weight) -> Self {
        ShortestPath { nodes, weight }
    }
}
