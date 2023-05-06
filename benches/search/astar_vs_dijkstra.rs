use ch_core::{
    graph::{node_index, Edge, Graph, Node},
    search::astar::AStar,
    search::dijkstra::Dijkstra,
    util::math::straight_line,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

fn gen_rand_graph(number_nodes: usize) -> Graph {
    let mut rng = rand::thread_rng();

    // 2.5 edges per node on average
    let number_edges: usize = (number_nodes as f32 * 2.5) as usize;

    let mut g = Graph::with_capacity(number_nodes, number_edges);

    for i in 0..number_nodes {
        g.add_node(Node::new(i, 0.0, 0.0));
    }

    for _ in 0..number_edges {
        let source = rng.gen_range(0..number_nodes);
        let target = rng.gen_range(0..number_nodes);
        let weight = rng.gen_range(1..100) as f64;
        g.add_edge(Edge::new(node_index(source), node_index(target), weight));
    }

    g
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut graphs: Vec<Graph> = [1000, 50_000, 100_000]
        .iter()
        .map(|i| gen_rand_graph(*i))
        .collect();

    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../crates/osm_reader/data/saarland_pp2.osm.pbf");
    graphs.push(Graph::from_pbf(&path).unwrap());

    let mut group = c.benchmark_group("astar_vs_dijkstra");
    let mut rng = rand::thread_rng();
    for graph in graphs {
        group.bench_with_input(
            BenchmarkId::new("Dijkstra", graph.nodes.len()),
            &graph,
            |b, g| {
                let src = rng.gen_range(0..g.nodes.len());
                let dst = rng.gen_range(0..g.nodes.len());
                let mut dijkstra = Dijkstra::new(g);
                b.iter(|| {
                    dijkstra.search(node_index(src), node_index(dst));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("AStar", graph.nodes.len()),
            &graph,
            |b, g| {
                let src = rng.gen_range(0..g.nodes.len());
                let dst = rng.gen_range(0..g.nodes.len());
                let mut astar = AStar::new(g);
                b.iter(|| {
                    astar.search(node_index(src), node_index(dst), straight_line);
                });
            },
        );
    }
    group.finish();
}
