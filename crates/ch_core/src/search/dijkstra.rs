use std::collections::BinaryHeap;

use crate::constants::{NodeId, Weight};
use crate::graph::*;
use crate::search::shortest_path::ShortestPath;
use crate::statistics::Stats;
use log::{debug, info};
use rustc_hash::FxHashMap;

#[derive(Debug)]
struct Candidate {
    node: NodeId,
    weight: Weight,
}

impl Candidate {
    fn new(node: NodeId, weight: Weight) -> Self {
        Self { node, weight }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.weight.partial_cmp(&self.weight)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        other.weight == self.weight
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub struct Dijkstra<'a> {
    pub stats: Stats,
    graph: &'a Graph,
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Dijkstra {
            graph,
            stats: Stats::default(),
        }
    }

    pub fn search(&mut self, src: NodeId, dst: NodeId) -> Option<ShortestPath> {
        self.stats.init();

        if src == dst {
            self.stats.nodes_settled += 1;
            self.stats.finish();
            return Some(ShortestPath::new(vec![src], 0.0));
        }

        let mut node_data: FxHashMap<NodeId, (Weight, Option<NodeId>)> = FxHashMap::default();
        node_data.insert(src, (0.0, None));

        let mut queue = BinaryHeap::new();

        queue.push(Candidate::new(src, 0.0));

        while let Some(Candidate { weight, node }) = queue.pop() {
            self.stats.nodes_settled += 1;

            if node == dst {
                break;
            }

            for edge in self.graph.connected_edges(node) {
                let new_distance = weight + edge.weight;
                if new_distance
                    < node_data
                        .get(&edge.to)
                        .unwrap_or(&(std::f64::INFINITY, None))
                        .0
                {
                    node_data.insert(edge.to, (new_distance, Some(node)));
                    queue.push(Candidate::new(edge.to, new_distance));
                }
            }
        }
        self.stats.finish();

        let sp = super::reconstruct_path(dst, src, &node_data);
        if sp.is_some() {
            debug!("Path found: {:?}", sp);
            info!(
                "Path found: {:?}/{} nodes settled",
                self.stats.duration.unwrap(),
                self.stats.nodes_settled
            );
        } else {
            info!(
                "No path found: {:?}/{} nodes settled",
                self.stats.duration.unwrap(),
                self.stats.nodes_settled
            );
        }

        sp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create test data for nodes
    fn create_nodes() -> Vec<Node> {
        (0..10).map(|i| Node::new(i, 0.0, 0.0)).collect()
    }

    #[test]
    fn simple_path() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let g = GraphBuilder::new()
            .add_edge(Edge::new(0, 1, 1.0))
            .add_edge(Edge::new(1, 2, 1.0))
            .add_edge(Edge::new(2, 3, 1.0))
            .add_edge(Edge::new(3, 4, 20.0))
            .add_edge(Edge::new(0, 5, 5.0))
            .add_edge(Edge::new(5, 6, 1.0))
            .add_edge(Edge::new(6, 4, 20.0))
            .add_edge(Edge::new(6, 3, 20.0))
            .add_edge(Edge::new(5, 7, 5.0))
            .add_edge(Edge::new(7, 8, 1.0))
            .add_edge(Edge::new(8, 9, 1.0))
            .add_edge(Edge::new(9, 4, 1.0))
            .add_nodes(create_nodes())
            .build();

        let mut d = Dijkstra::new(&g);

        assert_no_path(d.search(4, 0)); // Cannot be reached
        assert_path(vec![0, 5, 7, 8, 9, 4], 13.0, d.search(0, 4));
        assert_path(vec![6, 3], 20.0, d.search(6, 3));
        assert_path(vec![4], 0.0, d.search(4, 4));
        assert_path(vec![1, 2, 3, 4], 22.0, d.search(1, 4));
    }

    #[test]
    fn disconnected_graph() {
        // 0 -> 1 -> 2
        // 3 -> 4 -> 5
        let g = GraphBuilder::new()
            .add_edge(Edge::new(0, 1, 1.0))
            .add_edge(Edge::new(1, 2, 1.0))
            .add_edge(Edge::new(3, 4, 3.0))
            .add_edge(Edge::new(4, 5, 1.0))
            .add_nodes(create_nodes())
            .build();

        let mut d = Dijkstra::new(&g);

        assert_no_path(d.search(0, 3));
        assert_no_path(d.search(3, 0));
        assert_path(vec![0, 1, 2], 2.0, d.search(0, 2));
        assert_path(vec![3, 4, 5], 4.0, d.search(3, 5));
    }

    #[test]
    fn go_around() {
        // 0 -> 1
        // |    |
        // 2 -> 3
        let g = GraphBuilder::new()
            .add_edge(Edge::new(0, 1, 10.0))
            .add_edge(Edge::new(0, 2, 1.0))
            .add_edge(Edge::new(2, 3, 1.0))
            .add_edge(Edge::new(3, 1, 1.0))
            .add_nodes(create_nodes())
            .build();

        let mut d = Dijkstra::new(&g);

        assert_path(vec![0, 2, 3, 1], 3.0, d.search(0, 1));
    }

    fn assert_no_path(path: Option<ShortestPath>) {
        assert_eq!(None, path);
    }

    fn assert_path(
        expected_path: Vec<NodeId>,
        expected_weight: Weight,
        path: Option<ShortestPath>,
    ) {
        assert_eq!(
            Some(ShortestPath::new(expected_path, expected_weight)),
            path
        );
    }
}
