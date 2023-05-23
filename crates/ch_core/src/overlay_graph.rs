use std::fmt::Display;

use crate::graph::{DefaultIdx, Edge, EdgeIndex, Graph, NodeIndex};

/// Representation of the graph after running
///     - NodeContractor::run
///     - NodeContractor::run_with_order
/// Shortes path calculation is performed on this graph.
pub struct OverlayGraph<'a, Idx = DefaultIdx> {
    // Represents the upward graph G↑
    pub edges_fwd: Vec<Vec<EdgeIndex<Idx>>>,
    // Represents the downward graph G↓
    pub edges_bwd: Vec<Vec<EdgeIndex<Idx>>>,

    g: &'a Graph,
}

impl<'a> OverlayGraph<'a> {
    pub(crate) fn new(
        edges_fwd: Vec<Vec<EdgeIndex>>,
        edges_bwd: Vec<Vec<EdgeIndex>>,
        graph: &'a Graph,
    ) -> Self {
        OverlayGraph {
            edges_fwd,
            edges_bwd,
            g: graph,
        }
    }

    /// Returns the underlying road graph.
    pub fn road_graph(&self) -> &Graph {
        self.g
    }

    pub fn edge(&self, edge_idx: EdgeIndex) -> &Edge<DefaultIdx> {
        &self.g.edges[edge_idx.index()]
    }

    pub fn edges_fwd(&self, node: NodeIndex) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.edges_fwd[node.index()]
            .iter()
            .map(|edge_idx| (*edge_idx, &self.g.edges[edge_idx.index()]))
    }

    pub fn edges_bwd(&self, node: NodeIndex) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.edges_bwd[node.index()]
            .iter()
            .map(|edge_idx| (*edge_idx, &self.g.edges[edge_idx.index()]))
    }

    /// Recursively unpacks shortcut edges. Used to reconstruct the original path after the shortest path calculation.
    pub(crate) fn unpack_edge(&self, edge_idx: EdgeIndex) -> Vec<EdgeIndex> {
        let edge = &self.g.edges[edge_idx.index()];
        let mut unpacked = Vec::new();
        match edge.shortcut_for {
            Some([first, second]) => {
                unpacked.append(&mut self.unpack_edge(first));
                unpacked.append(&mut self.unpack_edge(second));
            }
            None => unpacked.push(edge_idx),
        }
        unpacked
    }

    pub fn print_info(&self) {
        println!(
            "SearchGraph:\t#Nodes: {}, #Edges: {}",
            self.edges_fwd.len(),
            self.edges_fwd.iter().flatten().count()
        );
    }
}

impl<'a> Display for OverlayGraph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SearchGraph: #Edges: {}, #Nodes: {}",
            self.edges_fwd.iter().flatten().count(),
            self.edges_fwd.len()
        )?;
        for (node, edges) in self.edges_fwd.iter().enumerate() {
            write!(f, "  {}:", node)?;
            for edge_idx in edges {
                write!(
                    f,
                    " {}->{} ",
                    self.g.edges[edge_idx.index()].source.index(),
                    self.g.edges[edge_idx.index()].target.index()
                )?;
            }
            writeln!(f)?;
        }

        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{edge, graph::*};
    use crate::{node_contraction::NodeContractor, util::test_graphs::generate_simple_graph};

    #[test]
    fn test_unpacking_edges() {
        //           B
        //           |
        // E -> A -> C
        //      |  /
        //      D
        let mut g = Graph::<DefaultIdx>::new();

        let a = g.add_node(Node::new(0, 0.0, 0.0));
        let b = g.add_node(Node::new(1, 0.0, 0.0));
        let c = g.add_node(Node::new(2, 0.0, 0.0));
        let d = g.add_node(Node::new(3, 0.0, 0.0));
        let e = g.add_node(Node::new(4, 0.0, 0.0));

        let ac = g.add_edge(edge!(a => c, 1.0));
        g.add_edge(edge!(a => d, 1.0));
        let ea = g.add_edge(edge!(e => a, 1.0));
        g.add_edges(edge!(c, b, 1.0));
        g.add_edges(edge!(c, d, 1.0));

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

        let unpacked_edges = overlay_graph.unpack_edge(7.into());
        assert_eq!(vec![ea, ac], unpacked_edges);
    }

    #[test]
    fn test_print_graph() {
        //           B
        //           |
        // E -> A -> C
        //      |  /
        //      D
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

        let overlay_graph = contractor.run_with_order(&node_order);

        println!("{}", overlay_graph);
    }
}
