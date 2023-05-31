//! Minimal example
use std::path::{Path, PathBuf};

use ch_core::{
    constants::OsmId,
    graph::{node_index, DefaultIdx, Graph},
    node_contraction::NodeContractor,
    overlay_graph::OverlayGraph,
    search::{astar::AStar, shortest_path::ShortestPath},
    search::{bidir_search::BiDirSearch, dijkstra::Dijkstra},
    util::math::straight_line,
};
use reedline_repl_rs::clap::{value_parser, Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Print graph info
fn info(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    context.graph.road_graph().print_info();
    context.graph.print_info();

    Ok(None)
}

fn run_dijkstra(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let src = *args.get_one::<usize>("src").unwrap();
    let dst = *args.get_one::<usize>("dst").unwrap();

    let mut dijkstra = Dijkstra::new(context.graph.road_graph());
    let sp = dijkstra.search(node_index(src), node_index(dst));

    if let Some(sp) = sp {
        let mut path = String::new();
        for node in sp.nodes {
            path.push_str(&format!("{:?}\n", node));
        }
        path.push_str(&format!("Took: {:?}", dijkstra.stats.duration));
        Ok(Some(path))
    } else {
        Ok(Some("No path found".to_string()))
    }
}

fn run_algorithm(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let src = *args.get_one::<usize>("src").unwrap();
    let dst = *args.get_one::<usize>("dst").unwrap();

    let (sp, stats) = match args.get_one::<String>("algo").unwrap().as_str() {
        "dijk" => {
            let mut d = Dijkstra::new(context.graph.road_graph());
            (d.run(src, dst), d.stats)
        }
        "astar" => {
            let mut a = AStar::new(context.graph.road_graph());
            (a.run(src, dst), a.stats)
        }
        "bidir" => {
            let mut b = BiDirSearch::new(&context.graph);
            (b.run(src, dst), b.stats)
        }
        _ => panic!("Unknown algorithm"),
    };

    if let Some(sp) = sp {
        let mut path = String::new();
        // for node in sp.nodes {
        //     path.push_str(&format!("{:?}\n", node.index()));
        // }
        path.push_str(&format!(
            "{:?}\n",
            sp.nodes.iter().map(|n| n.index()).collect::<Vec<_>>()
        ));
        path.push_str(&format!(
            "Took: {:?} / {} nodes settled",
            stats.duration, stats.nodes_settled
        ));
        Ok(Some(path))
    } else {
        Ok(Some("No path found".to_string()))
    }
}

fn save_graph(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let path = args.get_one::<PathBuf>("path").unwrap();
    match context.graph.encode(path) {
        Ok(bytes_written) => Ok(Some(format!("Graph saved ({} Bytes)", bytes_written))),
        Err(e) => Ok(Some(format!("Error saving graph: {}", e))),
    }
}
// fn measure_dijkstra(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
//     use rand::Rng;

//     let n = *args.get_one::<usize>("n").unwrap_or(&10);

//     // Select n random start and end nodes
//     let mut rng = rand::thread_rng();
//     let src_nodes: Vec<OsmId> = (0..n)
//         .map(|_| rng.gen_range(0..context.graph.nodes.len()))
//         .collect();
//     let dst_nodes: Vec<OsmId> = (0..n)
//         .map(|_| rng.gen_range(0..context.graph.nodes.len()))
//         .collect();

//     let mut res = String::new();
//     // Run Dijkstra for each pair of nodes
//     for (src, dst) in src_nodes.iter().zip(dst_nodes.iter()) {
//         let mut dijkstra = Dijkstra::new(&context.graph);
//         let sp = dijkstra.search(node_index(*src), node_index(*dst));
//         if sp.is_none() {
//             continue;
//         }
//         res.push_str(&format!(
//             "{} -> {}: {:?} / {} nodes settled\n",
//             src,
//             dst,
//             dijkstra.stats.duration.unwrap(),
//             dijkstra.stats.nodes_settled
//         ));
//     }

//     Ok(Some(res))
// }

struct Context {
    graph: OverlayGraph,
}

impl Context {
    fn new(graph: OverlayGraph) -> Context {
        Self { graph }
    }
}

trait Runnable {
    fn run(&mut self, src: OsmId, dst: OsmId) -> Option<ShortestPath<DefaultIdx>>;
    fn stats(&self) -> &ch_core::statistics::Stats;
}

impl Runnable for Dijkstra<'_> {
    fn run(&mut self, src: OsmId, dst: OsmId) -> Option<ShortestPath<DefaultIdx>> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::Stats {
        &self.stats
    }
}

impl Runnable for AStar<'_> {
    fn run(&mut self, src: OsmId, dst: OsmId) -> Option<ShortestPath<DefaultIdx>> {
        self.search(node_index(src), node_index(dst), straight_line)
    }

    fn stats(&self) -> &ch_core::statistics::Stats {
        &self.stats
    }
}

impl Runnable for BiDirSearch<'_> {
    fn run(&mut self, src: OsmId, dst: OsmId) -> Option<ShortestPath<DefaultIdx>> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::Stats {
        &self.stats
    }
}

fn main() -> Result<()> {
    env_logger::init();
    // Init Graph
    let path_to_pbf = std::env::args().nth(1).expect("No path to PBF file given");
    let mut graph = Graph::<DefaultIdx>::from_pbf(Path::new(&path_to_pbf)).unwrap();

    let mut contractor = NodeContractor::new(&mut graph);
    let overlay_graph = contractor.run();

    let context = Context::new(overlay_graph);

    let mut repl = Repl::new(context)
        .with_name("Pathfinder")
        .with_version("v0.1.0")
        .with_description("Simple REPL to test graph search algorithms")
        .with_banner("Welcome to Pathfinder")
        .with_history(PathBuf::from(r".\history"), 100)
        .with_command(Command::new("info").about("Print graph info"), info)
        // .with_command(
        //     Command::new("dijk")
        //         .arg(
        //             Arg::new("src")
        //                 .value_parser(value_parser!(usize))
        //                 .required(true)
        //                 .help("ID of source node"),
        //         )
        //         .arg(
        //             Arg::new("dst")
        //                 .value_parser(value_parser!(usize))
        //                 .required(true)
        //                 .help("ID of destination node"),
        //         )
        //         .about("Calculate shortest path using Dijkstra's algorithm"),
        //     run_dijkstra,
        // )
        // .with_command(
        //     Command::new("dijkm")
        //         .arg(
        //             Arg::new("n")
        //                 .value_parser(value_parser!(usize))
        //                 .required(false)
        //                 .help("Number of random shortest paths to calculate"),
        //         )
        //         .about("Measure `n` random shortest paths calculations"),
        //     measure_dijkstra,
        // )
        .with_command(
            Command::new("run")
                .arg(
                    Arg::new("algo")
                        .value_parser(["dijk", "astar", "bidir"])
                        .default_value("dijk")
                        .required(true)
                        .help("Name of algorithm"),
                )
                .arg(
                    Arg::new("src")
                        .value_parser(value_parser!(usize))
                        .required(true)
                        .help("ID of source node"),
                )
                .arg(
                    Arg::new("dst")
                        .value_parser(value_parser!(usize))
                        .required(true)
                        .help("ID of destination node"),
                )
                .about("Runs the selected algorithm"),
            run_algorithm,
        )
        .with_command(
            Command::new("save").arg(
                Arg::new("path")
                    .value_parser(value_parser!(PathBuf))
                    .required(true)
                    .help("Path to save graph to"),
            ),
            save_graph,
        );

    repl.run()
}
