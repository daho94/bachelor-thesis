use crate::{
    edge,
    graph::{Graph, Node},
};

pub fn generate_complex_graph() -> Graph {
    let mut g = Graph::new();

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

    g
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
