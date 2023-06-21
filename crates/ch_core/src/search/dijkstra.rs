use std::collections::BinaryHeap;

use crate::constants::Weight;
use crate::graph::*;
use crate::search::shortest_path::ShortestPath;
use crate::statistics::Stats;
use log::{debug, info};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub(crate) struct Candidate<Idx = DefaultIdx> {
    pub(crate) node_idx: NodeIndex<Idx>,
    pub(crate) weight: Weight,
}

impl Candidate {
    pub(crate) fn new(node_idx: NodeIndex, weight: Weight) -> Self {
        Self { node_idx, weight }
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

pub struct Dijkstra<'a, Idx = DefaultIdx> {
    pub stats: Stats,
    g: &'a Graph<Idx>,
}

impl<'a> Dijkstra<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Dijkstra {
            g: graph,
            stats: Stats::default(),
        }
    }

    pub fn search(&mut self, source: NodeIndex, target: NodeIndex) -> Option<ShortestPath> {
        self.stats.init();

        if source == target {
            self.stats.nodes_settled += 1;
            self.stats.finish();
            return Some(ShortestPath::new(vec![source], 0.0));
        }

        let mut node_data: FxHashMap<NodeIndex, (Weight, Option<NodeIndex>)> = FxHashMap::default();
        node_data.insert(source, (0.0, None));

        let mut queue = BinaryHeap::new();

        queue.push(Candidate::new(source, 0.0));

        while let Some(Candidate { weight, node_idx }) = queue.pop() {
            self.stats.nodes_settled += 1;

            if node_idx == target {
                break;
            }

            for (_, edge) in self.g.neighbors_outgoing(node_idx).filter(|(edge_idx, _)| {
                edge_idx.index() < self.g.edges.len() - self.g.num_shortcuts
            }) {
                let new_distance = weight + edge.weight;
                if new_distance
                    < node_data
                        .get(&edge.target)
                        .unwrap_or(&(std::f64::INFINITY, None))
                        .0
                {
                    node_data.insert(edge.target, (new_distance, Some(node_idx)));
                    queue.push(Candidate::new(edge.target, new_distance));
                }
            }
        }
        self.stats.finish();

        let sp = super::reconstruct_path(target, source, &node_data);
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
    use crate::search::{assert_no_path, assert_path};

    use super::*;

    #[test]
    fn simple_path() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let mut g = Graph::new();

        for i in 0..10 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        g.add_edge(Edge::new(node_index(0), node_index(1), 1.0));
        g.add_edge(Edge::new(node_index(1), node_index(2), 1.0));
        g.add_edge(Edge::new(node_index(2), node_index(3), 1.0));
        g.add_edge(Edge::new(node_index(3), node_index(4), 20.0));
        g.add_edge(Edge::new(node_index(0), node_index(5), 5.0));
        g.add_edge(Edge::new(node_index(5), node_index(6), 1.0));
        g.add_edge(Edge::new(node_index(6), node_index(4), 20.0));
        g.add_edge(Edge::new(node_index(6), node_index(3), 20.0));
        g.add_edge(Edge::new(node_index(5), node_index(7), 5.0));
        g.add_edge(Edge::new(node_index(7), node_index(8), 1.0));
        g.add_edge(Edge::new(node_index(8), node_index(9), 1.0));
        g.add_edge(Edge::new(node_index(9), node_index(4), 1.0));

        let mut d = Dijkstra::new(&g);

        assert_no_path(d.search(node_index(4), node_index(0))); // Cannot be reached
        assert_path(vec![0, 5, 7, 8, 9, 4], 13.0, d.search(0.into(), 4.into()));
        assert_path(vec![6, 3], 20.0, d.search(6.into(), 3.into()));
        assert_path(vec![4], 0.0, d.search(4.into(), 4.into()));
        assert_path(vec![1, 2, 3, 4], 22.0, d.search(1.into(), 4.into()));
    }

    #[test]
    fn disconnected_graph() {
        // 0 -> 1 -> 2
        // 3 -> 4 -> 5
        let mut g = Graph::new();
        for i in 0..6 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        g.add_edge(Edge::new(node_index(0), node_index(1), 1.0));
        g.add_edge(Edge::new(node_index(1), node_index(2), 1.0));
        g.add_edge(Edge::new(node_index(3), node_index(4), 3.0));
        g.add_edge(Edge::new(node_index(4), node_index(5), 1.0));

        let mut d = Dijkstra::new(&g);

        assert_no_path(d.search(0.into(), 3.into()));
        assert_no_path(d.search(3.into(), 0.into()));
        assert_path(vec![0, 1, 2], 2.0, d.search(0.into(), 2.into()));
        assert_path(vec![3, 4, 5], 4.0, d.search(3.into(), 5.into()));
    }

    #[test]
    fn go_around() {
        // 0 -> 1
        // |    |
        // 2 -> 3
        let mut g = Graph::new();
        let a = g.add_node(Node::new(0, 0.0, 0.0));
        let b = g.add_node(Node::new(1, 0.0, 0.0));
        let c = g.add_node(Node::new(2, 0.0, 0.0));
        let d = g.add_node(Node::new(3, 0.0, 0.0));

        g.add_edge(Edge::new(a, b, 10.0));
        g.add_edge(Edge::new(a, c, 1.0));
        g.add_edge(Edge::new(c, d, 1.0));
        g.add_edge(Edge::new(d, b, 1.0));

        let mut d = Dijkstra::new(&g);

        assert_path(vec![0, 2, 3, 1], 3.0, d.search(a, b));
    }
}
