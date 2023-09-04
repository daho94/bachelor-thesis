//! Minimal example
use std::path::PathBuf;

use ch_core::{
    constants::OSMId,
    graph::{node_index, DefaultIdx, Graph},
    node_contraction::NodeContractor,
    overlay_graph::OverlayGraph,
    search::{astar::AStar, shortest_path::ShortestPath},
    search::{ch_search::CHSearch, dijkstra::Dijkstra},
    util::{cli, math::straight_line},
};
use reedline_repl_rs::clap::{value_parser, Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

/// Print graph info
fn info(_args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    context.graph.road_graph().print_info();
    context.graph.print_info();

    Ok(None)
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
        "ch_par" => {
            let mut b = CHSearch::new(&context.graph);
            (b.search_par(node_index(src), node_index(dst)), b.stats)
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

trait Runnable {
    fn run(&mut self, src: OSMId, dst: OSMId) -> Option<ShortestPath>;
    fn stats(&self) -> &ch_core::statistics::SearchStats;
}

impl Runnable for Dijkstra<'_> {
    fn run(&mut self, src: OSMId, dst: OSMId) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst))
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

impl Runnable for AStar<'_> {
    fn run(&mut self, src: OSMId, dst: OSMId) -> Option<ShortestPath> {
        self.search(node_index(src), node_index(dst), straight_line)
    }

    fn stats(&self) -> &ch_core::statistics::SearchStats {
        &self.stats
    }
}

impl Runnable for CHSearch<'_> {
    fn run(&mut self, src: OSMId, dst: OSMId) -> Option<ShortestPath> {
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
            Command::new("run")
                .arg(
                    Arg::new("algo")
                        .value_parser(["dijk", "astar", "ch", "ch_par"])
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
