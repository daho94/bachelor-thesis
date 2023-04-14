//! Minimal example
use std::path::{Path, PathBuf};

use ch_core::{constants::NodeId, dijkstra::Dijkstra, graph::Graph};
use reedline_repl_rs::clap::{value_parser, Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Write "Hello" with given name

/// Print graph info
fn info(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    Ok(Some(format!(
        "Graph has {} nodes and {} edges",
        context.graph.nodes.len(),
        context.graph.edges.len()
    )))
}

fn run_dijkstra(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let src = *args.get_one::<usize>("src").unwrap();
    let dst = *args.get_one::<usize>("dst").unwrap();

    let mut dijkstra = Dijkstra::new(&context.graph);
    let sp = dijkstra.search(src, dst);

    if let Some(sp) = sp {
        let mut path = String::new();
        for node in sp.nodes {
            path.push_str(&format!("{}\n", node));
        }
        path.push_str(&format!("Took: {:?}", dijkstra.stats.duration));
        Ok(Some(path))
    } else {
        Ok(Some("No path found".to_string()))
    }
}

fn measure_dijkstra(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    use rand::Rng;

    let n = *args.get_one::<usize>("n").unwrap_or(&10);

    // Select n random start and end nodes
    let mut rng = rand::thread_rng();
    let src_nodes: Vec<NodeId> = (0..n)
        .map(|_| context.graph.nodes[rng.gen_range(0..context.graph.nodes.len())].id)
        .collect();
    let dst_nodes: Vec<NodeId> = (0..n)
        .map(|_| context.graph.nodes[rng.gen_range(0..context.graph.nodes.len())].id)
        .collect();

    let mut res = String::new();
    // Run Dijkstra for each pair of nodes
    for (src, dst) in src_nodes.iter().zip(dst_nodes.iter()) {
        let mut dijkstra = Dijkstra::new(&context.graph);
        let sp = dijkstra.search(*src, *dst);
        if sp.is_none() {
            continue;
        }
        res.push_str(&format!(
            "{} -> {}: {:?}\n",
            src, dst, dijkstra.stats.duration
        ));
    }

    Ok(Some(res))
}

#[derive(Default)]
struct Context {
    graph: Graph,
}

impl Context {
    fn new(graph: Graph) -> Self {
        Self { graph }
    }
}

fn main() -> Result<()> {
    // Init Graph
    let path_to_pbf = std::env::args().nth(1).expect("No path to PBF file given");
    let graph = Graph::from_pbf(Path::new(&path_to_pbf)).unwrap();
    let context = Context::new(graph);

    let mut repl = Repl::new(context)
        .with_name("Pathfinder")
        .with_version("v0.1.0")
        .with_description("Simple REPL to test graph search algorithms")
        .with_banner("Welcome to Pathfinder")
        .with_history(PathBuf::from(r".\history"), 100)
        .with_command(Command::new("info").about("Print graph info"), info)
        .with_command(
            Command::new("dijk")
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
                .about("Calculate shortest path using Dijkstra's algorithm"),
            run_dijkstra,
        )
        .with_command(
            Command::new("dijkm")
                .arg(
                    Arg::new("n")
                        .value_parser(value_parser!(usize))
                        .required(false)
                        .help("Number of random shortest paths to calculate"),
                )
                .about("Measure `n` random shortest paths calculations"),
            measure_dijkstra,
        );

    repl.run()
}
