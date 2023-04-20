use crate::constants::{NodeId, Weight};
use crate::graph::*;
use crate::priority_queue::*;
use crate::statistics::Stats;
use log::{debug, info};
use rustc_hash::FxHashMap;

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

        let mut node_data: FxHashMap<NodeId, (Weight, Option<NodeId>)> = FxHashMap::default();
        node_data.insert(src, (0.0, None));

        let mut queue = PriorityQueue::new();

        queue.push(HeapItem::new(0.0, src));

        while let Some(HeapItem { distance, node }) = queue.pop() {
            self.stats.nodes_settled += 1;

            if node == dst {
                let mut path = vec![node];
                let mut previous_node = node_data.get(&node)?.1?;
                while let Some(prev_node) = node_data.get(&previous_node)?.1 {
                    path.push(previous_node);
                    previous_node = prev_node;
                }
                path.push(src);
                path.reverse();

                self.stats.finish();
                debug!("Path found: {:?}", path);
                info!(
                    "Path found: {:?}/{} nodes settled",
                    self.stats.duration.unwrap(),
                    self.stats.nodes_settled
                );
                return Some(ShortestPath::new(path, distance));
            }

            for edge in self.graph.connected_edges(node) {
                let new_distance = distance + edge.weight;
                if new_distance
                    < node_data
                        .get(&edge.to)
                        .unwrap_or(&(std::f64::INFINITY, None))
                        .0
                {
                    node_data.insert(edge.to, (new_distance, Some(node)));
                    queue.push(HeapItem::new(new_distance, edge.to));
                }
            }
        }

        self.stats.finish();
        info!(
            "No path found: {:?}/{} nodes settled",
            self.stats.duration.unwrap(),
            self.stats.nodes_settled
        );
        None
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
