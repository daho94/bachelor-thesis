use std::path::Path;

use ch_core::{
    contraction_params::ContractionParams,
    graph::{node_index, Graph},
    node_contraction::NodeContractor,
    search::ch_search::CHSearch,
    util::test_graphs::graph_saarland,
};

fn main() {
    env_logger::init();
    const ITERATIONS: usize = 1_000;

    let g = if let Some(path) = std::env::args().nth(1) {
        Graph::from_pbf_with_simplification(Path::new(&path)).expect("Invalid path")
    } else {
        graph_saarland()
    };

    let configs = vec![(0, 500), (50, 500), (500, 500)];

    let mut construction_data = Vec::new();
    let mut query_data = vec![vec![]; configs.len()];

    for (c, (limit, initial_limit)) in configs.iter().enumerate() {
        println!("Trying config {}", c + 1);
        let mut g = g.clone();
        let params = ContractionParams::new()
            .witness_search_limit(*limit)
            .witness_search_initial_limit(*initial_limit);

        let mut contractor = NodeContractor::new_with_params(&mut g, params);
        let overlay_graph = contractor.run();
        construction_data.push((
            contractor.stats().node_ordering_time,
            contractor.stats().contraction_time,
        ));

        let mut ch = CHSearch::new(&overlay_graph);

        for _ in 0..ITERATIONS {
            ch.search(node_index(30), node_index(200));

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
