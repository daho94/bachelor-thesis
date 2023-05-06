use std::collections::BinaryHeap;

use log::{debug, info};
use rustc_hash::FxHashMap;

use crate::{
    constants::{OsmId, Weight},
    graph::{Graph, Node},
    statistics::Stats,
};

use super::shortest_path::ShortestPath;

pub struct AStar<'a> {
    pub stats: Stats,
    graph: &'a Graph,
}

#[derive(Debug)]
struct Candidate {
    node: OsmId,
    real_weight: Weight,
    tentative_weight: Weight,
}

impl Candidate {
    fn new(node: OsmId, real_weight: Weight, estimated_weight: Weight) -> Self {
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

impl<'a> AStar<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        AStar {
            graph,
            stats: Stats::default(),
        }
    }

    pub fn search(
        &mut self,
        src: OsmId,
        dst: OsmId,
        heuristic: impl Fn(&Node, &Node) -> Weight,
    ) -> Option<ShortestPath> {
        self.stats.init();

        if src == dst {
            self.stats.nodes_settled += 1;
            self.stats.finish();
            return Some(ShortestPath::new(vec![src], 0.0));
        }

        let mut node_data: FxHashMap<OsmId, (Weight, Option<OsmId>)> = FxHashMap::default();
        node_data.insert(src, (0.0, None));

        let mut queue = BinaryHeap::new();

        queue.push(Candidate::new(
            src,
            0.0,
            heuristic(self.graph.node(src).unwrap(), self.graph.node(dst).unwrap()),
        ));

        while let Some(Candidate {
            tentative_weight: _,
            real_weight,
            node,
        }) = queue.pop()
        {
            self.stats.nodes_settled += 1;

            if node == dst {
                break;
            }

            for edge in self.graph.connected_edges(node) {
                let real_weight = real_weight + edge.weight;

                if real_weight
                    < node_data
                        .get(&edge.target)
                        .unwrap_or(&(std::f64::INFINITY, None))
                        .0
                {
                    let tentative_weight = real_weight
                        + heuristic(
                            self.graph.node(edge.target).unwrap(),
                            self.graph.node(dst).unwrap(),
                        );

                    node_data.insert(edge.target, (real_weight, Some(node)));
                    queue.push(Candidate::new(edge.target, real_weight, tentative_weight));
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
    use crate::graph::{Edge, GraphBuilder};
    use crate::util::math::straight_line;

    use super::*;

    // Create test data for nodes
    fn create_nodes() -> Vec<Node> {
        (0..10).map(|i| Node::new(i, 0.0, 0.0)).collect()
    }

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

        let mut astar = AStar::new(&g);

        assert_no_path(astar.search(4, 0, null_heuristic)); // Cannot be reached
        assert_path(
            vec![0, 5, 7, 8, 9, 4],
            13.0,
            astar.search(0, 4, null_heuristic),
        );
        assert_path(vec![6, 3], 20.0, astar.search(6, 3, straight_line));
        assert_path(vec![4], 0.0, astar.search(4, 4, straight_line));
        assert_path(vec![1, 2, 3, 4], 22.0, astar.search(1, 4, straight_line));
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

        let mut astar = AStar::new(&g);

        assert_no_path(astar.search(0, 3, null_heuristic));
        assert_no_path(astar.search(3, 0, null_heuristic));
        assert_path(vec![0, 1, 2], 2.0, astar.search(0, 2, null_heuristic));
        assert_path(vec![3, 4, 5], 4.0, astar.search(3, 5, null_heuristic));
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

        let mut astar = AStar::new(&g);

        assert_path(vec![0, 2, 3, 1], 3.0, astar.search(0, 1, null_heuristic));
    }

    fn assert_no_path(path: Option<ShortestPath>) {
        assert_eq!(None, path);
    }

    fn assert_path(expected_path: Vec<OsmId>, expected_weight: Weight, path: Option<ShortestPath>) {
        assert_eq!(
            Some(ShortestPath::new(expected_path, expected_weight)),
            path
        );
    }
}
