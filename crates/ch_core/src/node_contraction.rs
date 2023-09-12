//! Module to build contraction hierarchies from a given [`Graph`].
//!
//! # Examples
//! ```
//! use rustc_hash::FxHashMap;
//! use ch_core::util::test_graphs::generate_simple_graph;
//! use ch_core::node_contraction::{NodeContractor};
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
    fmt::{Display, Write},
    time::{Duration, Instant},
};

use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use log::{debug, info};
use priority_queue::PriorityQueue;
use rustc_hash::FxHashSet;

use crate::{
    contraction_strategy::ContractionStrategy,
    graph::{node_index, Edge, EdgeIndex, Graph, NodeIndex},
    overlay_graph::OverlayGraph,
    witness_search::WitnessSearch,
};

type AddedEdges = (Vec<EdgeIndex>, usize);
type RemovedEdges = (Vec<EdgeIndex>, usize);
const STEP_SIZE: f64 = 5.0;

#[derive(Debug, Clone, Copy)]
pub struct ContractionParams {
    priority_params: PriorityParams,
    // Limit for lazy updates
    witness_search_limit: usize,
    // Limit for initial node ordering
    witness_search_initial_limit: usize,
}

impl ContractionParams {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn priority_params(mut self, params: PriorityParams) -> Self {
        self.priority_params = params;
        self
    }

    pub fn witness_search_limit(mut self, limit: usize) -> Self {
        self.witness_search_limit = limit;
        self
    }

    pub fn witness_search_initial_limit(mut self, limit: usize) -> Self {
        self.witness_search_initial_limit = limit;
        self
    }
}

impl Default for ContractionParams {
    fn default() -> Self {
        ContractionParams {
            priority_params: Default::default(),
            witness_search_limit: 50,
            witness_search_initial_limit: 500,
        }
    }
}

/// Coefficients for the priority function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorityParams {
    pub edge_difference_coeff: i32,
    pub contracted_neighbors_coeff: i32,
    pub search_space_coeff: i32,
    pub original_edges_coeff: i32,
}

