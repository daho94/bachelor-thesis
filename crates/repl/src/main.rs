//! Minimal example
use std::{path::PathBuf, time::Duration};

use ch_core::{
    graph::{node_index, Graph},
    node_contraction::NodeContractor,
    overlay_graph::OverlayGraph,
    search::{astar::AStar, shortest_path::ShortestPath},
    search::{ch_search::CHSearch, dijkstra::Dijkstra, BidirDijkstra},
    util::{cli, math::straight_line},
};
use indicatif::ProgressBar;
use reedline_repl_rs::clap::{value_parser, Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Print graph info
fn info(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    context.graph.road_graph().print_info();
    context.graph.print_info();

    Ok(None)
}

fn bench_algorithm(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let bench = Benchmark::new(
        context.graph.nodes().count(),
        *args.get_one::<usize>("iterations").unwrap(),
    );

    match args.get_one::<String>("algo").unwrap().as_str() {
        "dijk" => {
            let mut d = Dijkstra::new(context.graph.road_graph());
            println!("Bench: {}", d.bench(bench));
        }
        "astar" => {
            let mut a = AStar::new(context.graph.road_graph());
            println!("Bench: {}", a.bench(bench));
        }
        "ch" => {
            let mut b = CHSearch::new(&context.graph);
            println!("Bench: {}", b.bench(bench));
        }
        "bidir_dijk" => {
            let mut b = BidirDijkstra::new(context.graph.road_graph());
            println!("Bench: {}", b.bench(bench));
        }
        _ => println!("Unknown algorithm"),
    }
    Ok(Some("Done.".to_string()))
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
        "ch" => {
            let mut b = CHSearch::new(&context.graph);
            (b.run(src, dst), b.stats)
        }
        "bidir_dijk" => {
            let mut b = BidirDijkstra::new(context.graph.road_graph());
            (b.run(src, dst), b.stats)
        }
        _ => unreachable!("Unknown algorithm"),
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

struct Context {
    graph: OverlayGraph,
}

impl Context {
    fn new(graph: OverlayGraph) -> Context {
        Self { graph }
    }
}

#[derive(Debug)]
struct BenchmarkResult {
    pub mean_query: f64,
    pub median_query: f64,
    pub mean_nodes_settled: f64,
    pub median_nodes_settled: f64,
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Query: mean: {:?}, median: {:?}",
            Duration::from_secs_f64(self.mean_query),
            Duration::from_secs_f64(self.median_query),
        )?;
        writeln!(
            f,
            "Nodes settled: mean: # {}, median: # {}",
            self.mean_nodes_settled, self.median_nodes_settled,
        )
    }
}

struct Benchmark {
    num_nodes: usize,
    iterations: usize,
}

impl Benchmark {
    pub fn new(num_nodes: usize, iterations: usize) -> Self {
        Self {
            num_nodes,
            iterations,
        }
    }
}

trait Runnable {
    fn run(&mut self, src: usize, dst: usize) -> Option<ShortestPath>;
    fn bench(&mut self, b: Benchmark) -> BenchmarkResult {
        use rand::{rngs::StdRng, Rng};

        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(187);
        let mut timings = Vec::with_capacity(b.iterations);
        let mut nodes_settled = Vec::with_capacity(b.iterations);

        let pb = ProgressBar::new(b.iterations as u64);
        for _ in 0..b.iterations {
            let src = rng.gen_range(0..b.num_nodes);
            let dst = rng.gen_range(0..b.num_nodes);

            self.run(src, dst);

            timings.push(self.stats().duration.unwrap().as_secs_f64());
            nodes_settled.push(self.stats().nodes_settled as f64);
            pb.inc(1);
        }

        timings.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean_query = timings.iter().sum::<f64>() / timings.len() as f64;
        let mean_nodes_settled = nodes_settled.iter().sum::<f64>() / nodes_settled.len() as f64;

        let mid = timings.len() / 2;

        let median_query;
        let median_nodes_settled;

        if timings.len() % 2 == 0 {
            median_query = (timings[mid] + timings[mid - 1]) / 2.0;
            median_nodes_settled = (nodes_settled[mid] + nodes_settled[mid - 1]) / 2.0;
        } else {
            median_query = timings[mid];
            median_nodes_settled = nodes_settled[mid];
        }

        BenchmarkResult {
            mean_query,
            median_query,
            mean_nodes_settled,
            median_nodes_settled,
        }
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats;
}

impl Runnable for Dijkstra<'_> {
    fn run(&mut self, src: usize, dst: usize) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

impl Runnable for BidirDijkstra<'_> {
    fn run(&mut self, src: usize, dst: usize) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

impl Runnable for AStar<'_> {
    fn run(&mut self, src: usize, dst: usize) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst), straight_line)
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

impl Runnable for CHSearch<'_> {
    fn run(&mut self, src: usize, dst: usize) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let cfg = cli::parse();

    let mut graph = if cfg.simplify {
        Graph::from_pbf_with_simplification(&cfg.pbf_file).unwrap()
    } else {
        Graph::from_pbf(&cfg.pbf_file).unwrap()
    };

    let mut contractor = NodeContractor::new_with_params(&mut graph, cfg.params);
    let overlay_graph = contractor.run_with_strategy(cfg.strategy);

    let context = Context::new(overlay_graph);

    let mut repl = Repl::new(context)
        .with_name("Pathfinder")
        .with_version("v0.1.0")
        .with_description("Simple REPL to test graph search algorithms")
        .with_banner("Welcome to Pathfinder")
        .with_history(PathBuf::from(r".\history"), 100)
        .with_command(Command::new("info").about("Print graph info"), info)
        .with_command(
            Command::new("bench")
                .arg(
                    Arg::new("algo")
                        .value_parser(["dijk", "astar", "ch", "bidir_dijk"])
                        .default_value("dijk")
                        .required(true)
                        .help("Name of algorithm"),
                )
                .arg(
                    Arg::new("iterations")
                        .value_parser(value_parser!(usize))
                        .default_value("1000")
                        .default_missing_value("1000")
                        .required(false),
                )
                .about("Bench the selected algorithm"),
            bench_algorithm,
        )
        .with_command(
            Command::new("run")
                .arg(
                    Arg::new("algo")
                        .value_parser(["dijk", "astar", "ch", "bidir_dijk"])
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
                .about("Run the selected algorithm"),
            run_algorithm,
        )
        .with_command(
            Command::new("save")
                .arg(
                    Arg::new("path")
                        .value_parser(value_parser!(PathBuf))
                        .required(true)
                        .help("Path to save graph to"),
                )
                .about("Save graph to file"),
            save_graph,
        );

    repl.run()
}
