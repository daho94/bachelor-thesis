use crate::graph::{DefaultIdx, Edge, EdgeIndex, NodeIndex};

pub struct SearchGraph<Idx = DefaultIdx> {
    // Represents the upward graph G↑
    pub edges_fwd: Vec<Vec<EdgeIndex<Idx>>>,
    // Represents the downward graph G↓
    pub edges_bwd: Vec<Vec<EdgeIndex<Idx>>>,
    pub edges: Vec<Edge>,
}

impl SearchGraph {
    pub fn new(num_nodes: usize) -> Self {
        let edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); num_nodes];
        let edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); num_nodes];

        SearchGraph {
            edges_fwd,
            edges_bwd,
            edges: Vec::new(),
        }
    }

    pub fn edges_fwd(&self, node: NodeIndex) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.edges_fwd[node.index()]
            .iter()
            .map(|edge_idx| (*edge_idx, &self.edges[edge_idx.index()]))
    }

    pub fn edges_bwd(&self, node: NodeIndex) -> impl Iterator<Item = (EdgeIndex, &Edge)> {
        self.edges_bwd[node.index()]
            .iter()
            .map(|edge_idx| (*edge_idx, &self.edges[edge_idx.index()]))
    }

    pub fn unpack_edge(&self, edge_idx: EdgeIndex) -> Vec<EdgeIndex> {
        let edge = &self.edges[edge_idx.index()];
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

    pub fn with_capacity(num_nodes: usize, num_edges: usize) -> Self {
        let edges_fwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); num_nodes];
        let edges_bwd: Vec<Vec<EdgeIndex>> = vec![Vec::new(); num_nodes];

        SearchGraph {
            edges_fwd,
            edges_bwd,
            edges: Vec::with_capacity(num_edges),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::node_contraction::contract_nodes_with_order;
    use crate::{edge, graph::*};

    use super::SearchGraph;

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

        let search_graph = contract_nodes_with_order(&mut g, &node_order);

        let unpacked_edges = search_graph.unpack_edge(7.into());
        assert_eq!(vec![ea, ac], unpacked_edges);
    }
}
