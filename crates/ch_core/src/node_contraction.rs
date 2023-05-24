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
    nodes_rank: Vec<usize>,
    nodes_contracted: Vec<bool>,
    num_nodes: usize,
    num_shortcuts: usize,
}

impl<'a> NodeContractor<'a> {
    pub fn new(g: &'a mut Graph) -> Self {
        let num_nodes = g.nodes.len();
        NodeContractor {
            g,
            nodes_rank: vec![0; num_nodes],
            nodes_contracted: vec![false; num_nodes],
            num_nodes,
            num_shortcuts: 0,
        }
    }

    pub fn run(&mut self) -> OverlayGraph {
        let mut now = Instant::now();
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];

        self.g.edges.reserve(self.g.edges.len());

        let mut queue = self.calc_initial_node_order();

        let num_nodes = self.g.nodes.len();

        let mut next_goal = STEP_SIZE;

        while !queue.is_empty() {
            let node = queue.pop().unwrap().0;
            debug!("=> Contracting node: {}", node.index());

            // Contracte node
            self.contract_node(node);

            let mut neighbors = FxHashSet::default();

            for (in_idx, in_edge) in self.neighbors_incoming(node) {
                neighbors.insert(in_edge.source);
                edges_bwd[node.index()].push(in_idx);
            }

            for (out_idx, out_edge) in self.neighbors_outgoing(node) {
                neighbors.insert(out_edge.target);
                edges_fwd[node.index()].push(out_idx);
            }

            // Update priority of neighbors
            for neighbor in neighbors {
                let edge_difference =
                    self.calc_edge_difference(neighbor, WitnessSearch::with_params(self, 25));
                if let Some(Reverse(old_value)) =
                    queue.change_priority(&neighbor, Reverse(edge_difference))
                {
                    if edge_difference != old_value {
                        debug!(
                            "[Update] Changed priority of node {} from {} to {}",
                            neighbor.index(),
                            old_value,
                            edge_difference
                        );
                    }
                }
            }

            let progress = (num_nodes - queue.len()) as f64 / num_nodes as f64;
            if progress * 100.0 >= next_goal {
                info!("Progress: {:.2}%", progress * 100.0);
                next_goal += STEP_SIZE;
            }
            self.nodes_rank[node.index()] = num_nodes - queue.len();
        }

        info!("Contracting nodes took {:?}", now.elapsed());
        info!("Added shortcuts: {}", self.num_shortcuts);

        OverlayGraph::new(edges_fwd, edges_bwd, self.g)
    }

    pub fn run_with_order(&mut self, node_order: &[NodeIndex]) -> OverlayGraph {
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        self.nodes_rank = node_order.iter().map(|n| n.index()).collect();

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

        OverlayGraph::new(edges_fwd, edges_bwd, self.g)
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

    #[inline(always)]
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
            let ws = WitnessSearch::with_params(self, 25);
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
                    let shortcut =
                        Edge::new_shortcut(uv.source, vw.target, weight, [*uv_idx, *vw_idx]);

                    self.g.add_edge(shortcut);
                    self.num_shortcuts += 1;
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
            // let ws = WitnessSearch::new(g);
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
    use core::num;

    use crate::{
        edge,
        graph::{DefaultIdx, Node},
        util::test_graphs::{
            generate_complex_graph, generate_simple_graph, graph_saarland, graph_vaterstetten,
        },
    };

    use super::*;

    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
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

        assert_eq!(2, contractor.num_shortcuts)
    }

    #[test]
    fn contract_straight_line_of_nodes() {
        // 0 -> 1 -> 2 -> 3 -> 4
        let mut g = Graph::<DefaultIdx>::new();

        for i in 0..5 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        for i in 0..4 {
            g.add_edge(edge!(i => i + 1, 1.0));
        }

        let node_order = (1..5).map(node_index).collect::<Vec<_>>();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run_with_order(&node_order);

        assert_eq!(3, contractor.num_shortcuts)
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

        assert_eq!(3 * 2, contractor.num_shortcuts);
    }

    #[test]
    fn contract_complex_graph() {
        init_log();
        let mut g = generate_complex_graph();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run();

        assert_eq!(2, contractor.num_shortcuts);
    }

    #[test]
    fn contract_saarland() {
        init_log();
        let mut g = graph_saarland();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run();
    }

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
