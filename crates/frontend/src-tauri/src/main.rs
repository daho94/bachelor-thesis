#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::Path;
use std::time::Instant;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn create_graph_from_pbf(path: &str) -> Vec<[[f64; 2]; 2]> {
    println!("Creating graph from file '{}'...", path);
    let start = Instant::now();
    if let Ok(graph) = osm_reader::RoadGraph::from_pbf(Path::new(path)) {
        let duration = start.elapsed();
        println!(
            "Created Graph with {} nodes and {} edges in {:?}!",
            graph.get_nodes().len(),
            graph.get_edges().len(),
            duration
        );
        graph
            .get_edges()
            .iter()
            .map(|edge| {
                let from = graph.get_nodes().get(&edge.0).unwrap();
                let to = graph.get_nodes().get(&edge.1).unwrap();
                [*from, *to]
            })
            .collect()
    } else {
        vec![]
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_graph_from_pbf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
