//! The overlay graph is a representation of the graph after running the node contraction process.
use std::{fmt::Display, path::PathBuf};

use anyhow::Context;

use csv::Writer;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::graph::{DefaultIdx, Edge, EdgeIndex, Graph, Node, NodeIndex};

/// Representation of the graph after running the node contraction process. The
/// graph contains two adjancency lists, one for the upward graph G↑ and one for
/// the downward graph G↓.
#[derive(Serialize, Deserialize)]
pub struct OverlayGraph<Idx = DefaultIdx> {
    // Represents the upward graph G↑
    pub edges_fwd: Vec<Vec<EdgeIndex<Idx>>>,
    // Represents the downward graph G↓
    pub edges_bwd: Vec<Vec<EdgeIndex<Idx>>>,

    pub shortcuts: FxHashMap<EdgeIndex, [EdgeIndex<Idx>; 2]>,

    pub node_order: Vec<usize>,

    g: Graph<Idx>,
}

impl OverlayGraph {
    pub(crate) fn new(
        edges_fwd: Vec<Vec<EdgeIndex>>,
        edges_bwd: Vec<Vec<EdgeIndex>>,
        graph: Graph,
        shortcuts: FxHashMap<EdgeIndex, [EdgeIndex; 2]>,
        node_ranks: Vec<usize>,
    ) -> Self {
        OverlayGraph {
            edges_fwd,
            edges_bwd,
            g: graph,
            shortcuts,
            node_order: node_ranks,
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.g.nodes()
    }

    pub fn encode(&self, path: impl Into<PathBuf>) -> anyhow::Result<usize> {
        let mut file = std::fs::File::create(path.into())?;
        let bytes_written =
            bincode::serde::encode_into_std_write(self, &mut file, bincode::config::standard())?;

        Ok(bytes_written)
    }

    pub fn from_decode(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path.into()).context("Failed to read file")?;
        let mut reader = std::io::BufReader::new(file);

        let overlay_graph =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard())
                .context("Failed to decode file")?;

