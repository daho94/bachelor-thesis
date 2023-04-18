use std::path::Path;

use ch_core::{
    dijkstra::Dijkstra,
    graph::{Edge, Graph, GraphBuilder, Node},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::prelude::*;

criterion_group!(benches, dijkstra);
criterion_main!(benches);

fn gen_rand_graph(number_nodes: usize) -> Graph {
    let nodes = (0..number_nodes).map(|i| Node::new(i, 0.0, 0.0)).collect();

    let mut rng = rand::thread_rng();

    // 2.5 edges per node on average
    let number_edges: usize = (number_nodes as f32 * 2.5) as usize;

    let mut edges = Vec::with_capacity(number_edges);
    for _ in 0..number_edges {
        let src = rng.gen_range(0..number_nodes);
        let dst = rng.gen_range(0..number_nodes);
        let weight = rng.gen_range(1..100) as f64;
        edges.push(Edge::new(src, dst, weight));
    }

    GraphBuilder::new()
        .add_nodes(nodes)
        .add_edges(edges)
        .build()
}

// fn from_elem(c: &mut Criterion) {
//     static KB: usize = 1024;

//     let mut group = c.benchmark_group("from_elem");
//     for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
//         group.throughput(criterion::Throughput::Bytes(*size as u64));
//         group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
//             b.iter(|| std::iter::repeat(0u8).take(size).collect::<Vec<_>>());
//         });
//     }
//     group.finish();
// }

fn dijkstra(c: &mut Criterion) {
    let mut graphs: Vec<Graph> = (15..16).map(|i| gen_rand_graph(2usize.pow(i))).collect();
    graphs.push(
        Graph::from_pbf(Path::new(
            r"F:\Dev\uni\BA\bachelor_thesis\crates\osm_reader\data\saarland_pp2.osm.pbf",
        ))
        .unwrap(),
    );

    let mut group = c.benchmark_group("dijkstra");
    let mut rng = rand::thread_rng();
    for graph in graphs {
        group.throughput(criterion::Throughput::Elements(graph.nodes.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(graph.nodes.len()),
            &graph,
            |b, g| {
                let src = g.nodes[rng.gen_range(0..g.nodes.len())].id;
                let dst = g.nodes[rng.gen_range(0..g.nodes.len())].id;
                let mut dijkstra = Dijkstra::new(g);
                b.iter(|| {
                    dijkstra.search(src, dst);
                });
            },
        );
    }
    group.finish();
}
