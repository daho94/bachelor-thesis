//! Module to build contraction hierarchies from a given [`Graph`].
//!
//! # Examples
//! ```
//! use rustc_hash::FxHashMap;
//! use crate::util::test_graphs::generate_simple_graph;
//!
//! // Create a new graph
//! let mut g = generate_simple_graph();
//!
//! // Create a new NodeContractor instance with required parameters
//! let mut contractor = NodeContractor::new(&mut g);
//!
//! // Run the contraction algorithm
//! let overlay_graph = contractor.run();
//!
//!```
//! [`Graph`]: crate::graph::Graph
use std::{
    cmp::{max, Reverse},
    time::Instant,
};

use log::{debug, info};
use priority_queue::PriorityQueue;
use rustc_hash::FxHashSet;

use crate::{
    contraction_strategy::CHStrategy,
    graph::{node_index, Edge, EdgeIndex, Graph, NodeIndex},
    overlay_graph::OverlayGraph,
    witness_search::WitnessSearch,
};

const STEP_SIZE: f64 = 5.0;

/// Parameters for the priority function
/// P(v) = edge_difference_coeff * edge_difference(v)
///     + contracted_neighbors_coeff * contracted_neighbors(v)
///     + search_space_coeff * Level(v)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorityParams {
    edge_difference_coeff: i32,
    contracted_neighbors_coeff: i32,
    search_space_coeff: i32,
}

impl PriorityParams {
    pub fn new(
        edge_difference_coeff: i32,
        contracted_neighbors_coeff: i32,
        search_space_coeff: i32,
    ) -> Self {
        PriorityParams {
            edge_difference_coeff,
            contracted_neighbors_coeff,
            search_space_coeff,
        }
    }

    pub fn edge_difference_coeff(mut self, coeff: i32) -> Self {
        self.edge_difference_coeff = coeff;
        self
    }

    pub fn contracted_neighbors_coeff(mut self, coeff: i32) -> Self {
        self.contracted_neighbors_coeff = coeff;
        self
    }

    pub fn search_space_coeff(mut self, coeff: i32) -> Self {
        self.search_space_coeff = coeff;
        self
    }
}

// From Diploma thesis Contraction Hierarchies - Geisberger
// edge_difference_coeff: 190,
// contracted_neighbors_coeff: 120,
// search_space_coeff: 1,
//
// From Raster Search - Vaterstetten:
// edge_difference_coeff: 101,
// contracted_neighbors_coeff: 101,
// search_space_coeff: 6,
//
// From Raster Search - Saarland:
// edge_difference_coeff: 401,
// contracted_neighbors_coeff: 301,
// search_space_coeff: 2,
impl Default for PriorityParams {
    fn default() -> Self {
        PriorityParams {
            edge_difference_coeff: 101,
            contracted_neighbors_coeff: 101,
            search_space_coeff: 6,
        }
    }
}

/// A struct representing a NodeContractor used for graph contraction.
///
/// This struct holds information and data structures used during the process of contracting nodes
/// in a graph. It is used to optimize graph operations and represent graph nodes and their relationships.
///
/// # Fields
/// - `g`: A mutable reference to the graph that the NodeContractor operates on.
/// - `node_ranks`: A vector of usize values representing node ranks or levels. It is also known as
///   "levels" in some contexts.
/// - `nodes_contracted`: A vector of boolean values indicating whether nodes have been contracted.
/// - `contracted_neighbors`: A vector of usize values representing the contracted neighbors of nodes.
/// - `num_nodes`: The total number of nodes in the graph.
/// - `shortcuts`: A hash map used to store shortcut information between edge indices.
/// - `priority_params`: A set of priority parameters used for optimizing contraction order.
///
/// # Examples
/// ```
/// use rustc_hash::FxHashMap;
/// use crate::util::test_graphs::generate_simple_graph;
///
/// // Create a new graph
/// let mut g = generate_simple_graph();
/// // Create a new NodeContractor instance with required parameters
/// let mut contractor = NodeContractor::new(&mut g);
///
///```
pub struct NodeContractor<'a> {
    g: &'a mut Graph,
    node_ranks: Vec<usize>, // TODO: rename to levels to match thesis
    nodes_contracted: Vec<bool>,
    contracted_neighbors: Vec<usize>,
    num_nodes: usize,
    shortcuts: rustc_hash::FxHashMap<EdgeIndex, [EdgeIndex; 2]>,
    priority_params: PriorityParams,
}

