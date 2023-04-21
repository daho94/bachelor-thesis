use criterion::{black_box, criterion_group, criterion_main, Criterion};
use osm_reader::RoadGraph;

criterion_group!(benches, parse_saarland);
criterion_main!(benches);

fn parse_saarland(c: &mut Criterion) {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../crates/osm_reader/data/saarland_pp2.osm.pbf");

    c.bench_function("parse_saarland", |b| {
        b.iter(|| {
            let _ = RoadGraph::from_pbf(&black_box(&path));
        })
    });
}
