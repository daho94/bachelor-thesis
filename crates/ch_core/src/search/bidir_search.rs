use std::collections::BinaryHeap;

use log::{debug, info};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    constants::Weight,
    graph::{DefaultIdx, EdgeIndex, NodeIndex},
    overlay_graph::OverlayGraph,
    statistics::Stats,
};

use super::{dijkstra::Candidate, shortest_path::ShortestPath};

pub type NodeData = FxHashMap<NodeIndex, (Weight, Option<EdgeIndex>)>;

pub struct BiDirSearch<'a, Idx = DefaultIdx> {
    pub stats: Stats,
    g: &'a OverlayGraph<Idx>,

    settled_fwd: FxHashSet<NodeIndex<Idx>>,
    settled_bwd: FxHashSet<NodeIndex<Idx>>,

    pub data_fwd: NodeData,
    pub data_bwd: NodeData,

    intersect_node: Option<NodeIndex<Idx>>,
}

impl<'a> BiDirSearch<'a> {
    pub fn new(graph: &'a OverlayGraph) -> Self {
        BiDirSearch {
            g: graph,
            stats: Stats::default(),
            settled_fwd: FxHashSet::default(),
            settled_bwd: FxHashSet::default(),
            data_fwd: FxHashMap::default(),
            data_bwd: FxHashMap::default(),
            intersect_node: None,
        }
    }

    fn init(&mut self) {
        self.settled_fwd.clear();
        self.settled_bwd.clear();
        self.data_bwd.clear();
        self.data_fwd.clear();
        self.intersect_node = None;
        self.stats.init();
    }

    /// Performs a bidirectional search on the graph.
    pub fn search(&mut self, source: NodeIndex, target: NodeIndex) -> Option<ShortestPath> {
        self.init();
        info!(
            "BEGIN BIDIRECTIONAL SEARCH from {:?} to {:?}",
            source, target
        );

        let mut queue_fwd = BinaryHeap::new();
        let mut queue_bwd = BinaryHeap::new();

        self.data_fwd.insert(source, (0.0, None));
        self.data_bwd.insert(target, (0.0, None));

        let mut intersect_node = None;

        queue_fwd.push(Candidate::new(source, 0.0));
        queue_bwd.push(Candidate::new(target, 0.0));

        // Do a full dijkstra on upward graph
        'outer: while !queue_fwd.is_empty() {
            if let Some(cand) = queue_fwd.pop() {
                // Stall on demand optimization
                for (_, edge) in self.g.edges_bwd(cand.node_idx) {
                    if let Some((dist, _)) = self.data_fwd.get(&edge.source) {
                        if *dist + edge.weight < cand.weight {
                            continue 'outer;
                        }
                    }
                }

                for (edge_idx, edge) in self.g.edges_fwd(cand.node_idx) {
                    let new_distance = cand.weight + edge.weight;
                    if new_distance
                        < self
                            .data_fwd
                            .get(&edge.target)
                            .unwrap_or(&(std::f64::INFINITY, None))
                            .0
                    {
                        self.data_fwd
                            .insert(edge.target, (new_distance, Some(edge_idx)));
                        queue_fwd.push(Candidate::new(edge.target, new_distance));
                    }
                }
                self.stats.nodes_settled += 1;
                self.settled_fwd.insert(cand.node_idx);
            }
        }