impl PriorityParams {
    pub fn new(
        edge_difference_coeff: i32,
        contracted_neighbors_coeff: i32,
        search_space_coeff: i32,
        original_edges_coeff: i32,
    ) -> Self {
        PriorityParams {
            edge_difference_coeff,
            contracted_neighbors_coeff,
            search_space_coeff,
            original_edges_coeff,
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

    pub fn original_edges_coeff(mut self, coeff: i32) -> Self {
        self.original_edges_coeff = coeff;
        self
    }
}

// From Diploma thesis Contraction Hierarchies - Geisberger
// edge_difference_coeff: 190,
// contracted_neighbors_coeff: 120,
// search_space_coeff: 1,
// original_edges_coeff: 70,
//
// From Raster Search - Vaterstetten:
// edge_difference_coeff: 101,
// contracted_neighbors_coeff: 101,
// search_space_coeff: 6,
// original_edges_coeff: 70,
//
// From Raster Search - Saarland:
// Best aggressive params: PriorityParams {
//     edge_difference_coeff: 501,
//     contracted_neighbors_coeff: 401,
//     search_space_coeff: 7,
//     original_edges_coeff: 201,
// } with averagy query time: 75 μs
impl Default for PriorityParams {
    fn default() -> Self {
        PriorityParams {
            // edge_difference_coeff: 101,
            // contracted_neighbors_coeff: 101,
            // search_space_coeff: 6,
            // original_edges_coeff: 10,
            edge_difference_coeff: 501,
            contracted_neighbors_coeff: 401,
            search_space_coeff: 7,
            original_edges_coeff: 201,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConstructionStats {
    pub node_ordering_time: Duration,
    pub contraction_time: Duration,
    pub total_time: Duration,
    pub shortcuts_added: usize,
    timer: Instant,
}

impl Default for ConstructionStats {
    fn default() -> Self {
        ConstructionStats {
            node_ordering_time: Duration::new(0, 0),
            contraction_time: Duration::new(0, 0),
            total_time: Duration::new(0, 0),
            shortcuts_added: 0,
            timer: Instant::now(),
        }
    }
}

impl Display for ConstructionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---Construction Stats---")?;
        writeln!(
            f,
            "Node Ordering      : {:?}",
            self.node_ordering_time
        )?;
        writeln!(
            f,
            "Construction       : {:?}",
            self.contraction_time
        )?;
        writeln!(f, "------------------------")?;
        writeln!(f, "Totat time         : {:?}", self.total_time)?;
        writeln!(f, "Shortcuts added [#]: {}", self.shortcuts_added)
    }
}

impl ConstructionStats {
    fn init(&mut self) {
        self.timer = Instant::now();
        self.shortcuts_added = 0;
        self.node_ordering_time = Duration::new(0, 0);
        self.contraction_time = Duration::new(0, 0);
        self.total_time = Duration::new(0, 0);
    }

    fn stop_timer_node_ordering(&mut self) {
        self.node_ordering_time = self.timer.elapsed();
        self.total_time += self.node_ordering_time;
        self.timer = Instant::now();
    }

    fn stop_timer_construction(&mut self) {
        self.contraction_time = self.timer.elapsed();
        self.total_time += self.contraction_time;
        self.timer = Instant::now();
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
/// use ch_core::node_contraction::NodeContractor;
/// use ch_core::util::test_graphs::generate_simple_graph;
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
    /// Stores how many hops a shortcut represents
    hops: rustc_hash::FxHashMap<EdgeIndex, usize>,

    params: ContractionParams,
    stats: ConstructionStats,
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
            hops: rustc_hash::FxHashMap::with_capacity_and_hasher(num_edges, Default::default()),
            params: Default::default(),
            stats: Default::default(),
        }
    }

    pub fn new_with_params(g: &'a mut Graph, params: ContractionParams) -> Self {
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
            hops: rustc_hash::FxHashMap::with_capacity_and_hasher(num_edges, Default::default()),
            params,
            stats: ConstructionStats::default(),
        }
    }

    pub fn stats(&self) -> ConstructionStats {
        self.stats
    }

    pub fn run_with_strategy(&mut self, strategy: ContractionStrategy) -> OverlayGraph {
        info!("BEGIN contracting nodes");
        self.stats.init();
        let mut edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];
        let mut edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); self.num_nodes];

        let mut levels = vec![0; self.num_nodes];
        // Allocate additional space for shortcuts to avoid reallocations
        self.g.edges.reserve(self.g.edges.len());

        info!("Calculating initial node order...");

        let mut queue = match strategy {
            ContractionStrategy::FixedOrder(order) => {
                let mut pq = PriorityQueue::new();

                for (priority, node) in order.iter().enumerate() {
                    pq.push(*node, Reverse(priority as i32));
                }

                pq
            }
            _ => self.calc_initial_node_order(),
        };
        self.stats.stop_timer_node_ordering();

        let mut step_size = STEP_SIZE;
        let mut next_goal = step_size;

        let mut consecutive_lazy_updates = 0;
        let mut did_fixed_update = false;

        info!("Progress: {:.2}%", 0.0 * 100.0);
        let pb = ProgressBar::new(queue.len() as u64);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} Nodes ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
        while !queue.is_empty() {
            if let ContractionStrategy::LazyUpdate(strat) = strategy {
                // Do recalculation if
                // - too many lazy updates were performed consecutively
                // - at 50%
                let do_fixed_update = self.num_nodes / 2 == self.num_nodes - queue.len();
                if strat.update_periodic()
                    && 
                    // (strat.periodic_update_triggered(consecutive_lazy_updates)
                        // ||
                         (!did_fixed_update && do_fixed_update)
                        // )
                {
                    info!("Periodic update of priority queue triggered");
                    info!("Consecutive lazy updates: {}", consecutive_lazy_updates);

                    let mut new_queue = PriorityQueue::new();

                    for (v, _) in queue.iter_mut() {
                        let priority =
                            self.calc_priority(*v, 0, self.params.witness_search_initial_limit);
                        new_queue.push(*v, Reverse(priority));
                    }

                    // Replace queue with new queue
                    queue = new_queue;
                    consecutive_lazy_updates = 0;

                    if do_fixed_update {
                        did_fixed_update = true;
                    }
                }
            }

            let (node, Reverse(priority)) = queue.pop().unwrap();

            if let ContractionStrategy::LazyUpdate(strat) = strategy {
                if strat.update_jit() {
                    // Lazy Update node: If the priority of the node is worse (higher), it will be updated instead of contracted
                    let importance = self.calc_priority(node, 0, self.params.witness_search_limit);
                    // let importance = self.calc_priority_alt(node, 0, 50);

                    if importance > priority {
                        consecutive_lazy_updates += 1;
                        queue.push(node, Reverse(importance));
                        continue;
                    }
                    consecutive_lazy_updates = 0;
                }
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
                // Contracted Neighbors
                self.contracted_neighbors[neighbor.index()] += 1;
                // Search Space Depth
                levels[neighbor.index()] = max(levels[node.index()] + 1, levels[neighbor.index()]);

                if let ContractionStrategy::LazyUpdate(strat) = strategy {
                    if strat.update_local() {
                        let importance = self.calc_priority(
                            neighbor,
                            levels[neighbor.index()],
                            self.params.witness_search_limit,
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
                }
            }

            self.node_ranks[node.index()] = self.num_nodes - queue.len();

            let progress = (self.num_nodes - queue.len()) as f64 / self.num_nodes as f64;
            if progress * 100.0 >= next_goal {
                info!(
                    "Progress: {:.2}%, Shortcuts: {}",
                    progress * 100.0,
                    self.stats.shortcuts_added
                );
                if progress * 100.0 >= 95.0 {
                    step_size = 0.5;
                }
                next_goal += step_size;
            }
            pb.inc(1);
        }
        self.stats.stop_timer_construction();
        pb.finish_with_message("Done contracting nodes");
        info!("{:?}", self.stats);
        println!("{}", self.stats);

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
        self.run_with_strategy(ContractionStrategy::default())
    }

    pub fn run_with_order(&mut self, node_order: &[NodeIndex]) -> OverlayGraph {
        self.run_with_strategy(ContractionStrategy::FixedOrder(node_order))
    }

    fn add_shortcut(&mut self, edge: Edge, replaces: [EdgeIndex; 2]) -> EdgeIndex {
        let edge_idx = self.g.add_shortcut(edge);
        self.stats.shortcuts_added += 1;
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
    ) -> (RemovedEdges, AddedEdges) {
        // let mut added_shortcuts = 0;
        // let mut removed_edges = 0;

        let edges_in: Vec<(EdgeIndex, Edge)> = self
            .neighbors_incoming(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        let edges_out: Vec<(EdgeIndex, Edge)> = self
            .neighbors_outgoing(v)
            .map(|(i, e)| (i, e.clone()))
            .collect();

        // removed_edges += edges_in.len();
        // removed_edges += edges_out.len();
        let mut added_edges = Vec::new();

        let mut sum_hops_added = 0;

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
                let hops_uv = *self.hops.get(uv_idx).unwrap_or(&1);
                let hops_vw = *self.hops.get(vw_idx).unwrap_or(&1);
                sum_hops_added += hops_uv + hops_vw;

                if !is_simulation {
                    let edge_idx = self.add_shortcut(shortcut, [*uv_idx, *vw_idx]);
                    added_edges.push(edge_idx);

                    self.hops.insert(edge_idx, hops_uv + hops_vw);
                } else {
                    // Add some value for counting
                    added_edges.push(EdgeIndex::end());
                }
            }
        }

        if !is_simulation {
            self.disconnect_node(v);
        }

        let removed_edges: Vec<EdgeIndex> = [edges_in, edges_out]
            .concat()
            .iter()
            .map(|(edge_idx, _)| *edge_idx)
            .collect();

        let sum_hops_removed = removed_edges
            .iter()
            .map(|e| self.hops.get(e).unwrap_or(&1))
            .sum::<usize>(); // sum(hops in A(x))

        debug!("{v:?}: ({},{})", removed_edges.len(), added_edges.len());
        (
            (removed_edges, sum_hops_removed),
            (added_edges, sum_hops_added),
        )
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
            let importance = self.calc_priority(v, 0, self.params.witness_search_initial_limit);
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
    fn calc_priority(&mut self, v: NodeIndex, level: usize, max_nodes_settled_limit: usize) -> i32 {
        let ((removed_edges, _sum_hops_removed), (added_edges, sum_hops_added)) =
            self.handle_contract_node(v, max_nodes_settled_limit, true);

        let edge_difference = added_edges.len() as i32 - removed_edges.len() as i32;
        let contracted_neighbors = self.contracted_neighbors[v.index()];
        let original_edges_replaced = sum_hops_added;

        let params = self.params.priority_params;

        edge_difference * params.edge_difference_coeff
            + level as i32 * params.search_space_coeff
            + contracted_neighbors as i32 * params.contracted_neighbors_coeff
            + original_edges_replaced as i32 * params.original_edges_coeff
    }

    /// Implementation according to <https://doi.org/10.1145/2886843>
    /// I(x) = L(x) + |A(x)| / |D(x)| + sum(hops in A(x)) / sum(hops in D(x))
    #[allow(dead_code)]
    fn calc_priority_alt(
        &mut self,
        v: NodeIndex,
        level: usize, //L(x)
    ) -> i32 {
        let ((removed_edges, sum_hops_removed), (added_edges, sum_hops_added)) =
            self.handle_contract_node(v, self.params.witness_search_limit, true); // A(x), D(x)

        // let sum_hops_removed = removed_edges
        //     .iter()
        //     .map(|e| self.hops.get(e).unwrap_or(&1))
        //     .sum::<usize>(); // sum(hops in A(x))
        // let sum_hops_added = added_edges
        //     .iter()
        //     .map(|e| self.hops.get(e).unwrap_or(&1))
        //     .sum::<usize>(); // sum(hops in D(x))
        // let sum_hops_removed = removed_edges.iter().map(|(_, hops)| hops).sum::<usize>();
        // let sum_hops_added = added_edges.iter().map(|(_, hops)| hops).sum::<usize>();
        dbg!(sum_hops_removed, sum_hops_added);
        let importance = level as f32
            + (added_edges.len() as f32 + 1.0) / (removed_edges.len() as f32 + 1.0)
            + (sum_hops_added as f32 + 1.0) / (sum_hops_removed as f32 + 1.0);

        (importance * 1000.0) as i32
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        contraction_strategy::UpdateStrategy,
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
        dbg!(overlay_graph.node_order);

        assert_eq!(3, contractor.g.num_shortcuts)
    }

    #[test]
    // https://jlazarsfeld.github.io/ch.150.project/sections/8-contraction/
    fn contract_complex_graph_with_order() {
        init_log();
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
        let overlay_graph = contractor.run_with_order(&node_order);

        info!("Hops: {:#?}", &contractor.hops);
        info!("Shortcuts: {:#?}", &overlay_graph.shortcuts);

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
        let overlay_graph = contractor.run_with_strategy(ContractionStrategy::default());
        info!("Hops: {:#?}", &contractor.hops);
        info!("Shortcuts: {:#?}", &overlay_graph.shortcuts);
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
    fn contract_with_periodic_updates() {
        init_log();

        // let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        //     .join("../osm_reader/data/bayern_pp.osm.pbf");

        // let mut g = Graph::from_pbf_with_simplification(&path).unwrap();

        let mut g = graph_saarland();

        let mut contractor = NodeContractor::new(&mut g);
        let strategy = ContractionStrategy::LazyUpdate(UpdateStrategy::default().set_periodic_updates(true));
        contractor.run_with_strategy(strategy);
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
