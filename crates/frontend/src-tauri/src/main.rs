#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::Path;
use std::time::Instant;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn create_graph_from_pbf(path: &str) -> String {
    println!("Creating graph from file '{}'...", path);
    let start = Instant::now();
    if let Ok(graph) = osm_reader::RoadGraph::from_pbf(Path::new(path)) {
        let duration = start.elapsed();
        format!(
            "Created Graph with {} nodes in {:?}!",
            graph.get_nodes().len(),
            duration
        )
    } else {
        format!("Failed to create graph from file '{}'!", path)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_graph_from_pbf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
