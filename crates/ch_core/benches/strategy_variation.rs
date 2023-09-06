use std::path::Path;

use ch_core::{
    contraction_strategy::{ContractionStrategy, UpdateStrategy},
    graph::Graph,
    node_contraction::{ContractionParams, NodeContractor, PriorityParams},
    util::test_graphs::graph_saarland,
};

fn main() {
    env_logger::init();

    let g = if let Some(path) = std::env::args().nth(1) {
        Graph::from_pbf_with_simplification(Path::new(&path)).expect("Invalid path")
    } else {
        graph_saarland()
    };

    // Only Self Update
    let self_update = UpdateStrategy::default().set_update_local(false);
    let self_update_p = UpdateStrategy::default()
        .set_update_local(false)
        .set_periodic_updates(true);

    // Only Neighbors Update
    let neighbors_update = UpdateStrategy::default().set_update_jit(false);
    let neighbors_update_p = UpdateStrategy::default()
        .set_update_jit(false)
        .set_periodic_updates(true);

    // Both Self and Neighbors Update
    let combined_update = UpdateStrategy::default();
    let combined_update_p = UpdateStrategy::default().set_periodic_updates(true);

    let strats: Vec<(&str, UpdateStrategy)> = vec![
        ("self_update", self_update),
        ("self_update_p", self_update_p),
        ("neighbors_update", neighbors_update),
        ("neighbors_update_p", neighbors_update_p),
        ("combined_update", combined_update),
        ("combined_update_p", combined_update_p),
    ];

    // let g = graph_saarland();

    for (name, strat) in strats {
        let mut g = g.clone();

        let params = ContractionParams::new().priority_params(PriorityParams::new(190, 120, 1, 70));
        let mut contractor = NodeContractor::new_with_params(&mut g, params);

        println!("Strategy: {name}");
        contractor.run_with_strategy(ContractionStrategy::LazyUpdate(strat));
        // println!("{}", contractor.stats());
    }
}
