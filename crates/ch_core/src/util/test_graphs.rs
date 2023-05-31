use crate::{
    edge,
    graph::{Graph, Node},
};

pub fn generate_complex_graph() -> Graph {
    let mut graph = Graph::new();

    let a = graph.add_node(Node::new(0, 6.0, 2.0));
    let b = graph.add_node(Node::new(1, 3.0, 3.0));
    let c = graph.add_node(Node::new(2, 4.0, 6.0));
    let d = graph.add_node(Node::new(3, 2.0, 7.0));
    let e = graph.add_node(Node::new(4, 3.0, 10.0));
    let f = graph.add_node(Node::new(5, 2.0, 13.0));
    let g = graph.add_node(Node::new(6, 7.0, 15.0));
    let h = graph.add_node(Node::new(7, 5.0, 12.0));
    let i = graph.add_node(Node::new(8, 7.0, 11.0));
    let j = graph.add_node(Node::new(9, 5.0, 9.0));
    let k = graph.add_node(Node::new(10, 7.0, 7.0));

    graph.add_edges(edge!(a, b, 3.0)); // A <=> B
    graph.add_edges(edge!(a, c, 5.0)); // A <=> C
    graph.add_edges(edge!(a, k, 3.0)); // A <=> K

    graph.add_edges(edge!(b, d, 5.0)); // B <=> D
    graph.add_edges(edge!(b, c, 3.0)); // B <=> C

    graph.add_edges(edge!(c, d, 2.0)); // C <=> D
    graph.add_edges(edge!(c, j, 2.0)); // C <=> J

    graph.add_edges(edge!(d, j, 4.0)); // D <=> J
    graph.add_edges(edge!(d, e, 7.0)); // D <=> E

    graph.add_edges(edge!(e, j, 3.0)); // E <=> J
    graph.add_edges(edge!(e, f, 6.0)); // E <=> F

    graph.add_edges(edge!(f, h, 2.0)); // F <=> H
    graph.add_edges(edge!(f, g, 4.0)); // F <=> G

    graph.add_edges(edge!(g, h, 3.0)); // G <=> H
    graph.add_edges(edge!(g, i, 5.0)); // G <=> I

    graph.add_edges(edge!(h, i, 3.0)); // H <=> I
    graph.add_edges(edge!(h, j, 2.0)); // H <=> J

    graph.add_edges(edge!(i, j, 4.0)); // I <=> J
    graph.add_edges(edge!(i, k, 6.0)); // I <=> K

    graph.add_edges(edge!(j, k, 3.0)); // J <=> K

    graph
}

pub fn generate_simple_graph() -> Graph {
    //           B
    //           |
    // E -> A -> C
    //      |  /
    //      D
    let mut g = Graph::new();

    let a = g.add_node(Node::new(0, 0.0, 0.0));
    let b = g.add_node(Node::new(1, 0.0, 0.0));
    let c = g.add_node(Node::new(2, 0.0, 0.0));
    let d = g.add_node(Node::new(3, 0.0, 0.0));
    let e = g.add_node(Node::new(4, 0.0, 0.0));

    g.add_edge(edge!(a => c, 1.0));
    g.add_edge(edge!(a => d, 1.0));
    g.add_edge(edge!(e => a, 1.0));
    g.add_edges(edge!(c, b, 1.0));
    g.add_edges(edge!(c, d, 1.0));

    g
}

pub fn graph_vaterstetten() -> Graph {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../osm_reader/test_data/vaterstetten_pp.osm.pbf");

    Graph::from_pbf(&path).unwrap()
}

#[cfg(test)]
pub fn overlay_graph_vaterstetten() -> crate::overlay_graph::OverlayGraph {
    let mut g = graph_vaterstetten();
    let mut contractor = crate::node_contraction::NodeContractor::new(&mut g);
    contractor.run()
}

pub fn graph_saarland() -> Graph {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../osm_reader/test_data/saarland_pp.osm.pbf");

    Graph::from_pbf(&path).unwrap()
}
