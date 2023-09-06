use std::{collections::BinaryHeap, fs::File, path::Path};

use ch_core::{
    constants::Weight,
    graph::{node_index, NodeIndex},
    node_contraction::{ContractionParams, NodeContractor, PriorityParams},
    search::bidir_dijkstra::BidirDijkstra,
    util::{
        cli,
        test_graphs::{graph_saarland, graph_vaterstetten},
    },
};
use ch_core::{graph::Graph, search::dijkstra::Candidate};
use ch_core::{
    search::{astar::AStar, ch_search::CHSearch, dijkstra::Dijkstra},
    util::math::straight_line,
};
use indicatif::ProgressBar;
use plotly::{
    box_plot::BoxPoints,
    color::Rgb,
    common::{Font, Line, Marker, Mode, Orientation, Title},
    layout::{Axis, BoxMode, Legend, Margin, VAlign},
    BoxPlot, Layout, Plot, Scatter,
};
use rand::prelude::*;
use rustc_hash::FxHashMap;
use std::io::Write;

// Randomly select a source node s.
// - For each power of two r = 2^k, use Dijkstra's algorithm to find a node t
//   with Dijkstra Rank rks(t) = r. This means that t is the r-th node to be
//   settled during the Dijkstra traversal from node s.
// - Perform an s-t query using your algorithm.
// - Collect statistics for each value of r = 2^k, which represents different
//   levels of difficulty for the query based on how far the node t is from node
//   s in terms of Dijkstra Rank.
// - Plot the statistics using a box-and-whiskers plot, where each box
//   represents the spread between the lower and upper quartiles, the median is
//   shown inside the box, and the whiskers extend to the minimum and maximum
//   values, excluding outliers.
fn main() {
    const ITERATIONS: usize = 1_00;

    // let cli = cli::parse();

    let mut g = if let Some(path) = std::env::args().find(|p| p.ends_with(".pbf")) {
        Graph::from_pbf_with_simplification(Path::new(&path)).expect("Invalid path")
    } else {
        graph_saarland()
    };
    // let mut g;
    // if cli.simplify {
    //     g = Graph::from_pbf_with_simplification(Path::new(&cli.pbf_file)).expect("Invalid path");
    // } else {
    //     g = Graph::from_pbf(Path::new(&cli.pbf_file)).expect("Invalid path");
    // }
    let mut params = ContractionParams::default();
    params = params.priority_params(PriorityParams {
        edge_difference_coeff: 190,
        contracted_neighbors_coeff: 120,
        search_space_coeff: 1,
        original_edges_coeff: 70,
    });

    let mut contractor = NodeContractor::new_with_params(&mut g, params);
    println!("Started node contraction");
    let overlay_graph = contractor.run();
    println!("Finished node contraction");

    let num_nodes = g.nodes.len();
    let max_rank = (g.nodes.len() as f64).log2() as u32;

    let rank_start = 10;
    let rank_end = dbg!(max_rank);
    // let rank_end = 16;

    let num_ranks = (rank_end - rank_start + 1) as usize;

    let mut timings_dijk = vec![vec![]; num_ranks];
    let mut timings_astar = vec![vec![]; num_ranks];
    let mut timings_bidir = vec![vec![]; num_ranks];
    let mut timings_ch = vec![vec![]; num_ranks];

    let mut nodes_settled_dijk = vec![vec![]; num_ranks];
    let mut nodes_settled_astar = vec![vec![]; num_ranks];
    let mut nodes_settled_bidir = vec![vec![]; num_ranks];
    let mut nodes_settled_ch = vec![vec![]; num_ranks];

    let mut ch = CHSearch::new(&overlay_graph);
    let mut dijk = Dijkstra::new(&g);
    let mut bidir = BidirDijkstra::new(&g);
    let mut astar = AStar::new(&g);

    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(187);

    println!("Start iterations");
    let pb = ProgressBar::new(ITERATIONS as u64);
    for _ in 0..ITERATIONS {
        // Generate random start node
        let source = node_index(rng.gen_range(0..num_nodes));
        // Find targets
        let targets = calculate_st_queries(source, &g, rank_start, rank_end);

        // Measure query times and #nodes settled for Dijsktra-, AStar- and CH-query
        for (target, rank) in targets {
            let idx = (rank.ilog2() - rank_start) as usize;

            dijk.search(source, target).unwrap();
            timings_dijk[idx].push(dijk.stats.duration.unwrap().as_micros() as f64);
            nodes_settled_dijk[idx].push(dijk.stats.nodes_settled as f64);

            astar.search(source, target, straight_line).unwrap();
            timings_astar[idx].push(astar.stats.duration.unwrap().as_micros() as f64);
            nodes_settled_astar[idx].push(astar.stats.nodes_settled as f64);

            bidir.search(source, target).unwrap();
            timings_bidir[idx].push(bidir.stats.duration.unwrap().as_micros() as f64);
            nodes_settled_bidir[idx].push(bidir.stats.nodes_settled as f64);

            ch.search(source, target).unwrap();
            timings_ch[idx].push(ch.stats.duration.unwrap().as_micros() as f64);
            nodes_settled_ch[idx].push(ch.stats.nodes_settled as f64);
        }

        pb.inc(1);
    }
    pb.finish_with_message("Measurements finished.");

    // Write stats to file
    let mut file = File::create("query_time_algos.csv").expect("Couldn't create file");
    let header = {
        let ranks: Vec<String> = (rank_start..=rank_end)
            .map(|r| format!("2{}", superscript_digits(r)))
            .collect();

        format!("stats,{}", ranks.join(","))
    };

    writeln!(&mut file, "{}", header).unwrap();

    write_stats(&mut file, &mut timings_dijk, &nodes_settled_dijk);
    write_stats(&mut file, &mut timings_astar, &nodes_settled_astar);
    write_stats(&mut file, &mut timings_bidir, &nodes_settled_bidir);
    write_stats(&mut file, &mut timings_ch, &nodes_settled_ch);

    // Create plots
    let x: Vec<String> = (rank_start..=rank_end)
        .flat_map(|k| {
            (0..ITERATIONS)
                .map(|_| format!("2{}", superscript_digits(k)))
                .collect::<Vec<String>>()
        })
        .collect();

    let mut plot = Plot::new();

    let marker = Marker::new()
        .symbol(plotly::common::MarkerSymbol::CircleOpen)
        .line(Line::new().outlier_width(1));

    let trace_dijk = BoxPlot::new_xy(x.clone(), timings_dijk.into_iter().flatten().collect())
        .name("Dijkstra")
        .marker(marker.clone())
        .box_points(BoxPoints::Outliers)
        .line(Line::new().width(0.7))
        .whisker_width(8.);

    let trace_astar = BoxPlot::new_xy(x.clone(), timings_astar.into_iter().flatten().collect())
        .name("AStar")
        .marker(marker.clone())
        .box_points(BoxPoints::Outliers)
        .line(Line::new().width(0.7))
        .whisker_width(8.);

    let trace_bidir = BoxPlot::new_xy(x.clone(), timings_bidir.into_iter().flatten().collect())
        .name("Bidir. Dijkstra")
        .marker(marker.clone())
        .box_points(BoxPoints::Outliers)
        .line(Line::new().width(0.7))
        .whisker_width(8.);

    let trace_ch = BoxPlot::new_xy(x.clone(), timings_ch.into_iter().flatten().collect())
        .name("CHs")
        .marker(marker)
        .box_points(BoxPoints::Outliers)
        .line(Line::new().width(0.7))
        .whisker_width(8.);

    plot.add_trace(trace_dijk);
    plot.add_trace(trace_astar);
    plot.add_trace(trace_bidir);
    // plot.add_trace(trace_ch);

    let y_axis_log = Axis::new()
        .title(Title::new("Query-Time [μs]"))
        .zero_line(true)
        .tick_font(Font::new().size(20).family("Calibri Light"))
        .type_(plotly::layout::AxisType::Log);

    let y_axis = Axis::new()
        .title(Title::new("Query-Time [μs]"))
        .tick_font(Font::new().size(20).family("Calibri Light"))
        .zero_line(true);

    let layout = Layout::new()
        // .width(1000) //800
        .height(600)
        .y_axis(y_axis)
        .font(Font::new().family("Calibri").size(20))
        .x_axis(
            Axis::new()
                .title(Title::new("Dijkstra Rank"))
                .tick_font(Font::new().size(20).family("Calibri")),
        )
        .colorway(vec![
            Rgb::new(216, 27, 96),
            Rgb::new(39, 136, 229),
            Rgb::new(0, 77, 64),
            Rgb::new(255, 193, 7),
        ])
        .margin(Margin::default().top(8).bottom(8))
        .legend(
            Legend::new()
                .orientation(Orientation::Horizontal)
                // .y(-1.00)
                .valign(VAlign::Top),
        )
        .box_mode(BoxMode::Group);

    plot.set_layout(layout.clone());
    plot.show();

    plot.write_image("boxplot_rank.pdf", plotly::ImageFormat::PDF, 800, 600, 1.0);

    plot.set_layout(layout.clone().y_axis(y_axis_log.clone()));
    plot.show();

    plot.write_image(
        "boxplot_rank_log.pdf",
        plotly::ImageFormat::PDF,
        800,
        600,
        1.0,
    );
    plot.write_image(
        "boxplot_rank_log.svg",
        plotly::ImageFormat::SVG,
        1600,
        600,
        1.0,
    );

    // Plot nodes settled
    let x: Vec<String> = (rank_start..=rank_end)
        .map(|k| format!("2{}", superscript_digits(k)))
        .collect::<Vec<String>>();

    let mut plot = Plot::new();
    dbg!(x.clone());
    let trace_dijk = Scatter::new(
        x.clone(),
        nodes_settled_dijk.into_iter().map(|n| mean(&n)).collect(),
    )
    .mode(Mode::Lines)
    .name("Dijkstra");

    let trace_astar = Scatter::new(
        x.clone(),
        nodes_settled_astar.into_iter().map(|n| mean(&n)).collect(),
    )
    .mode(Mode::Lines)
    .name("AStar");

    let trace_bidir = Scatter::new(
        x.clone(),
        nodes_settled_bidir.into_iter().map(|n| mean(&n)).collect(),
    )
    .mode(Mode::Lines)
    .name("Bidir. Dijkstra");

    let trace_ch = Scatter::new(x, nodes_settled_ch.into_iter().map(|n| mean(&n)).collect())
        .mode(Mode::Lines)
        .name("CHs");

    plot.add_trace(trace_dijk);
    plot.add_trace(trace_astar);
    plot.add_trace(trace_bidir);
    // plot.add_trace(trace_ch);

    let y_axis = Axis::new()
        .title(Title::new("Nodes Settled [#]"))
        .type_(plotly::layout::AxisType::Log)
        .zero_line(false);

    let layout = layout.y_axis(y_axis);

    plot.set_layout(layout);
    plot.show();

    plot.write_image(
        "boxplot_rank_nodes.pdf",
        plotly::ImageFormat::PDF,
        800,
        600,
        1.0,
    );
    plot.write_image(
        "boxplot_rank_nodes.svg",
        plotly::ImageFormat::SVG,
        1600,
        600,
        1.0,
    );
}

