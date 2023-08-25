use std::path::Path;

use ch_core::{
    graph::{node_index, Graph},
    node_contraction::{ContractionParams, NodeContractor, PriorityParams},
    search::ch_search::CHSearch,
    util::test_graphs::graph_saarland,
};
use rand::{rngs::StdRng, Rng};

// Test different combinations of priority terms
// The used coefficients were calculated beforehand `raster_search`
fn main() {
    env_logger::init();
    const ITERATIONS: usize = 1_000;

    let g = if let Some(path) = std::env::args().nth(1) {
        Graph::from_pbf_with_simplification(Path::new(&path)).expect("Invalid path")
    } else {
        graph_saarland()
    };

    let num_nodes = g.nodes.len();

    let queries = {
        let mut queries = Vec::new();
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(137);

        for _ in 0..ITERATIONS {
            // Generate random start node
            let source = node_index(rng.gen_range(0..num_nodes));
            // Find target
            let target = node_index(rng.gen_range(0..num_nodes));

            queries.push((source, target));
        }

        queries
    };

    // Parameter config
    // let e = PriorityParams::new(501, 0, 0, 0);
    // let ec = PriorityParams::new(501, 401, 0, 0);
    // let ecs = PriorityParams::new(501, 401, 1, 0);
    // let ecso = PriorityParams::new(501, 401, 1, 70);
    let e = PriorityParams::new(190, 0, 0, 0);
    let ec = PriorityParams::new(190, 120, 0, 0);
    let ecs = PriorityParams::new(190, 120, 1, 0);
    let ecso = PriorityParams::new(190, 120, 1, 70);

    let configs = vec![e, ec, ecs, ecso];

    let mut construction_data = Vec::new();
    let mut query_data = vec![vec![]; configs.len()];

    for (c, config) in configs.iter().enumerate() {
        let mut g = g.clone();
        let params = ContractionParams::new().priority_params(*config);
        let mut contractor = NodeContractor::new_with_params(&mut g, params);

        let overlay_graph = contractor.run();
        construction_data.push((
            contractor.stats().node_ordering_time,
            contractor.stats().contraction_time,
        ));

        let mut ch = CHSearch::new(&overlay_graph);

        for (start, target) in &queries {
            ch.search(*start, *target);

            query_data[c].push((
                ch.stats.duration.unwrap().as_micros() as f64,
                ch.stats.nodes_settled as f64,
            ));
        }
    }

    let mut query_stats = Vec::new();

    for i in 0..configs.len() {
        let avg_time = query_data[i].iter().map(|(t, _)| t).sum::<f64>() / ITERATIONS as f64;
        let avg_nodes = query_data[i].iter().map(|(_, n)| n).sum::<f64>() / ITERATIONS as f64;

        query_stats.push((avg_time, avg_nodes));
    }

    println!("Query stats: {:?}", query_stats);
    println!("Construction stats: {:?}", construction_data);
}
