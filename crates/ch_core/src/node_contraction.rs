use std::{
    cmp::Reverse,
    time::{Duration, Instant},
};

use log::{debug, info};
use priority_queue::PriorityQueue;
use rustc_hash::FxHashSet;

use crate::{
    graph::{node_index, Edge, EdgeIndex, Graph, NodeIndex},
    overlay_graph::OverlayGraph,
    witness_search::WitnessSearch,
};

const STEP_SIZE: f64 = 5.0;

pub struct NodeContractor<'a> {
    g: &'a mut Graph,
    node_ranks: Vec<usize>,
    nodes_contracted: Vec<bool>,
    nodes_removed_neighbors: Vec<usize>,
    num_nodes: usize,
    shortcuts: rustc_hash::FxHashMap<EdgeIndex, [EdgeIndex; 2]>,
}

impl<'a> NodeContractor<'a> {
    pub fn new(g: &'a mut Graph) -> Self {
        let num_nodes = g.nodes.len();
        let num_edges = g.edges.len();
        NodeContractor {
            g,
            node_ranks: vec![0; num_nodes],
            nodes_contracted: vec![false; num_nodes],
            nodes_removed_neighbors: vec![0; num_nodes],
            num_nodes,
            shortcuts: rustc_hash::FxHashMap::with_capacity_and_hasher(
                num_edges,
                Default::default(),
            ),
        }
    }

    pub fn run(&mut self) -> OverlayGraph {
        let now = Instant::now();
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];

        // Allocate additional space for shortcuts to avoid reallocations
        self.g.edges.reserve(self.g.edges.len());

        let mut queue = self.calc_initial_node_order();

        let mut step_size = STEP_SIZE;
        let mut next_goal = step_size;

        while !queue.is_empty() {
            let (node, Reverse(priority)) = queue.pop().unwrap();

            // Lazy Update node: If the priority of the node is worse (higher), it will be updated instead of contracted
            let importance = self.calc_importance(node, WitnessSearch::with_params(self, 25));

            if importance > priority {
                queue.push(node, Reverse(importance));
                continue;
            }

            debug!("=> Contracting node: {}", node.index());

            let mut neighbors = FxHashSet::default();

            for (in_idx, in_edge) in self.neighbors_incoming(node) {
                neighbors.insert(in_edge.source);
                edges_bwd[node.index()].push(in_idx);
            }

            for (out_idx, out_edge) in self.neighbors_outgoing(node) {
                neighbors.insert(out_edge.target);
                edges_fwd[node.index()].push(out_idx);
            }

            // Contract node
            self.contract_node(node);

            // Update only the priority of neighbors = Lazy Neighbor Updating
            for neighbor in neighbors {
                // Spatial Uniformity heuristic
                self.nodes_removed_neighbors[neighbor.index()] += 1;

                // Linear combination of heuristics
                let importance =
                    self.calc_importance(neighbor, WitnessSearch::with_params(self, 25));

                if let Some(Reverse(old_value)) =
                    queue.change_priority(&neighbor, Reverse(importance))
                {
                    if importance != old_value {
                        debug!(
                            "[Update] Changed priority of node {} from {} to {}",
                            neighbor.index(),
                            old_value,
                            importance
                        );
                    }
                }
            }

            self.node_ranks[node.index()] = self.num_nodes - queue.len() + 1;

            let progress = (self.num_nodes - queue.len()) as f64 / self.num_nodes as f64;
            if progress * 100.0 >= next_goal {
                info!("Progress: {:.2}%", progress * 100.0);
                if progress * 100.0 >= 95.0 {
                    step_size = 0.5;
                }
                next_goal += step_size;
            }
        }

        info!("Contracting nodes took {:?}", now.elapsed());
        info!("Added shortcuts: {}", self.g.num_shortcuts);