impl<'a> NodeContractor<'a> {
    pub fn new(g: &'a mut Graph) -> Self {
        let num_nodes = g.nodes.len();
        let num_edges = g.edges.len();
        NodeContractor {
            g,
            node_ranks: vec![0; num_nodes],
            nodes_contracted: vec![false; num_nodes],
            contracted_neighbors: vec![0; num_nodes],
            num_nodes,
            shortcuts: rustc_hash::FxHashMap::with_capacity_and_hasher(
                num_edges,
                Default::default(),
            ),
            priority_params: Default::default(),
        }
    }

    pub fn new_with_priority_params(g: &'a mut Graph, priority_params: PriorityParams) -> Self {
        let num_nodes = g.nodes.len();
        let num_edges = g.edges.len();
        NodeContractor {
            g,
            node_ranks: vec![0; num_nodes],
            nodes_contracted: vec![false; num_nodes],
            contracted_neighbors: vec![0; num_nodes],
            num_nodes,
            shortcuts: rustc_hash::FxHashMap::with_capacity_and_hasher(
                num_edges,
                Default::default(),
            ),
            priority_params,
        }
    }

    pub fn run_with_strategy(&mut self, strategy: CHStrategy) -> OverlayGraph {
        let now = Instant::now();
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];

        let mut levels = vec![0; self.num_nodes];
        // Allocate additional space for shortcuts to avoid reallocations
        self.g.edges.reserve(self.g.edges.len());

        let mut queue = match strategy {
            CHStrategy::FixedOrder(order) => {
                let mut pq = PriorityQueue::new();

                for (priority, node) in order.iter().enumerate() {
                    pq.push(*node, Reverse(priority as i32));
                }

                pq
            }
            _ => self.calc_initial_node_order(),
        };

        let mut step_size = STEP_SIZE;
        let mut next_goal = step_size;

        while !queue.is_empty() {
            let (node, Reverse(priority)) = queue.pop().unwrap();

            match strategy {
                CHStrategy::LazyUpdateSelfAndNeighbors | CHStrategy::LazyUpdateSelf => {
                    // Lazy Update node: If the priority of the node is worse (higher), it will be updated instead of contracted
                    let importance = self.calc_priority(node, 0, 50, self.priority_params);

                    if importance > priority {
                        queue.push(node, Reverse(importance));
                        continue;
                    }
                }
                _ => {}
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
                self.contracted_neighbors[neighbor.index()] += 1;
                levels[neighbor.index()] = max(levels[node.index()] + 1, levels[neighbor.index()]);

                match strategy {
                    CHStrategy::LazyUpdateSelfAndNeighbors | CHStrategy::LazyUpdateNeighbors => {
                        // Linear combination of heuristics
                        let importance = self.calc_priority(
                            neighbor,
                            levels[neighbor.index()],
                            25,
                            self.priority_params,
                        );

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
                    _ => {}
                }
            }

            self.node_ranks[node.index()] = self.num_nodes - queue.len();

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
            self.node_ranks.clone(),
        )
    }

    pub fn run(&mut self) -> OverlayGraph {
        self.run_with_strategy(CHStrategy::LazyUpdateSelfAndNeighbors)
    }

    pub fn run_with_order(&mut self, node_order: &[NodeIndex]) -> OverlayGraph {
        self.run_with_strategy(CHStrategy::FixedOrder(node_order))
    }

    fn add_shortcut(&mut self, edge: Edge, replaces: [EdgeIndex; 2]) -> EdgeIndex {
        let edge_idx = self.g.add_edge(edge);
        // self.num_shortcuts += 1;
        self.g.num_shortcuts += 1;

        self.shortcuts.insert(edge_idx, replaces);

        edge_idx
    }

    /// Iterator over all outgoing edges of a node v excluding edges to already contracted nodes
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

    /// Iterator over all incoming edges of a node v excluding edges to already contracted nodes
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