        Ok(overlay_graph)
    }

    /// Returns the underlying road graph.
    pub fn road_graph(&self) -> &Graph {
        &self.g
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
        let mut unpacked = Vec::new();

        match self.shortcuts.get(&edge_idx) {
            Some([incoming, outgoing]) => {
                unpacked.append(&mut self.unpack_edge(*incoming));
                unpacked.append(&mut self.unpack_edge(*outgoing));
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

    pub fn export_csv(&self) -> anyhow::Result<()> {
        self.g.export_csv()?;

        let mut wtr = Writer::from_path("edges_fwd.csv")?;
        wtr.write_record(["source", "target_edge"])?;

        // Export edges_fwd
        for (idx_from, edges) in self.edges_fwd.iter().enumerate() {
            for idx_to in edges {
                wtr.write_record(&[idx_from.to_string(), idx_to.index().to_string()])?;
            }
        }
        wtr.flush()?;

        let mut wtr = Writer::from_path("edges_bwd.csv")?;
        wtr.write_record(["source", "target_edge"])?;

        // Export edges_fwd
        for (idx_from, edges) in self.edges_bwd.iter().enumerate() {
            for idx_to in edges {
                wtr.write_record(&[idx_from.to_string(), idx_to.index().to_string()])?;
            }
        }
        wtr.flush()?;

        let mut wtr = csv::Writer::from_path("shortcuts.csv")?;
        wtr.write_record(["id", "in", "out"])?;

        for (edge_idx, replaces) in self.shortcuts.iter() {
            wtr.write_record(&[
                edge_idx.index().to_string(),
                replaces[0].index().to_string(),
                replaces[1].index().to_string(),
            ])?;
        }
        wtr.flush()?;

        Ok(())
    }

    pub fn from_csv<P: Into<PathBuf>>(
        g: Graph,
        csv_shortcuts: P,
        csv_fwd: P,
        csv_bwd: P,
    ) -> anyhow::Result<OverlayGraph> {
        let mut edges_fwd = vec![Vec::new(); g.nodes.len()];
        let mut edges_bwd = vec![Vec::new(); g.nodes.len()];

        let mut rdr = csv::Reader::from_path(csv_fwd.into())?;
        for result in rdr.records() {
            let record = result?;
            let source = record[0].parse::<usize>()?;
            let target = record[1].parse::<usize>()?;

            edges_fwd[source].push(EdgeIndex::new(target));
        }

        let mut rdr = csv::Reader::from_path(csv_bwd.into())?;
        for result in rdr.records() {
            let record = result?;
            let source = record[0].parse::<usize>()?;
            let target = record[1].parse::<usize>()?;

            edges_bwd[source].push(EdgeIndex::new(target));
        }

        let mut rdr = csv::Reader::from_path(csv_shortcuts.into())?;
        let mut shortcuts = FxHashMap::default();
        for result in rdr.records() {
            let record = result?;
            let id = record[0].parse::<usize>()?;
            let in_ = record[1].parse::<usize>()?;
            let out = record[2].parse::<usize>()?;

            shortcuts.insert(
                EdgeIndex::new(id),
                [EdgeIndex::new(in_), EdgeIndex::new(out)],
            );
        }

        Ok(OverlayGraph::new(
            edges_fwd,
            edges_bwd,
            g,
            shortcuts,
            Default::default(),
        ))
    }
}

impl Display for OverlayGraph {
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
    use crate::{
        edge, graph::*, overlay_graph::OverlayGraph, util::test_graphs::overlay_graph_vaterstetten,
    };
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

    #[test]
    fn export_csv() {
        let mut g = generate_simple_graph();

        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];
        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run_with_order(&node_order);

        let res = overlay_graph.export_csv();

        assert!(res.is_ok());
    }

    #[test]
    fn import_csv() {
        let mut g = generate_simple_graph();

        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];
        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run_with_order(&node_order);

        overlay_graph.export_csv().unwrap();

        let h = overlay_graph.road_graph().clone();

        let overlay_graph_imported =
            OverlayGraph::from_csv(h, "shortcuts.csv", "edges_fwd.csv", "edges_bwd.csv").unwrap();

        assert_eq!(overlay_graph.edges_bwd, overlay_graph_imported.edges_bwd);
        assert_eq!(overlay_graph.edges_fwd, overlay_graph_imported.edges_fwd);
        assert_eq!(overlay_graph.shortcuts, overlay_graph_imported.shortcuts);
    }

    #[test]
    fn test_encode_and_decode_simple_graph() {
        let mut g = generate_simple_graph();

        let node_order = vec![
            node_index(0),
            node_index(4),
            node_index(3),
            node_index(2),
            node_index(1),
        ];
        let mut contractor = NodeContractor::new(&mut g);

        let overlay_graph = contractor.run_with_order(&node_order);

        // Encode
        let _ = overlay_graph.encode("simple_graph.bin");

        // Decode
        let overlay_graph_imported =
            OverlayGraph::from_decode("simple_graph.bin").expect("Failed to decode file");

        assert_eq!(overlay_graph.edges_bwd, overlay_graph_imported.edges_bwd);
        assert_eq!(overlay_graph.edges_fwd, overlay_graph_imported.edges_fwd);
        assert_eq!(overlay_graph.g.edges_in, overlay_graph_imported.g.edges_in);
        assert_eq!(
            overlay_graph.g.edges_out,
            overlay_graph_imported.g.edges_out
        );
        assert_eq!(overlay_graph.shortcuts, overlay_graph_imported.shortcuts);
    }

    #[test]
    fn encode_and_decode_vaterstetten() {
        let overlay_graph = overlay_graph_vaterstetten();

        let _ = overlay_graph.encode("vaterstetten.bin");

        let overlay_graph_imported = OverlayGraph::from_decode("vaterstetten.bin").unwrap();

        assert_eq!(overlay_graph.edges_bwd, overlay_graph_imported.edges_bwd);
        assert_eq!(overlay_graph.edges_fwd, overlay_graph_imported.edges_fwd);
        assert_eq!(overlay_graph.g.edges_in, overlay_graph_imported.g.edges_in);
        assert_eq!(
            overlay_graph.g.edges_out,
            overlay_graph_imported.g.edges_out
        );
        assert_eq!(overlay_graph.shortcuts, overlay_graph_imported.shortcuts);
    }
}
