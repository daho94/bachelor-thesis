use std::path::Path;

use ch_core::{dijkstra::Dijkstra, graph::Graph};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

criterion_group!(benches, criterion_benchmark, from_elem);
criterion_main!(benches);

pub fn criterion_benchmark(c: &mut Criterion) {
    let vaterstetten: Graph = Graph::from_pbf(Path::new(
        "../crates/osm_reader/test_data/vaterstetten_pp.osm.pbf",
    ))
    .unwrap();

    c.bench_with_input(
        BenchmarkId::new("dijkstra_on_input_graph", stringify!(vaterstetten)),
        &vaterstetten,
        |b, g| {
            b.iter(|| {
                let mut dijkstra = Dijkstra::new(g);
                dijkstra.search(659848261, 29272508);
            })
        },
    );
}
fn from_elem(c: &mut Criterion) {
    static KB: usize = 1024;

    let mut group = c.benchmark_group("from_elem");
    for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
        group.throughput(criterion::Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| std::iter::repeat(0u8).take(size).collect::<Vec<_>>());
        });
    }
    group.finish();
}
