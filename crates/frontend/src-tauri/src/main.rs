#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ch_core::search::dijkstra::Dijkstra;
use log::info;
use serde::Serialize;
use std::time::Instant;
use std::{path::Path, sync::Mutex};
use tauri::{Manager, State};
use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget};

type Graph = Mutex<ch_core::graph::Graph>;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn create_graph_from_pbf(graph: State<'_, Graph>, path: &str) -> Result<(), String> {
    let start = Instant::now();
    if let Ok(g) = ch_core::graph::Graph::from_pbf(Path::new(path)) {
        let duration = start.elapsed();
        info!(
            "Created Graph with {} nodes and {} edges in {:?}!",
            g.nodes.len(),
            g.edges.len(),
            duration
        );
        let mut graph = graph.lock().unwrap();
        *graph = g;

        Ok(())
    } else {
        Err("Could not create graph!".to_string())
    }
}

#[tauri::command]
fn get_edges(graph: State<'_, Graph>) -> Vec<[[f64; 2]; 2]> {
    let graph = &graph.lock().unwrap();
    graph
        .edges
        .iter()
        .map(|edge| {
            let from = &graph
                .nodes
                .iter()
                .find(|n| n.id == edge.from)
                .map(|n| [n.lat, n.lon])
                .unwrap();
            let to = &graph
                .nodes
                .iter()
                .find(|n| n.id == edge.to)
                .map(|n| [n.lat, n.lon])
                .unwrap();
            [*from, *to]
        })
        .collect()
}

#[tauri::command]
fn get_nodes(graph: State<'_, Graph>) -> Vec<[f64; 3]> {
    let graph = &graph.lock().unwrap();
    graph
        .nodes
        .iter()
        .map(|node| [node.id as f64, node.lon, node.lat])
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PathResult {
    path: Vec<[f64; 2]>,
    weight: f64,
    duration: f64,
    nodes_settled: usize,
}

impl PathResult {
    fn new(path: Vec<[f64; 2]>, weight: f64, duration: f64, nodes_settled: usize) -> Self {
        PathResult {
            path,
            weight,
            duration,
            nodes_settled,
        }
    }
}

#[tauri::command]
fn calc_path(graph: State<'_, Graph>, src_coords: [f64; 2], dst_coords: [f64; 2]) -> PathResult {
    let graph = graph.lock().unwrap();

    let mut d = Dijkstra::new(&graph);

    let src_id = dbg!(p2p_matching(&graph.nodes, src_coords));
    let dst_id = dbg!(p2p_matching(&graph.nodes, dst_coords));

    if let Some(sp) = d.search(src_id, dst_id) {
        // Lookup coordinates
        let path = sp
            .nodes
            .iter()
            .map(|node_id| {
                let node = graph.nodes.iter().find(|n| n.id == *node_id).unwrap();
                [node.lon, node.lat]
            })
            .collect();

        PathResult::new(
            path,
            sp.weight,
            d.stats.duration.unwrap_or(Default::default()).as_secs_f64(),
            d.stats.nodes_settled,
        )
    } else {
        PathResult::new(vec![], 0.0, 0.0, 0)
    }
}

fn p2p_matching(nodes: &[ch_core::graph::Node], coords: [f64; 2]) -> ch_core::constants::NodeId {
    nodes
        .iter()
        .min_by(|a, b| {
            let a_dist = (a.lat - coords[0]).powi(2) + (a.lon - coords[1]).powi(2);
            let b_dist = (b.lat - coords[0]).powi(2) + (b.lon - coords[1]).powi(2);
            a_dist.partial_cmp(&b_dist).unwrap()
        })
        .unwrap()
        .id
}

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    // write to the OS logs folder
                    // LogTarget::LogDir,
                    // write to stdout
                    LogTarget::Stdout,
                    // forward logs to the webview
                    LogTarget::Webview,
                ])
                .level(log::LevelFilter::Info)
                .with_colors(
                    ColoredLevelConfig::new().info(tauri_plugin_log::fern::colors::Color::Green),
                )
                .build(),
        )
        // .manage(Mutex::new(Dijkstra::default()))
        .manage(Graph::default())
        .invoke_handler(tauri::generate_handler![
            create_graph_from_pbf,
            get_edges,
            get_nodes,
            calc_path
        ])
        // .run(tauri::generate_context!())
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
