#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ch_core::{constants::NodeId, dijkstra::Dijkstra};
use std::time::Instant;
use std::{path::Path, sync::Mutex};
use tauri::{Manager, State};

struct Graph(Mutex<ch_core::graph::Graph>);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn create_graph_from_pbf(
    // graph: State<'_, Graph>,
    d: State<'_, Mutex<Dijkstra>>,
    path: &str,
) -> Result<(), String> {
    println!("Creating graph from file '{}'...", path);
    let start = Instant::now();
    if let Ok(g) = ch_core::graph::Graph::from_pbf(Path::new(path)) {
        let duration = start.elapsed();
        println!(
            "Created Graph with {} nodes and {} edges in {:?}!",
            g.nodes.len(),
            g.edges.len(),
            duration
        );
        // let mut graph = graph.0.lock().unwrap();
        // *graph = g;
        let mut d = d.lock().unwrap();
        *d = Dijkstra::new(g);

        Ok(())
    } else {
        Err("Could not create graph!".to_string())
    }
}

#[tauri::command]
fn get_edges(d: State<'_, Mutex<Dijkstra>>) -> Vec<[[f64; 2]; 2]> {
    let graph = &d.lock().unwrap().graph;
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
fn get_nodes(d: State<'_, Mutex<Dijkstra>>) -> Vec<[f64; 3]> {
    let graph = &d.lock().unwrap().graph;
    graph
        .nodes
        .iter()
        .map(|node| [node.id as f64, node.lon, node.lat])
        .collect()
}

#[tauri::command]
fn calc_path(
    dijkstra: State<'_, Mutex<Dijkstra>>,
    src_coords: [f64; 2],
    dst_coords: [f64; 2],
) -> (Vec<[f64; 2]>, f64) {
    let d = dijkstra.lock().unwrap();

    let src_id = dbg!(p2p_matching(&d.graph.nodes, src_coords));
    let dst_id = dbg!(p2p_matching(&d.graph.nodes, dst_coords));

    let time = Instant::now();
    if let Some(sp) = dbg!(d.search(src_id, dst_id)) {
        let elapsed = time.elapsed();
        println!("Found path in {:?}", elapsed);
        // Lookup coordinates
        (
            sp.nodes
                .iter()
                .map(|node_id| {
                    let node = &d.graph.nodes.iter().find(|n| n.id == *node_id).unwrap();
                    [node.lon, node.lat]
                })
                .collect(),
            elapsed.as_secs_f64(),
        )
    } else {
        (vec![], 0.0)
    }
}

fn p2p_matching(nodes: &[ch_core::graph::Node], coords: [f64; 2]) -> ch_core::constants::NodeId {
    println!("test");
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
        // .manage(Graph(Default::default()))
        .manage(Mutex::new(Dijkstra::default()))
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
