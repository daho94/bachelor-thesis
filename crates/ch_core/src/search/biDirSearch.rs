use std::collections::BinaryHeap;

use log::{debug, info};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    constants::Weight,
    graph::{DefaultIdx, EdgeIndex, NodeIndex},
    search_graph::SearchGraph,
    statistics::Stats,
};

use super::{dijkstra::Candidate, shortest_path::ShortestPath};

type NodeData = FxHashMap<NodeIndex, (Weight, Option<EdgeIndex>)>;

pub struct BiDirSearch<'a, Idx = DefaultIdx> {
    pub stats: Stats,
    g: &'a SearchGraph<Idx>,
    settled_fwd: FxHashSet<NodeIndex<Idx>>,
    settled_bwd: FxHashSet<NodeIndex<Idx>>,
    intersect_node: Option<NodeIndex<Idx>>,
}

impl<'a> BiDirSearch<'a> {
    pub fn new(graph: &'a SearchGraph) -> Self {
        BiDirSearch {
            g: graph,
            stats: Stats::default(),
            settled_fwd: FxHashSet::default(),
            settled_bwd: FxHashSet::default(),
            intersect_node: None,
        }
    }

    fn init(&mut self) {
        self.settled_fwd.clear();
        self.settled_bwd.clear();
        self.intersect_node = None;
        self.stats.init();
    }

    /// Performs a bidirectional search on the graph.
    pub fn search(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
    ) -> Option<ShortestPath<DefaultIdx>> {
        self.init();
        debug!(
            "BEGIN BIDIRECTIONAL SEARCH from {:?} to {:?}",
            source, target
        );

        let mut queue_fwd = BinaryHeap::new();
        let mut queue_bwd = BinaryHeap::new();

        let mut data_fwd: NodeData = FxHashMap::default();
        let mut data_bwd = FxHashMap::default();

        data_fwd.insert(source, (0.0, None));
        data_bwd.insert(target, (0.0, None));

        let mut intersect_node = None;

        queue_fwd.push(Candidate::new(source, 0.0));
        queue_bwd.push(Candidate::new(target, 0.0));

        // Do a full dijkstra on upward graph
        while !queue_fwd.is_empty() {
            if let Some(cand) = queue_fwd.pop() {
                for (edge_idx, edge) in self.g.edges_fwd(cand.node_idx) {
                    let new_distance = cand.weight + edge.weight;
                    if new_distance
                        < data_fwd
                            .get(&edge.target)
                            .unwrap_or(&(std::f64::INFINITY, None))
                            .0
                    {
                        data_fwd.insert(edge.target, (new_distance, Some(edge_idx)));
                        queue_fwd.push(Candidate::new(edge.target, new_distance));
                    }
                }
                self.stats.nodes_settled += 1;
                self.settled_fwd.insert(cand.node_idx);
            }
        }

        // Do a full dijkstra on downward graph
        while !queue_bwd.is_empty() {
            if let Some(cand) = queue_bwd.pop() {
                for (edge_idx, edge) in self.g.edges_bwd(cand.node_idx) {
                    let new_distance = cand.weight + edge.weight;
                    if new_distance
                        < data_bwd
                            .get(&edge.source)
                            .unwrap_or(&(std::f64::INFINITY, None))
                            .0
                    {
                        data_bwd.insert(edge.source, (new_distance, Some(edge_idx)));
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
        // dist(s,t) = min { dist(s,v) + dist(t,v) | v in I}
        // and remember intersect node `v`
        let mut min_dist = std::f64::INFINITY;
        for node in intersect {
            let dist_fwd = data_fwd.get(node).unwrap().0;
            let dist_bwd = data_bwd.get(node).unwrap().0;

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
            let weight = data_fwd.get(&v)?.0 + data_bwd.get(&v)?.0;

            let path_fwd = (|| {
                let mut path = vec![];

                let mut previous_node = self.g.edges[data_fwd.get(&v)?.1?.index()].source;

                while let Some(prev_edge) = data_fwd.get(&previous_node)?.1 {
                    let unpacked = self.g.unpack_edge(prev_edge);

                    for edge_idx in unpacked.iter().rev() {
                        path.push(self.g.edges[edge_idx.index()].target);
                    }

                    previous_node = self.g.edges[prev_edge.index()].source;
                }
                path.push(source);
                path.reverse();

                Some(path)
            })()
            .unwrap_or(vec![]);
            debug!("Path fwd: {:?}", &path_fwd);

            // Add the backward path and weight
            let path_bwd = (|| {
                let mut path = vec![];

                let mut previous_node = self.g.edges[data_bwd.get(&v)?.1?.index()].source;

                while let Some(prev_edge) = data_bwd.get(&previous_node)?.1 {
                    let unpacked = self.g.unpack_edge(prev_edge);

                    let mut segment = vec![];
                    for edge_idx in unpacked.iter().rev() {
                        segment.push(self.g.edges[edge_idx.index()].target);
                    }
                    segment.reverse();

                    path.append(&mut segment);

                    previous_node = self.g.edges[prev_edge.index()].target;
                }

                Some(path)
            })()
            .unwrap_or(Vec::<NodeIndex>::new());

            debug!("Path bwd: {:?}", &path_bwd);
            let path = [path_fwd, vec![v], path_bwd].concat();

            debug!("Path combined: {:?}", &path);

            info!("{}", self.stats);

            Some(ShortestPath::new(path, weight))
        } else {
            debug!("No path found");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::node_index,
        node_contraction::{contract_nodes, contract_nodes_with_order},
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

        let search_graph = contract_nodes_with_order(&mut g, &node_order);

        let mut bdir = BiDirSearch::new(&search_graph);
        let sp = bdir.search(a, b);

        assert_path(vec![0, 2, 1], 2.0, sp);

        let sp = bdir.search(e, b);
        assert_path(vec![4, 0, 2, 1], 3.0, sp);
    }

    #[test]
    fn search_on_complex_path() {
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

        let search_graph = contract_nodes_with_order(&mut g, &node_order);

        let mut bdir = BiDirSearch::new(&search_graph);

        let sp = bdir.search(node_index(1), node_index(6)); // B -> G
        assert_path(vec![1, 2, 9, 7, 6], 10.0, sp);

        let sp = bdir.search(node_index(0), node_index(6)); // A -> G
        assert_path(vec![0, 10, 9, 7, 6], 11.0, sp);
    }

    #[ignore = "Takes too long"]
    #[test]
    fn search_on_vaterstetten() {
        init_log();
        let mut g = graph_vaterstetten();
        let a = node_index(1701);
        let b = node_index(278);

        let mut dijkstra = super::super::dijkstra::Dijkstra::new(&g);
        let sp = dijkstra.search(a, b);
        // 4264 nodes settled

        let search_graph = contract_nodes(&mut g);
        let mut bidir = BiDirSearch::new(&search_graph);
        let sp_bidir = bidir.search(a, b);

        // 137 nodes settled

        assert_eq!(sp.unwrap().nodes, sp_bidir.unwrap().nodes);
    }
}
