use criterion::{black_box, criterion_group, criterion_main, Criterion};
use osm_reader::RoadGraph;

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = parse_saarland, parse_bavaria
}
criterion_main!(benches);

fn parse_saarland(c: &mut Criterion) {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../crates/osm_reader/data/saarland.osm.pbf");

    c.bench_function("parse_saarland", |b| {
        b.iter(|| {
            let _ = RoadGraph::from_pbf(black_box(&path));
        })
    });
}

fn parse_bavaria(c: &mut Criterion) {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../crates/osm_reader/data/bayern_pp.osm.pbf");
    c.bench_function("parse_bavaria", |b| {
        b.iter(|| {
            let _ = RoadGraph::from_pbf(black_box(&path));
        })
    });
}
