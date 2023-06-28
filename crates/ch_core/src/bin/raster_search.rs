use std::time::Instant;

use ch_core::{
    graph::node_index,
    node_contraction::{NodeContractor, PriorityParams},
    search::bidir_search::BiDirSearch,
    util::test_graphs::{graph_saarland, graph_vaterstetten},
};

use rand::prelude::*;

fn main() {
    env_logger::init();

    let g = graph_saarland();

    let num_nodes = g.nodes().count();

    let mut rng = thread_rng();

    let mut now;

    let mut min_params_aggressive = None;
    let mut min_time_aggressive = std::u128::MAX;

    // let min_params_economicalx;
    // let min_time_economical = std::u128::MAX;

    for edge_difference_coeff in (1..=501).step_by(100) {
        for contracted_neighbors_coeff in (1..=501).step_by(100) {
            for search_space_coeff in (1..=7).step_by(1) {
                let params = PriorityParams::new(
                    edge_difference_coeff,
                    contracted_neighbors_coeff,
                    search_space_coeff,
                );

                let mut g = g.clone();

                let mut contractor = NodeContractor::new_with_priority_params(&mut g, params);

                now = Instant::now();

                let overlay_graph = contractor.run();

                let time_construction = now.elapsed().as_millis();

                // Do 1000 random queries
                let mut bidir_search = BiDirSearch::new(&overlay_graph);

                let mut time_total = 0;

                let mut successfull_searches = 0;
                for _ in 0..1000 {
                    // Random start and end node
                    let start: usize = (rng.gen::<f32>() * (num_nodes as f32 - 1.0)) as usize;
                    let end: usize = (rng.gen::<f32>() * (num_nodes as f32 - 1.0)) as usize;

                    now = Instant::now();
                    if bidir_search
                        .search(node_index(start), node_index(end))
                        .is_some()
                    {
                        successfull_searches += 1;
                        time_total += now.elapsed().as_micros();
                    };
                }

                let time_avg = time_total / successfull_searches;

                if time_avg < min_time_aggressive {
                    min_time_aggressive = time_avg;
                    min_params_aggressive = Some(params);
                }

                println!(
                    "ED: {} CN: {} SS: {}",
                    edge_difference_coeff, contracted_neighbors_coeff, search_space_coeff
                );
                println!("Construction time: {} ms", time_construction);
                println!("Avg. query time: {} μs", time_avg);
            }
        }
    }

    println!(
        "Best aggressive params: {:#?} with averagy query time: {} μs",
        min_params_aggressive.unwrap(),
        min_time_aggressive
    );
}