fn write_stats(file: &mut File, timings: &mut [Vec<f64>], nodes_settled: &[Vec<f64>]) {
    let t_mean: Vec<String> = timings.iter().map(|t| format!("{:.0}", mean(t))).collect();
    let _ = writeln!(file, "mean,{}", t_mean.join(","));

    let t_median: Vec<String> = timings
        .iter_mut()
        .map(|t| format!("{:.0}", median(t)))
        .collect();
    let _ = writeln!(file, "median,{}", t_median.join(","));

    let avg_settled: Vec<String> = nodes_settled
        .iter()
        .map(|t| format!("{:.0}", mean(t)))
        .collect();
    let _ = writeln!(file, "nodes_settled,{}", avg_settled.join(","));
}

fn median(numbers: &mut [f64]) -> f64 {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mid = numbers.len() / 2;

    if numbers.len() % 2 == 0 {
        mean(&[numbers[mid - 1], numbers[mid]])
    } else {
        numbers[mid]
    }
}

fn mean(numbers: &[f64]) -> f64 {
    let sum: f64 = numbers.iter().sum();

    sum / numbers.len() as f64
}

fn calculate_st_queries(
    source: NodeIndex,
    g: &Graph,
    rank_start: u32,
    rank_end: u32,
) -> Vec<(NodeIndex, usize)> {
    let mut ranks = (rank_start..=rank_end)
        .rev()
        .map(|k| 2usize.pow(k))
        .collect::<Vec<usize>>();

    let mut node_data: FxHashMap<NodeIndex, (Weight, Option<NodeIndex>)> = FxHashMap::default();
    let mut node_ranks: FxHashMap<NodeIndex, usize> = FxHashMap::default();

    node_data.insert(source, (0.0, None));

    let mut queue = BinaryHeap::new();
    let mut nodes_settled = 0;

    queue.push(Candidate::new(source, 0.0));
    let mut next_rank = ranks.pop().unwrap();

    while let Some(Candidate { weight, node_idx }) = queue.pop() {
        nodes_settled += 1;

        if nodes_settled >= next_rank {
            node_ranks.insert(node_idx, next_rank);
            if let Some(r) = ranks.pop() {
                next_rank = r;
            } else {
                break;
            }
        }

        for (_, edge) in g
            .neighbors_outgoing(node_idx)
            .filter(|(edge_idx, _)| edge_idx.index() < g.edges.len() - g.num_shortcuts)
        {
            let new_distance = weight + edge.weight;
            if new_distance
                < node_data
                    .get(&edge.target)
                    .unwrap_or(&(std::f64::INFINITY, None))
                    .0
            {
                node_data.insert(edge.target, (new_distance, Some(node_idx)));
                queue.push(Candidate::new(edge.target, new_distance));
            }
        }
    }

    let mut targets = node_ranks
        .iter()
        .map(|(node_idx, rank)| (*node_idx, *rank))
        .collect::<Vec<(NodeIndex, usize)>>();

    targets.sort_by(|(_, rank1), (_, rank2)| rank1.cmp(rank2));

    targets
}

fn superscript_digits(number: u32) -> String {
    let superscripts = vec!["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];

    let num_str = number.to_string();
    let mut superscript_str = String::new();

    for digit in num_str.chars() {
        let digit_value = digit.to_digit(10).unwrap();
        superscript_str.push_str(superscripts[digit_value as usize]);
    }

    superscript_str
}
