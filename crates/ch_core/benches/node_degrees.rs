use std::path::Path;

use ch_core::{graph::Graph, statistics};
use plotly::{
    color::NamedColor,
    common::{Anchor, Font, Orientation, Side, Title},
    layout::{self, Axis, GridPattern, LayoutGrid, Legend, Margin},
    Bar, ImageFormat, Layout, Plot,
};

fn main() {
    // Path to:
    // - saarland-latest.osm.pbf
    // - bavaria-latest.osm.pbf
    // - new-york-latest.osm.pbf
    // - germany-latest.osm.pbf
    let paths = std::env::args()
        .skip(1)
        .filter(|p| p.ends_with("pbf"))
        .collect::<Vec<String>>();

    let mut plot = Plot::new();
    let mut show_legend = true;
    let mut num_plots = 0;
    for p in &paths {
        num_plots += 1;
        dbg!(&p);
        let (trace, trace_simplified) = create_traces_out(Path::new(&p));
        let x = format!("x{}", num_plots);
        let y = format!("y{}", num_plots);

        plot.add_trace(trace.x_axis(&x).y_axis(&y).show_legend(show_legend));
        plot.add_trace(
            trace_simplified
                .x_axis(x)
                .y_axis(y)
                .show_legend(show_legend),
        );
        show_legend = false;
    }

    let layout = Layout::new()
        .grid(
            LayoutGrid::new()
                .rows(num_plots / 2)
                .columns(2)
                .pattern(GridPattern::Independent),
        )
        .legend(Legend::new().orientation(Orientation::Horizontal))
        .x_axis(
            Axis::new()
                .fixed_range(true)
                .range(vec![0, 1, 2, 3, 4, 5, 6])
                .title(Title::new("Saarland").font(Font::new().size(12))),
        )
        .x_axis2(
            Axis::new()
                .fixed_range(true)
                .range(vec![0, 1, 2, 3, 4, 5, 6])
                .title(Title::new("Bayern").font(Font::new().size(12))),
        )
        .x_axis3(
            Axis::new()
                .fixed_range(true)
                .range(vec![0, 1, 2, 3, 4, 5, 6])
                .title(
                    Title::new("Ney York")
                        .y_anchor(Anchor::Bottom)
                        .side(Side::Top)
                        .font(Font::new().size(12)),
                ),
        )
        .x_axis4(
            Axis::new()
                .fixed_range(true)
                .range(vec![0, 1, 2, 3, 4, 5, 6])
                .title(
                    Title::new("Deutschland")
                        .y_anchor(Anchor::Bottom)
                        .font(Font::new().size(12)),
                ),
        )
        .colorway(vec![NamedColor::Gray, NamedColor::LightGray])
        .margin(Margin::default().top(8).bottom(8));

    plot.set_layout(layout.clone());

    plot.write_image("hist_deg_out.png", ImageFormat::PDF, 800, 600, 1.0);

    let mut plot = Plot::new();
    let mut show_legend = true;

    for p in paths {
        dbg!(&p);
        let (trace, trace_simplified) = create_traces_in(Path::new(&p));
        let x = format!("x{}", num_plots);
        let y = format!("y{}", num_plots);

        plot.add_trace(trace.x_axis(&x).y_axis(&y).show_legend(show_legend));
        plot.add_trace(
            trace_simplified
                .x_axis(x)
                .y_axis(y)
                .show_legend(show_legend),
        );
        show_legend = false;
    }

    plot.set_layout(layout);

    plot.write_image("hist_deg_in.png", ImageFormat::PDF, 800, 600, 1.0);
}

#[allow(dead_code)]
fn draw_single_graph() {
    let path = std::env::args().nth(1).unwrap();
    let path = Path::new(&path);
    let (trace_out, trace_out_simplified) = create_traces_out(path);

    let mut plot = Plot::new();
    plot.add_trace(trace_out);
    plot.add_trace(trace_out_simplified);

    let layout = Layout::new()
        .legend(
            Legend::new()
                .orientation(Orientation::Horizontal)
                .valign(layout::VAlign::Middle),
        )
        .margin(Margin::default().top(8));
    plot.set_layout(layout.clone());

    plot.write_image("hist_deg_out.png", ImageFormat::PDF, 600, 400, 1.0);

    let (trace_in, trace_in_simplified) = create_traces_in(path);

    let mut plot = Plot::new();
    plot.add_trace(trace_in);
    plot.add_trace(trace_in_simplified);

    plot.set_layout(layout);

    plot.write_image("hist_deg_in.png", ImageFormat::PDF, 600, 400, 1.0);
}

#[allow(clippy::type_complexity)]
fn create_traces(p: &Path, out_degree: bool) -> (Box<Bar<String, u32>>, Box<Bar<String, u32>>) {
    let degree_hist = if out_degree {
        statistics::degree_out_hist
    } else {
        statistics::degree_in_hist
    };

    let g = Graph::from_pbf(p).unwrap();

    let hist = degree_hist(&g);

    let mut x = Vec::new();
    let mut y = Vec::new();

    for bucket in hist.into_iter().filter(|b| b.count() > 1 && b.low() < 7) {
        x.push(bucket.low().to_string());
        y.push(bucket.count());
    }

    let trace = Bar::new(x.clone(), y).name("Normal");

    let g = Graph::from_pbf_with_simplification(p).unwrap();

    let hist = degree_hist(&g);

    let mut x = Vec::new();
    let mut y = Vec::new();

    for bucket in hist.into_iter().filter(|b| b.count() > 1 && b.low() < 7) {
        x.push(bucket.low().to_string());
        y.push(bucket.count());
    }

    let trace_simplified = Bar::new(x.clone(), y).name("Vereinfacht");

    (trace, trace_simplified)
}

#[allow(clippy::type_complexity)]
fn create_traces_out(p: &Path) -> (Box<Bar<String, u32>>, Box<Bar<String, u32>>) {
    create_traces(p, true)
}

#[allow(clippy::type_complexity)]
fn create_traces_in(p: &Path) -> (Box<Bar<String, u32>>, Box<Bar<String, u32>>) {
    create_traces(p, false)
}