        self.g.edges.shrink_to_fit();
        self.shortcuts.shrink_to_fit();
        OverlayGraph::new(
            edges_fwd,
            edges_bwd,
            self.g.to_owned(),
            self.shortcuts.clone(),
        )
    }

    pub fn run_with_order(&mut self, node_order: &[NodeIndex]) -> OverlayGraph {
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        self.node_ranks = node_order.iter().map(|n| n.index()).collect();

        let now = Instant::now();
        info!("Contracting nodes");

        let mut next_goal = STEP_SIZE;

        for (progress, node) in node_order.iter().enumerate() {
            let node = *node;

            self.contract_node(node);

            for (in_idx, _) in self.neighbors_incoming(node) {
                edges_bwd[node.index()].push(in_idx);
            }

            for (out_idx, _) in self.neighbors_outgoing(node) {
                edges_fwd[node.index()].push(out_idx);
            }

            let progress = (progress + 1) as f64 / node_order.len() as f64;
            if progress * 100.0 >= next_goal {
                info!("Progress: {:.2}%", progress * 100.0);
                next_goal += STEP_SIZE;
            }
        }
        info!("Contracting nodes took {:?}", now.elapsed());
        self.g.edges.shrink_to_fit();
        self.shortcuts.shrink_to_fit();

        OverlayGraph::new(
            edges_fwd,
            edges_bwd,
            self.g.to_owned(),
            self.shortcuts.clone(),
        )
    }

    fn add_shortcut(&mut self, edge: Edge, replaces: [EdgeIndex; 2]) -> EdgeIndex {
        let edge_idx = self.g.add_edge(edge);
        // self.num_shortcuts += 1;
        self.g.num_shortcuts += 1;

        self.shortcuts.insert(edge_idx, replaces);

        edge_idx
    }

    pub(crate) fn neighbors_outgoing(
        &self,
        node_idx: NodeIndex,
    ) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.g.edges_out[node_idx.index()]
            .iter()
            .filter(move |edge_idx| {
                !self.nodes_contracted[self.g.edges[edge_idx.index()].target.index()]
            })
            .map(|edge_idx| (*edge_idx, &self.g.edges[edge_idx.index()]))
    }

    pub(crate) fn neighbors_incoming(
        &self,
        node_idx: NodeIndex,
    ) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.g.edges_in[node_idx.index()]
            .iter()
            .filter(move |edge_idx| {
                !self.nodes_contracted[self.g.edges[edge_idx.index()].source.index()]
            })
            .map(|edge_idx| (*edge_idx, &self.g.edges[edge_idx.index()]))
    }

    fn contract_node(&mut self, v: NodeIndex) -> (Duration, usize) {
        let mut time = Default::default();
        let mut added_shortcuts = 0;

        let edges_in: Vec<(EdgeIndex, Edge)> = self
            .neighbors_incoming(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        let edges_out: Vec<(EdgeIndex, Edge)> = self
            .neighbors_outgoing(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        for (uv_idx, uv) in edges_in.iter() {
            let mut max_weight = 0.0;
            let mut target_nodes = Vec::new();
            // Calculate max_weight <u,v,w>
            for (_, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                if weight > max_weight {
                    max_weight = weight;
                }
                target_nodes.push(vw.target);
            }

            // Start seach from u
            let ws = WitnessSearch::with_params(self, usize::MAX);
            let start = Instant::now();
            let res = ws.search(uv.source, &target_nodes, v, max_weight);
            time += start.elapsed();

            // Add shortcut if no better path <u,...,w> was found
            for (vw_idx, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                if weight < *res.get(&vw.target).unwrap_or(&std::f64::INFINITY) {
                    let shortcut = Edge::new(uv.source, vw.target, weight);

                    self.add_shortcut(shortcut, [*uv_idx, *vw_idx]);
                    added_shortcuts += 1;
                }
            }
        }

        self.disconnect_node(v);

        (time, added_shortcuts)
    }

    fn disconnect_node(&mut self, v: NodeIndex) {
        self.nodes_contracted[v.index()] = true;
    }

    fn calc_initial_node_order(&self) -> PriorityQueue<NodeIndex, Reverse<i32>> {
        let mut pq = PriorityQueue::new();

        for v in 0..self.num_nodes {
            let v = node_index(v);
            let edge_difference =
                self.calc_edge_difference(v, WitnessSearch::with_params(self, 500));
            pq.push(v, Reverse(edge_difference));
        }

        pq
    }

    fn calc_importance(&self, v: NodeIndex, ws: WitnessSearch) -> i32 {
        let edge_difference = self.calc_edge_difference(v, ws);
        let removed_neighbors = self.nodes_removed_neighbors[v.index()];

        edge_difference + removed_neighbors as i32
    }

    /// ED = Shortcuts - Removed edges
    fn calc_edge_difference(&self, v: NodeIndex, ws: WitnessSearch) -> i32 {
        let mut removed_edges = 0;

        let edges_in: Vec<(EdgeIndex, Edge)> = self
            .neighbors_incoming(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        let edges_out: Vec<(EdgeIndex, Edge)> = self
            .neighbors_outgoing(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        removed_edges += edges_in.len() as i32;
        removed_edges += edges_out.len() as i32;

        let mut added_shortcuts = 0;
        for (_, uv) in edges_in.iter() {
            let mut max_weight = 0.0;
            let mut target_nodes = Vec::new();
            // Calculate max_weight <u,v,w>
            for (_, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                if weight > max_weight {
                    max_weight = weight;
                }
                target_nodes.push(vw.target);
            }

            // Start seach from u
            let res = ws.search(uv.source, &target_nodes, v, max_weight);

            // Add shortcut if no better path <u,...,w> was found
            for (_, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                if weight < *res.get(&vw.target).unwrap_or(&std::f64::INFINITY) {
                    added_shortcuts += 1;
                }
            }
        }

        added_shortcuts - removed_edges
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        edge,
        graph::{DefaultIdx, Node},
        util::test_graphs::{
            generate_complex_graph, generate_simple_graph, graph_saarland, graph_vaterstetten,
        },
    };

    use super::*;

    fn init_log() {
        let _ = env_logger::builder().is_test(false).try_init();
    }

    #[test]
    fn contract_simple_graph_with_order() {
        //           B
        //           |
        // E -> A -> C
        //      |  /
        //      D
        init_log();
        let mut g = generate_simple_graph();

        // A,E,D,C,B
        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];

        let mut contractor = NodeContractor::new(&mut g);

        contractor.run_with_order(&node_order);

        assert_eq!(2, contractor.g.num_shortcuts)
    }

    #[test]
    fn contract_straight_line_of_nodes() {
        // 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7
        let mut g = Graph::<DefaultIdx>::new();

        for i in 0..8 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        for i in 0..7 {
            g.add_edge(edge!(i => i + 1, 1.0));
        }

        // let node_order = (1..5).map(node_index).collect::<Vec<_>>();

        let mut contractor = NodeContractor::new(&mut g);
        // contractor.run_with_order(&node_order);
        contractor.run();

        assert_eq!(3, contractor.g.num_shortcuts)
    }

    #[test]
    // https://jlazarsfeld.github.io/ch.150.project/sections/8-contraction/
    fn contract_complex_graph_with_order() {
        let mut g = generate_complex_graph();

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
        contractor.run_with_order(&node_order);

        assert_eq!(3 * 2, contractor.g.num_shortcuts);
    }

    #[test]
    fn contract_complex_graph() {
        init_log();
        let mut g = generate_complex_graph();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run();

        assert_eq!(2, contractor.g.num_shortcuts);
    }

    #[ignore = "Takes too long"]
    #[test]
    fn contract_saarland() {
        init_log();
        let mut g = graph_saarland();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run();
    }

    #[ignore = "Takes too long"]
    #[test]
    fn contract_bavaria_with_order() {
        init_log();

        let mut g = Graph::<DefaultIdx>::from_pbf(std::path::Path::new(
            "../osm_reader/data/bayern_pp.osm.pbf",
        ))
        .unwrap();

        // let node_order = (0..g.nodes.len()).map(node_index).collect::<Vec<_>>();

        let mut contractor = NodeContractor::new(&mut g);

        // contractor.run_with_order(&node_order);
        contractor.run();
    }

    // Lazy Update Self + Neighbors: 7890
    // Lazy Update Neighbors: 7907
    // Lazy Update Self: 7918
    #[test]
    fn contract_vaterstetten() {
        init_log();

        let mut g = graph_vaterstetten();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run();
    }

    #[test]
    fn disconnect_node() {
        let mut g = Graph::<DefaultIdx>::new();
        let a = g.add_node(Node::new(0, 0.0, 0.0));
        let b = g.add_node(Node::new(1, 0.0, 0.0));
        let c = g.add_node(Node::new(2, 0.0, 0.0));
        let u = g.add_node(Node::new(3, 0.0, 0.0));

        g.add_edge(edge!(a => u, 1.0));
        g.add_edge(edge!(u => c, 1.0));
        g.add_edge(edge!(c => b, 1.0));
        g.add_edge(edge!(u => b, 1.0));

        let mut contractor = NodeContractor::new(&mut g);
        contractor.disconnect_node(u);

        assert_eq!(contractor.neighbors_outgoing(a).count(), 0);
        assert_eq!(contractor.neighbors_outgoing(b).count(), 0);
        assert_eq!(contractor.neighbors_outgoing(c).count(), 1);

        assert_eq!(contractor.neighbors_incoming(a).count(), 0);
        assert_eq!(contractor.neighbors_incoming(b).count(), 1);
        assert_eq!(contractor.neighbors_incoming(c).count(), 0);
    }
}
