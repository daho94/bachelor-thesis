//! Minimal example
use std::path::{Path, PathBuf};

use ch_core::{
    astar::AStar,
    constants::NodeId,
    dijkstra::{Dijkstra, ShortestPath},
    graph::Graph,
};
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

fn run_algorithm(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let src = *args.get_one::<usize>("src").unwrap();
    let dst = *args.get_one::<usize>("dst").unwrap();

    let (sp, stats) = match args.get_one::<String>("algo").unwrap().as_str() {
        "dijk" => {
            let mut d = Dijkstra::new(&context.graph);
            (d.run(src, dst), d.stats)
        }
        "astar" => {
            let mut a = AStar::new(&context.graph);
            (a.run(src, dst), a.stats)
        }
        _ => panic!("Unknown algorithm"),
    };

    if let Some(sp) = sp {
        let mut path = String::new();
        for node in sp.nodes {
            path.push_str(&format!("{}\n", node));
        }
        path.push_str(&format!(
            "Took: {:?} / {} nodes settled",
            stats.duration, stats.nodes_settled
        ));
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
            "{} -> {}: {:?} / {} nodes settled\n",
            src,
            dst,
            dijkstra.stats.duration.unwrap(),
            dijkstra.stats.nodes_settled
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

trait Runnable {
    fn run(&mut self, src: NodeId, dst: NodeId) -> Option<ShortestPath>;
    fn stats(&self) -> &ch_core::statistics::Stats;
}

impl Runnable for Dijkstra<'_> {
    fn run(&mut self, src: NodeId, dst: NodeId) -> Option<ShortestPath> {
        self.search(src, dst)
    }

    fn stats(&self) -> &ch_core::statistics::Stats {
        &self.stats
    }
}

impl Runnable for AStar<'_> {
    fn run(&mut self, src: NodeId, dst: NodeId) -> Option<ShortestPath> {
        fn straight_line(
            src: &ch_core::graph::Node,
            dst: &ch_core::graph::Node,
        ) -> ch_core::constants::Weight {
            // Calculate the distance between two nodes using the Haversine formula
            let lat1 = src.lat.to_radians();
            let lat2 = dst.lat.to_radians();
            let lon1 = src.lon.to_radians();
            let lon2 = dst.lon.to_radians();
            let a = (lat2 - lat1) / 2.0;
            let b = (lon2 - lon1) / 2.0;
            let c = a.sin().powi(2) + lat1.cos() * lat2.cos() * b.sin().powi(2);
            let d = 2.0 * c.sqrt().asin();

            6371.0 * d / 110.0 * 3600.0 // Umrechnung in Sekunden
        }
        self.search(src, dst, straight_line)
    }

    fn stats(&self) -> &ch_core::statistics::Stats {
        &self.stats
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
        )
        .with_command(
            Command::new("run")
                .arg(
                    Arg::new("algo")
                        .value_parser(["dijk", "astar"])
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
        );

    repl.run()
}
