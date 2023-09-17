//! Implementation of the bidirectional Dijkstra search algorithm.
use std::collections::BinaryHeap;

use crate::constants::Weight;
use crate::graph::*;
use crate::search::shortest_path::ShortestPath;
use crate::statistics::SearchStats;
use log::{debug, info};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug)]
pub struct Candidate<Idx = DefaultIdx> {
    pub node_idx: NodeIndex<Idx>,
    pub weight: Weight,
}

impl Candidate {
    pub fn new(node_idx: NodeIndex, weight: Weight) -> Self {
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

pub struct BidirDijkstra<'a, Idx = DefaultIdx> {
    pub stats: SearchStats,
    pub settled_fwd: FxHashSet<NodeIndex<Idx>>,
    pub settled_bwd: FxHashSet<NodeIndex<Idx>>,
    pub data_fwd: NodeData,
    pub data_bwd: NodeData,
    pub best_weight: Weight,
    pub intersect_node: Option<NodeIndex<Idx>>,
    g: &'a Graph<Idx>,
}

type NodeData = FxHashMap<NodeIndex, (Weight, Option<NodeIndex>)>;

impl<'a> BidirDijkstra<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        BidirDijkstra {
            g: graph,
            settled_fwd: FxHashSet::default(),
            settled_bwd: FxHashSet::default(),
            data_fwd: FxHashMap::default(),
            data_bwd: FxHashMap::default(),
            stats: SearchStats::default(),
            best_weight: Weight::MAX,
            intersect_node: None,
        }
    }

    pub fn init(&mut self) {
        self.settled_fwd.clear();
        self.settled_bwd.clear();
        self.data_fwd.clear();
        self.data_bwd.clear();
        self.best_weight = Weight::MAX;
        self.intersect_node = None;
        self.stats.init();
    }

    fn get_weight_bwd(&self, node: NodeIndex) -> Weight {
        self.data_bwd.get(&node).unwrap_or(&(Weight::MAX, None)).0
    }

    fn get_weight_fwd(&self, node: NodeIndex) -> Weight {
        self.data_fwd.get(&node).unwrap_or(&(Weight::MAX, None)).0
    }

    fn reconstruct_shortest_path(
        &mut self,
        intersect_node: NodeIndex,
        source: NodeIndex,
    ) -> Option<ShortestPath> {
        let weight = self.best_weight;

        let path_fwd = (|| {
            let mut path = vec![];
            let mut next_node = intersect_node;

            while let Some(prev_node) = self.data_fwd.get(&next_node)?.1 {
                path.push(prev_node);
                next_node = prev_node;
            }
            path.reverse();

            Some(path)
        })()
        .unwrap_or(vec![source]);

        debug!("Path fwd: {:?}", &path_fwd);

        let path_bwd = (|| {
            let mut path = vec![];
            let mut next_node = intersect_node;

            while let Some(prev_node) = self.data_bwd.get(&next_node)?.1 {
                path.push(prev_node);
                next_node = prev_node;
            }

            Some(path)
        })()
        .unwrap_or(Vec::<NodeIndex>::new());

        debug!("Path bwd: {:?}", &path_bwd);
        let path = [path_fwd, vec![intersect_node], path_bwd].concat();

        Some(ShortestPath::new(path, weight))
    }

    pub fn search(&mut self, source: NodeIndex, target: NodeIndex) -> Option<ShortestPath> {
        self.init();

        info!(
            "BEGIN bidir. DIJKSTRA SEARCH from {:?} to {:?}",
            source, target
        );

        if source == target {
            self.stats.nodes_settled += 1;
            self.stats.finish();
            return Some(ShortestPath::new(vec![source], 0.0));
        }

        self.data_fwd.insert(source, (0.0, None));
        self.data_bwd.insert(target, (0.0, None));

        let mut queue_fwd = BinaryHeap::new();
        let mut queue_bwd = BinaryHeap::new();

        queue_fwd.push(Candidate::new(source, 0.0));
        queue_bwd.push(Candidate::new(target, 0.0));

        while !queue_fwd.is_empty() && !queue_bwd.is_empty() {
            let u = queue_fwd.pop().unwrap();
            let v = queue_bwd.pop().unwrap();

            self.settled_fwd.insert(u.node_idx);
            self.settled_bwd.insert(v.node_idx);

            // Forward search
            for (_, edge) in self
                .g
                .neighbors_outgoing(u.node_idx)
                .filter(|(edge_idx, _)| {
                    edge_idx.index() < self.g.edges.len() - self.g.num_shortcuts
                })
            {
                let new_distance = u.weight + edge.weight;
                if !self.settled_fwd.contains(&edge.target)
                    && new_distance < self.get_weight_fwd(edge.target)
                {
                    self.data_fwd
                        .insert(edge.target, (new_distance, Some(u.node_idx)));
                    queue_fwd.push(Candidate::new(edge.target, new_distance));
                }

                if !self.settled_bwd.contains(&edge.target)
                    && u.weight + edge.weight + self.get_weight_bwd(edge.target) < self.best_weight
                {
                    debug!("FWD: new best_weight: {}", self.best_weight);
                    self.best_weight = u.weight + edge.weight + self.get_weight_bwd(edge.target);
                    self.intersect_node = Some(edge.target);
                }
            }
            self.stats.nodes_settled += 1;

            // Backward search
            for (_, edge) in self
                .g
                .neighbors_incoming(v.node_idx)
                .filter(|(edge_idx, _)| {
                    edge_idx.index() < self.g.edges.len() - self.g.num_shortcuts
                })
            {
                let new_distance = v.weight + edge.weight;
                if !self.settled_bwd.contains(&edge.source)
                    && new_distance < self.get_weight_bwd(edge.source)
                {
                    self.data_bwd
                        .insert(edge.source, (new_distance, Some(v.node_idx)));
                    queue_bwd.push(Candidate::new(edge.source, new_distance));
                }

                if self.settled_fwd.contains(&edge.source)
                    && v.weight + edge.weight + self.get_weight_fwd(edge.source) < self.best_weight
                {
                    debug!("BWD: new best_weight: {}", self.best_weight);
                    self.best_weight = v.weight + edge.weight + self.get_weight_fwd(edge.source);
                    self.intersect_node = Some(edge.source);
                }
            }

            self.stats.nodes_settled += 1;

            if self.get_weight_fwd(u.node_idx) + self.get_weight_bwd(v.node_idx) >= self.best_weight
            {
                debug!("Vorzeitig BREAK");
                break;
            }
        }

        self.stats.finish();

        info!("Intersect node: {:?}", self.intersect_node);
        info!("Weight: {}", self.best_weight);

        if let Some(intersect_node) = self.intersect_node {
            info!("{}, weight: {}", self.stats, self.best_weight);
            self.reconstruct_shortest_path(intersect_node, source)
        } else {
            info!("No path found");
            None
        }
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
        env_logger::init();
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

        let mut d = BidirDijkstra::new(&g);

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

        let mut d = BidirDijkstra::new(&g);

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

        let mut d = BidirDijkstra::new(&g);

        assert_path(vec![0, 2, 3, 1], 3.0, d.search(a, b));
    }
}
