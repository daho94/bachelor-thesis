use std::collections::HashMap;

use crate::constants::{NodeId, Weight};
use crate::graph::*;
use crate::priority_queue::*;
use crate::statistics::Stats;

#[derive(Debug, PartialEq)]
pub struct ShortestPath {
    pub nodes: Vec<NodeId>,
    pub weight: Weight,
}

impl ShortestPath {
    fn new(nodes: Vec<NodeId>, weight: Weight) -> Self {
        ShortestPath { nodes, weight }
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

        let mut distances = HashMap::new();
        let mut previous = HashMap::new();
        let mut queue = PriorityQueue::new();

        distances.insert(src, 0.0);
        queue.push(HeapItem::new(0.0, src));

        while let Some(HeapItem { distance, node }) = queue.pop() {
            self.stats.nodes_settled += 1;
            if node == dst {
                let mut path = vec![node];
                let mut previous_node = previous.get(&node)?;
                while let Some(prev_node) = previous.get(previous_node) {
                    path.push(*previous_node);
                    previous_node = prev_node;
                }
                path.push(src);
                path.reverse();

                self.stats.finish();
                return Some(ShortestPath::new(path, distance));
            }

            if distance > *distances.get(&node).unwrap_or(&std::f64::INFINITY) {
                continue;
            }

            for edge in self.graph.connected_edges(node) {
                let new_distance = distance + edge.weight;
                if new_distance < *distances.get(&edge.to).unwrap_or(&std::f64::INFINITY) {
                    distances.insert(edge.to, new_distance);
                    previous.insert(edge.to, node);
                    queue.push(HeapItem::new(new_distance, edge.to));
                }
            }
        }

        self.stats.finish();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let mut g = Graph::new();
        g.add_edge(Edge::new(0, 1, 1.0));
        g.add_edge(Edge::new(1, 2, 1.0));
        g.add_edge(Edge::new(2, 3, 1.0));
        g.add_edge(Edge::new(3, 4, 20.0));
        g.add_edge(Edge::new(0, 5, 5.0));
        g.add_edge(Edge::new(5, 6, 1.0));
        g.add_edge(Edge::new(6, 4, 20.0));
        g.add_edge(Edge::new(6, 3, 20.0));
        g.add_edge(Edge::new(5, 7, 5.0));
        g.add_edge(Edge::new(7, 8, 1.0));
        g.add_edge(Edge::new(8, 9, 1.0));
        g.add_edge(Edge::new(9, 4, 1.0));

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
        let mut g = Graph::new();
        g.add_edge(Edge::new(0, 1, 1.0));
        g.add_edge(Edge::new(1, 2, 1.0));
        g.add_edge(Edge::new(3, 4, 3.0));
        g.add_edge(Edge::new(4, 5, 1.0));

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
        let mut g = Graph::new();
        g.add_edge(Edge::new(0, 1, 10.0));
        g.add_edge(Edge::new(0, 2, 1.0));
        g.add_edge(Edge::new(2, 3, 1.0));
        g.add_edge(Edge::new(3, 1, 1.0));
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