    /// Returns (E, S) the number of removed edges and added shortcuts
    /// If the is_simulation flag is set, the function only simulates the node contraction and returns the number of shortcuts that would be added
    fn handle_contract_node(
        &mut self,
        v: NodeIndex,
        max_nodes_settled_limit: usize,
        is_simulation: bool,
    ) -> (usize, usize) {
        let mut added_shortcuts = 0;
        let mut removed_edges = 0;

        let edges_in: Vec<(EdgeIndex, Edge)> = self
            .neighbors_incoming(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        let edges_out: Vec<(EdgeIndex, Edge)> = self
            .neighbors_outgoing(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        removed_edges += edges_in.len();
        removed_edges += edges_out.len();

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
            let ws = WitnessSearch::with_params(self, max_nodes_settled_limit);

            let res = ws.search(uv.source, &target_nodes, v, max_weight);

            // Add shortcut if no better path <u,...,w> was found
            for (vw_idx, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                let witness_weight = *res.get(&vw.target).unwrap_or(&std::f64::INFINITY);

                if witness_weight <= weight {
                    continue;
                }

                let shortcut = Edge::new(uv.source, vw.target, weight);

                if !is_simulation {
                    self.add_shortcut(shortcut, [*uv_idx, *vw_idx]);
                }
                added_shortcuts += 1;
            }
        }

        if !is_simulation {
            self.disconnect_node(v);
        }

        debug!("{v:?}: ({removed_edges},{added_shortcuts})");
        (removed_edges, added_shortcuts)
    }

    fn contract_node(&mut self, v: NodeIndex) {
        self.handle_contract_node(v, 50, false);
    }

    fn disconnect_node(&mut self, v: NodeIndex) {
        self.nodes_contracted[v.index()] = true;
    }

    fn calc_initial_node_order(&mut self) -> PriorityQueue<NodeIndex, Reverse<i32>> {
        let mut pq = PriorityQueue::new();

        for v in 0..self.num_nodes {
            let v = node_index(v);
            let importance = self.calc_priority(v, 0, 500, self.priority_params);
            pq.push(v, Reverse(importance));
        }

        pq
    }

    /// Calculates the importance/relevance of a node v
    /// The lower the value, the more important the node.
    /// Priority terms:
    /// - Edge difference: Shortcuts - Removed edges
    /// - Level: Level of the node in the hierarchy.
    // Coefficients of priority terms (From Diploma thesis Contraction Hierarchies - Geisberger)
    fn calc_priority(
        &mut self,
        v: NodeIndex,
        level: usize,
        // ws: WitnessSearch,
        max_nodes_settled_limit: usize,
        params: PriorityParams,
    ) -> i32 {
        let edge_difference = self.calc_edge_difference(v, max_nodes_settled_limit);
        let contracted_neighbors = self.contracted_neighbors[v.index()];

        edge_difference * params.edge_difference_coeff
            + level as i32 * params.search_space_coeff
            + contracted_neighbors as i32 * params.contracted_neighbors_coeff
    }

    /// ED = Shortcuts - Removed edges
    fn calc_edge_difference(
        &mut self,
        v: NodeIndex,
        max_nodes_settled_limit: usize, /*ws: WitnessSearch*/
    ) -> i32 {
        let (removed_edges, added_shortcuts) =
            self.handle_contract_node(v, max_nodes_settled_limit, true);
        added_shortcuts as i32 - removed_edges as i32
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
        let mut g = Graph::new();

        for i in 0..8 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        for i in 0..7 {
            g.add_edge(edge!(i => i + 1, 1.0));
        }

        let mut contractor = NodeContractor::new(&mut g);
        let overlay_graph = contractor.run();

        dbg!(overlay_graph.shortcuts);
        dbg!(overlay_graph.node_ranks);

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
    fn contract_complex_graph_with_optimal_order() {
        let mut g = generate_complex_graph();

        // [D, I, F, G, E, B, C, A, K, H, J]
        let node_order = vec![
            node_index(3),
            node_index(8),
            node_index(5),
            node_index(6),
            node_index(4),
            node_index(1),
            node_index(2),
            node_index(0),
            node_index(10),
            node_index(7),
            node_index(9),
        ];

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run_with_order(&node_order);

        assert_eq!(0, contractor.g.num_shortcuts);
    }

    #[test]
    fn contract_complex_graph() {
        init_log();
        let mut g = generate_complex_graph();

        let mut contractor = NodeContractor::new(&mut g);
        contractor.run_with_strategy(CHStrategy::LazyUpdateSelfAndNeighbors);
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
        let mut g = Graph::new();
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