        // Do a full dijkstra on downward graph
        'outer: while !queue_bwd.is_empty() {
            if let Some(cand) = queue_bwd.pop() {
                // Stall on demand optimization
                for (_, edge) in self.g.edges_fwd(cand.node_idx) {
                    if let Some((dist, _)) = self.data_bwd.get(&edge.source) {
                        if *dist + edge.weight < cand.weight {
                            continue 'outer;
                        }
                    }
                }

                for (edge_idx, edge) in self.g.edges_bwd(cand.node_idx) {
                    let new_distance = cand.weight + edge.weight;
                    if new_distance
                        < self
                            .data_bwd
                            .get(&edge.source)
                            .unwrap_or(&(std::f64::INFINITY, None))
                            .0
                    {
                        self.data_bwd
                            .insert(edge.source, (new_distance, Some(edge_idx)));
                        queue_bwd.push(Candidate::new(edge.source, new_distance));
                    }
                }
                self.stats.nodes_settled += 1;
                self.settled_bwd.insert(cand.node_idx);
            }
        }

        // Find the set `I` of nodes settled in both dijkstras
        let intersect = self.settled_fwd.intersection(&self.settled_bwd);

        // Find
        // dist(s,t) = min { dist(s,v) + dist(v,t) | v in I}
        // and remember intersect node `v`
        let mut min_dist = std::f64::INFINITY;
        for node in intersect {
            let dist_fwd = self.data_fwd.get(node).unwrap().0;
            let dist_bwd = self.data_bwd.get(node).unwrap().0;

            let dist = dist_fwd + dist_bwd;
            if dist < min_dist {
                min_dist = dist;
                intersect_node = Some(*node);
            }
        }

        debug!("Intersection node: {:?}", intersect_node);
        debug!("min {{ dist(s,v) + dist(t,v) | v in I }} = {}", min_dist);

        self.stats.finish();

        if let Some(v) = intersect_node {
            // Reconstruct the path by backtracking and unpacking shortcuts
            let weight = self.data_fwd.get(&v)?.0 + self.data_bwd.get(&v)?.0;

            let path_fwd = (|| {
                let mut path = vec![];

                let mut previous_node = v;

                while let Some(prev_edge) = self.data_fwd.get(&previous_node)?.1 {
                    let unpacked = self.g.unpack_edge(prev_edge);

                    for edge_idx in unpacked.iter().rev() {
                        path.push(self.g.edge(*edge_idx).target);
                    }

                    previous_node = self.g.edge(prev_edge).source;
                }
                path.push(source);
                path.reverse();

                Some(path)
            })()
            .unwrap_or(vec![source]);
            debug!("Path fwd: {:?}", &path_fwd);

            // Add the backward path and weight
            let path_bwd = (|| {
                let mut path = vec![];

                let mut previous_node = v;

                while let Some(prev_edge) = self.data_bwd.get(&previous_node)?.1 {
                    let unpacked = self.g.unpack_edge(prev_edge);

                    for edge_idx in unpacked.iter() {
                        path.push(self.g.edge(*edge_idx).target);
                    }

                    previous_node = self.g.edge(prev_edge).target;
                }

                Some(path)
            })()
            .unwrap_or(Vec::<NodeIndex>::new());

            debug!("Path bwd: {:?}", &path_bwd);
            let path = [path_fwd, path_bwd].concat();

            info!("{}, weight: {}", self.stats, weight);

            Some(ShortestPath::new(path, weight))
        } else {
            info!("No path found");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use core::num;

    use approx::{abs_diff_eq, assert_abs_diff_eq, assert_ulps_eq};

    use crate::{
        graph::node_index,
        node_contraction::NodeContractor,
        search::assert_path,
        util::test_graphs::{generate_complex_graph, generate_simple_graph, graph_vaterstetten},
    };

    use super::*;
    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn search_on_simple_graph() {
        //           B
        //           |
        // E -> A -> C
        //      |  /
        //      D
        init_log();
        let mut g = generate_simple_graph();

        let a = 0.into();
        let b = 1.into();
        let e = 4.into();

        // A,E,D,C,B
        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];

        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run_with_order(&node_order);

        let mut bdir = BiDirSearch::new(&overlay_graph);
        let sp = bdir.search(a, b);

        assert_path(vec![0, 2, 1], 2.0, sp);

        let sp = bdir.search(e, b);
        assert_path(vec![4, 0, 2, 1], 3.0, sp);
    }

    #[test]
    fn search_on_ordered_complex_graph() {
        init_log();
        let mut g = generate_complex_graph();

        let mut dijkstra = super::super::dijkstra::Dijkstra::new(&g);

        let sp = dijkstra.search(1.into(), 6.into()); // B -> G
        dbg!(sp);
        info!("{}", dijkstra.stats);

        // [B, E, I, K, D, G, C, J, H, F, A]
        let node_order = vec![
            node_index(1),
            node_index(4),
            node_index(8),
            node_index(10),
            node_index(3),
            node_index(6),
            node_index(2),
            node_index(9),
            node_index(7),
            node_index(5),
            node_index(0),
        ];

        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run_with_order(&node_order);

        let mut bdir = BiDirSearch::new(&overlay_graph);

        let sp = bdir.search(node_index(1), node_index(6)); // B -> G
        assert_path(vec![1, 2, 9, 7, 6], 10.0, sp);

        let sp = bdir.search(node_index(0), node_index(6)); // A -> G
        assert_path(vec![0, 10, 9, 7, 6], 11.0, sp);
    }

    fn test_search(overlay_graph: &OverlayGraph, a: usize, b: usize) {
        let a = node_index(a);
        let b = node_index(b);

        let mut dijkstra = super::super::dijkstra::Dijkstra::new(overlay_graph.road_graph());
        let sp_ab = dijkstra.search(a, b);
        let sp_ba = dijkstra.search(b, a);

        let mut bidir = BiDirSearch::new(overlay_graph);
        let sp_bidir_ab = bidir.search(a, b);
        let sp_bidir_ba = bidir.search(b, a);

        if sp_ab.is_some() {
            assert_abs_diff_eq!(
                sp_ab.unwrap().weight,
                sp_bidir_ab.unwrap().weight,
                epsilon = 1e-4,
            );
        } else {
            // Both should be None
            assert_eq!(sp_ab, sp_bidir_ab);
        }

        if sp_ba.is_some() {
            assert_abs_diff_eq!(
                sp_ba.unwrap().weight,
                sp_bidir_ba.unwrap().weight,
                epsilon = 1e-4,
            );
        } else {
            // Both should be None
            assert_eq!(sp_ba, sp_bidir_ba);
        }
    }

    #[test]
    fn search_on_complex_graph() {
        init_log();
        let mut g = generate_complex_graph();

        let num_nodes = g.nodes.len();

        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run();

        let mut runner = proptest::test_runner::TestRunner::default();

        runner
            .run(&(0..num_nodes, 0..num_nodes), |(a, b)| {
                test_search(&overlay_graph, a, b);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn search_on_vaterstetten() {
        init_log();
        let mut g = graph_vaterstetten();

        let num_nodes = g.nodes.len();

        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run();

        let mut runner = proptest::test_runner::TestRunner::default();

        runner
            .run(&(0..num_nodes, 0..num_nodes), |(a, b)| {
                test_search(&overlay_graph, a, b);
                Ok(())
            })
            .unwrap();
    }
}
