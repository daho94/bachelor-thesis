use rustc_hash::FxHashSet;

use crate::{
    graph::{node_index, Edge, EdgeIndex, Graph, NodeIndex},
    witness_search::WitnessSearch,
};

/// Contract nodes in the graph.
///
///  u1      w1  
///    \    /
/// u1-->v-->w2
///    /    \    
///  u2      w3
pub fn contract_nodes(g: &mut Graph, node_order: &[NodeIndex]) {
    let mut removed_edges = FxHashSet::default();
    for v in node_order {
        let v = *v;
        let edges_in: Vec<(EdgeIndex, Edge)> = g
            .neighbors_incoming(v)
            // Clone edge to avoid borrowing issues
            // Ignore removed edges
            .filter(|(i, _)| !removed_edges.contains(i))
            .map(|(i, e)| (i, e.clone()))
            .collect();

        let edges_out: Vec<(EdgeIndex, Edge)> = g
            .neighbors_outgoing(v)
            // Clone edge to avoid borrowing issues
            // Ignore removed edges
            .filter(|(i, _)| !removed_edges.contains(i))
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
            let res = ws.search(uv.source, &target_nodes, v, max_weight, &removed_edges);

            // Add shortcut if no better path <u,...,w> was found
            for (vw_idx, vw) in edges_out.iter() {
                if uv.source == vw.target {
                    continue;
                }

                let weight = uv.weight + vw.weight;
                if weight < *res.get(&vw.target).unwrap_or(&std::f64::INFINITY) {
                    let shortcut =
                        Edge::new_shortcut(uv.source, vw.target, weight, [*uv_idx, *vw_idx]);

                    g.add_edge(shortcut);
                }
            }
        }

        // Remove edges for further usage
        for (uv_idx, _) in edges_in.iter() {
            removed_edges.insert(*uv_idx);
        }

        for (vw_idx, _) in edges_out.iter() {
            removed_edges.insert(*vw_idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        edge,
        graph::{DefaultIdx, Node},
    };

    use super::*;

    #[test]
    fn contract_simple_graph() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let mut g = Graph::<DefaultIdx>::new();

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

        let node_order = (0..10).map(node_index).collect::<Vec<_>>();
        contract_nodes(&mut g, &node_order);
        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        dbg!(shortcuts);
    }

    #[test]
    // https://jlazarsfeld.github.io/ch.150.project/sections/8-contraction/
    fn contract_complex_graph() {
        let mut g = Graph::<DefaultIdx>::new();

        // 'A'..='K'
        for i in 0..11 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        g.add_edges(edge!(0, 1, 3.0)); // A <=> B
        g.add_edges(edge!(0, 2, 5.0)); // A <=> C
        g.add_edges(edge!(0, 10, 3.0)); // A <=> K

        g.add_edges(edge!(1, 3, 5.0)); // B <=> D
        g.add_edges(edge!(1, 2, 3.0)); // B <=> C

        g.add_edges(edge!(2, 3, 2.0)); // C <=> D
        g.add_edges(edge!(2, 9, 2.0)); // C <=> J

        g.add_edges(edge!(3, 9, 4.0)); // D <=> J
        g.add_edges(edge!(3, 4, 7.0)); // D <=> E

        g.add_edges(edge!(4, 9, 3.0)); // E <=> J
        g.add_edges(edge!(4, 5, 6.0)); // E <=> F

        g.add_edges(edge!(5, 7, 2.0)); // F <=> H
        g.add_edges(edge!(5, 6, 4.0)); // F <=> G

        g.add_edges(edge!(6, 7, 3.0)); // G <=> H
        g.add_edges(edge!(6, 8, 5.0)); // G <=> I

        g.add_edges(edge!(7, 8, 3.0)); // H <=> I
        g.add_edges(edge!(7, 9, 2.0)); // H <=> J

        g.add_edges(edge!(8, 9, 4.0)); // I <=> J
        g.add_edges(edge!(8, 10, 6.0)); // I <=> K

        g.add_edges(edge!(9, 10, 3.0)); // J <=> K

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

        contract_nodes(&mut g, &node_order);

        // Display number of shortcuts
        let shortcuts = g.edges().filter(|e| e.is_shortcut()).count();
        assert_eq!(3 * 2, shortcuts);
    }
}
