use std::path::Path;

use ch_core::{
    graph::{node_index, Graph},
    node_contraction::{ContractionParams, NodeContractor, PriorityParams},
    search::ch_search::CHSearch,
    util::test_graphs::graph_saarland,
};
use indicatif::ProgressBar;
use rand::{rngs::StdRng, Rng};

fn main() {
    env_logger::init();
    const ITERATIONS: usize = 1_000;
    let mut g = if let Some(path) = std::env::args().nth(1) {
        Graph::from_pbf_with_simplification(Path::new(&path)).expect("Invalid path")
    } else {
        graph_saarland()
    };

    let num_nodes = g.nodes.len();

    let params = ContractionParams::new().priority_params(PriorityParams::new(190, 120, 1, 70));

    let mut contractor = NodeContractor::new_with_params(&mut g, params);
    let overlay_graph = contractor.run();

    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(187);

    let mut ch = CHSearch::new(&overlay_graph);

    // (time, nodes_settled, nodes_stalled)
    let mut stall_data = Vec::new();
    let mut no_stall_data = Vec::new();

    let pb = ProgressBar::new(ITERATIONS as u64);

    for _ in 0..ITERATIONS {
        let source = node_index(rng.gen_range(0..num_nodes));
        let target = node_index(rng.gen_range(0..num_nodes));

        ch.search(source, target);
        stall_data.push((
            ch.stats.duration.unwrap().as_micros() as f64,
            ch.stats.nodes_settled,
            ch.nodes_stalled,
        ));

        ch.search_without_stalling(source, target);
        no_stall_data.push((
            ch.stats.duration.unwrap().as_micros() as f64,
            ch.stats.nodes_settled,
            ch.nodes_stalled,
        ));

        pb.inc(1);
    }

    // Calculate stats
    let (avg_time, avg_nodes_settled, avg_nodes_stalled) = calc_stats(&no_stall_data);
    println!("_____No stalling_____");
    println!(
        "avg time: {:.2}ys, avg nodes settled: {:.2}, avg nodes stalled: {:.2}",
        avg_time, avg_nodes_settled, avg_nodes_stalled
    );

    let (avg_time, avg_nodes_settled, avg_nodes_stalled) = calc_stats(&stall_data);
    println!("_____Stalling_____");
    println!(
        "avg time: {:.2}ys, avg nodes settled: {:.2}, avg nodes stalled: {:.2}",
        avg_time, avg_nodes_settled, avg_nodes_stalled
    );
}

fn calc_stats(data: &[(f64, usize, usize)]) -> (f64, f64, f64) {
    let mut sum_time = 0.0;
    let mut sum_nodes_settled = 0.0;
    let mut sum_nodes_stalled = 0.0;

    for (time, nodes_settled, nodes_stalled) in data {
        sum_time += time;
        sum_nodes_settled += *nodes_settled as f64;
        sum_nodes_stalled += *nodes_stalled as f64;
    }

    let avg_time = sum_time / data.len() as f64;
    let avg_nodes_settled = sum_nodes_settled / data.len() as f64;
    let avg_nodes_stalled = sum_nodes_stalled / data.len() as f64;

    (avg_time, avg_nodes_settled, avg_nodes_stalled)
}
