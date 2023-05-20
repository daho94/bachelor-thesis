use std::{cmp::Reverse, time::Instant};

use log::{debug, info};
use priority_queue::PriorityQueue;
use rustc_hash::FxHashSet;

use crate::{
    graph::{node_index, Edge, EdgeIndex, Graph, NodeIndex},
    search_graph::SearchGraph,
    witness_search::WitnessSearch,
};

fn calc_initial_node_order(g: &Graph) -> PriorityQueue<NodeIndex, Reverse<i32>> {
    let mut pq = PriorityQueue::new();
    let num_nodes = g.nodes.len();

    for v in 0..num_nodes {
        let v = node_index(v);
        let edge_difference = calc_edge_difference(v, g);
        pq.push(v, Reverse(edge_difference));
    }

    pq
}

/// ED = Shortcuts - Removed edges
fn calc_edge_difference(v: NodeIndex, g: &Graph) -> i32 {
    let mut removed_edges = 0;

    let edges_in: Vec<(EdgeIndex, Edge)> = g
        .neighbors_incoming(v)
        .map(|(i, e)| (i, e.clone()))
        .collect();

    let edges_out: Vec<(EdgeIndex, Edge)> = g
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
        let ws = WitnessSearch::new(g);
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

/// Contract nodes by using a priority queue.
/// TODO: Find the best node order
/// 1. Calculate edge difference for each node and put them in a priority queue. This is the initial node order.
///     - Edge difference: Removed edges - shortcut edges
pub fn contract_nodes(g: &mut Graph) -> SearchGraph {
    let mut search_graph = SearchGraph::with_capacity(g.nodes.len(), g.edges.len());

    let mut queue = calc_initial_node_order(g);

    while !queue.is_empty() {
        let node = queue.pop().unwrap().0;
        debug!("=> Contracting node: {}", node.index());

        // Contracte node
        contract_node(g, node);

        let mut neighbors = FxHashSet::default();

        for (in_idx, in_edge) in g.neighbors_incoming(node) {
            neighbors.insert(in_edge.source);
            search_graph.edges_bwd[node.index()].push(in_idx);
        }

        for (out_idx, out_edge) in g.neighbors_outgoing(node) {
            neighbors.insert(out_edge.target);
            search_graph.edges_fwd[node.index()].push(out_idx);
        }

        // Update priority of neighbors
        for neighbor in neighbors {
            let edge_difference = calc_edge_difference(neighbor, g);
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
    }

    search_graph.edges = g.edges.clone();
    search_graph
}

/// Contract nodes in the graph by a given order.
///
///  u1      w1  
///    \    /
/// u1-->v-->w2
///    /    \    
///  u2      w3
pub fn contract_nodes_with_order(g: &mut Graph, node_order: &[NodeIndex]) -> SearchGraph {
    let mut search_graph = SearchGraph::new(g.nodes.len());
    let now = Instant::now();
    info!("Contracting nodes");
    for node in node_order {
        let node = *node;

        contract_node(g, node);

        for (in_idx, _) in g.neighbors_incoming(node) {
            search_graph.edges_bwd[node.index()].push(in_idx);
        }

        for (out_idx, _) in g.neighbors_outgoing(node) {
            search_graph.edges_fwd[node.index()].push(out_idx);
        }
    }
    info!("Contracting nodes took {:?}", now.elapsed());

    search_graph.edges = g.edges.clone();
    search_graph
}

fn contract_node(g: &mut Graph, v: NodeIndex) {
    let edges_in: Vec<(EdgeIndex, Edge)> = g
        .neighbors_incoming(v)
        .map(|(i, e)| (i, e.clone()))
        .collect();

    let edges_out: Vec<(EdgeIndex, Edge)> = g
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
        let ws = WitnessSearch::new(g);
        let res = ws.search(uv.source, &target_nodes, v, max_weight);

        // Add shortcut if no better path <u,...,w> was found
        for (vw_idx, vw) in edges_out.iter() {
            if uv.source == vw.target {
                continue;
            }

            let weight = uv.weight + vw.weight;
            if weight < *res.get(&vw.target).unwrap_or(&std::f64::INFINITY) {
                let shortcut = Edge::new_shortcut(uv.source, vw.target, weight, [*uv_idx, *vw_idx]);

                g.add_edge(shortcut);
            }
        }
    }

    g.disconnect_node(v);
}

#[cfg(test)]
mod tests {
    use crate::{
        edge,
        graph::{DefaultIdx, Node},
        util::test_graphs::{generate_complex_graph, generate_simple_graph},
    };

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn contract_simple_graph_with_order() {
        //           B
        //           |
        // E -> A -> C
        //      |  /
        //      D
        init();
        let mut g = generate_simple_graph();

        // A,E,D,C,B
        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];

        contract_nodes_with_order(&mut g, &node_order);

        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(2, shortcuts)
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
        contract_nodes_with_order(&mut g, &node_order);

        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(3, shortcuts)
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

        contract_nodes_with_order(&mut g, &node_order);

        // Display number of shortcuts
        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(3 * 2, shortcuts);
    }

    #[test]
    fn contract_complex_graph() {
        init();
        let mut g = generate_complex_graph();

        contract_nodes(&mut g);

        // Display number of shortcuts
        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(2, shortcuts);
    }

    #[ignore = "Takes too long"]
    #[test]
    fn vaterstetten_works() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../osm_reader/test_data/vaterstetten_pp.osm.pbf");

        let mut g = Graph::<DefaultIdx>::from_pbf(&path).unwrap();
        dbg!(g.nodes.len());
        dbg!(g.edges.len());

        // let node_order = (0..g.nodes.len()).map(node_index).collect::<Vec<_>>();

        let mut order = calc_initial_node_order(&g);

        let mut node_order = Vec::new();

        while let Some(p) = order.pop() {
            node_order.push(p.0);
        }

        // contract_nodes_with_order(&mut g, &node_order);
        contract_nodes(&mut g);

        // 46198 - Node order 0,1,2...
        // 11771 - Calculated node order

        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(3 * 2, shortcuts);
    }
}
