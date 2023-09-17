//! Implementation of the A* search algorithm.
use std::collections::BinaryHeap;

use log::{debug, info};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    constants::Weight,
    graph::{DefaultIdx, Graph, Node, NodeIndex},
    statistics::SearchStats,
};

use super::shortest_path::ShortestPath;

#[derive(Debug)]
struct Candidate<Idx = DefaultIdx> {
    node: NodeIndex<Idx>,
    real_weight: Weight,
    tentative_weight: Weight,
}

impl Candidate {
    fn new(node: NodeIndex, real_weight: Weight, estimated_weight: Weight) -> Self {
        Self {
            node,
            real_weight,
            tentative_weight: estimated_weight,
        }
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.tentative_weight.partial_cmp(&self.tentative_weight)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        other.tentative_weight == self.tentative_weight
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .tentative_weight
            .partial_cmp(&self.tentative_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub struct AStar<'a, Idx = DefaultIdx> {
    pub stats: SearchStats,
    pub nodes_settled: FxHashSet<NodeIndex<Idx>>,
    g: &'a Graph<Idx>,
}

impl<'a> AStar<'a> {
    pub fn new(g: &'a Graph) -> Self {
        AStar {
            g,
            stats: SearchStats::default(),
            nodes_settled: FxHashSet::default(),
        }
    }

    pub fn search(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        heuristic: impl Fn(&Node, &Node) -> Weight,
    ) -> Option<ShortestPath> {
        info!("BEGIN ASTAR SEARCH from {:?} to {:?}", source, target);
        self.stats.init();
        if source == target {
            self.stats.nodes_settled += 1;
            self.stats.finish();
            return Some(ShortestPath::new(vec![source], 0.0));
        }

        let mut node_data: FxHashMap<NodeIndex, (Weight, Option<NodeIndex>)> = FxHashMap::default();
        node_data.insert(source, (0.0, None));

        let mut queue = BinaryHeap::new();

        queue.push(Candidate::new(
            source,
            0.0,
            heuristic(self.g.node(source).unwrap(), self.g.node(target).unwrap()),
        ));

        while let Some(Candidate {
            tentative_weight: _,
            real_weight,
            node,
        }) = queue.pop()
        {
            self.stats.nodes_settled += 1;

            if node == target {
                break;
            }

            for (_, edge) in self.g.neighbors_outgoing(node).filter(|(edge_idx, _)| {
                edge_idx.index() < self.g.edges.len() - self.g.num_shortcuts
            }) {
                let real_weight = real_weight + edge.weight;

                if real_weight
                    < node_data
                        .get(&edge.target)
                        .unwrap_or(&(std::f64::INFINITY, None))
                        .0
                {
                    let tentative_weight = real_weight
                        + heuristic(
                            self.g.node(edge.target).unwrap(),
                            self.g.node(target).unwrap(),
                        );

                    node_data.insert(edge.target, (real_weight, Some(node)));
                    queue.push(Candidate::new(edge.target, real_weight, tentative_weight));
                }
            }

            self.nodes_settled.insert(node);
        }

        self.stats.finish();

        // let sp = super::reconstruct_path(dst, src, &node_data);
        // if sp.is_some() {
        if let Some(sp) = super::reconstruct_path(target, source, &node_data) {
            debug!("Path found: {:?}", sp);
            info!("{}, weight: {}", self.stats, sp.weight);

            Some(sp)
        } else {
            // info!(
            //     "No path found: {:?}/{} nodes settled",
            //     self.stats.duration.unwrap(),
            //     self.stats.nodes_settled
            // );
            info!(
                "No path found: {:?}/{} nodes settled",
                self.stats.duration.unwrap(),
                self.stats.nodes_settled
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{node_index, Edge},
        search::assert_no_path,
    };
    use crate::{search::assert_path, util::math::straight_line};

    use super::*;

    fn null_heuristic(_: &Node, _: &Node) -> Weight {
        0.0
    }

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

        let mut astar = AStar::new(&g);

        assert_no_path(astar.search(4.into(), 0.into(), null_heuristic)); // Cannot be reached
        assert_path(
            vec![0, 5, 7, 8, 9, 4],
            13.0,
            astar.search(0.into(), 4.into(), null_heuristic),
        );
        assert_path(
            vec![6, 3],
            20.0,
            astar.search(6.into(), 3.into(), straight_line),
        );
        assert_path(
            vec![4],
            0.0,
            astar.search(4.into(), 4.into(), straight_line),
        );
        assert_path(
            vec![1, 2, 3, 4],
            22.0,
            astar.search(1.into(), 4.into(), straight_line),
        );
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

        let mut astar = AStar::new(&g);

        assert_no_path(astar.search(0.into(), 3.into(), null_heuristic));
        assert_no_path(astar.search(3.into(), 0.into(), null_heuristic));
        assert_path(
            vec![0, 1, 2],
            2.0,
            astar.search(0.into(), 2.into(), null_heuristic),
        );
        assert_path(
            vec![3, 4, 5],
            4.0,
            astar.search(3.into(), 5.into(), null_heuristic),
        );
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

        let mut astar = AStar::new(&g);

        assert_path(
            vec![0, 2, 3, 1],
            3.0,
            astar.search(0.into(), 1.into(), null_heuristic),
        );
    }
}
