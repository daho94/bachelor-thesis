use ch_core::graph::Graph;
use macroquad::prelude::*;

const EARTH_RADIUS: f32 = 6378137.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Graph View".to_string(),
        fullscreen: false,
        window_resizable: false,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn draw_graph(lines: &[(Vec2, Vec2)], bbox: &(Vec2, Vec2)) {
    let screen_size = screen_width().min(screen_height());
    let Vec2 { x: x_min, y: y_min } = bbox.0;
    let Vec2 { x: x_max, y: y_max } = bbox.1;

    // Draw all edges and scale according to screen size
    for (from, to) in lines {
        let from = vec2(
            screen_size * (from.x - x_min) / (x_max - x_min),
            screen_size * (from.y - y_min) / (y_max - y_min),
        );
        let to = vec2(
            screen_size * (to.x - x_min) / (x_max - x_min),
            screen_size * (to.y - y_min) / (y_max - y_min),
        );
        draw_line(from.x, from.y, to.x, to.y, 1.0, BLACK);
    }
}

fn spherical_to_cartesian(lat: f32, lon: f32) -> (f32, f32) {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    (y, x)
}

#[macroquad::main(window_conf)]
async fn main() {
    // let pbf_path = std::env::args().nth(1).expect("No path to PBF file given");
    let pbf_path = r"F:\Dev\uni\BA\bachelor_thesis\crates\osm_reader\data\vaterstetten_pp.osm.pbf";
    let g = Graph::from_pbf(std::path::Path::new(&pbf_path)).unwrap();

    let mut lines = Vec::with_capacity(g.edges.len());

    // Transform all coordinates from spherical to cartesian
    // Also find the bounding box of the graph
    for edge in &g.edges {
        let from = edge.from;
        let to = edge.to;
        let (x_from, y_from) = spherical_to_cartesian(
            g.node(from).unwrap().lat as f32,
            g.node(from).unwrap().lon as f32,
        );
        let (x_to, y_to) = spherical_to_cartesian(
            g.node(to).unwrap().lat as f32,
            g.node(to).unwrap().lon as f32,
        );

        lines.push((vec2(x_from, y_from), vec2(x_to, y_to)));
    }

    let graph_bbox = (
        vec2(
            lines
                .iter()
                .map(|(from, to)| from.x.min(to.x))
                .fold(f32::INFINITY, f32::min),
            lines
                .iter()
                .map(|(from, to)| from.y.min(to.y))
                .fold(f32::INFINITY, f32::min),
        ),
        vec2(
            lines
                .iter()
                .map(|(from, to)| from.x.max(to.x))
                .fold(f32::NEG_INFINITY, f32::max),
            lines
                .iter()
                .map(|(from, to)| from.y.max(to.y))
                .fold(f32::NEG_INFINITY, f32::max),
        ),
    );

    loop {
        clear_background(WHITE);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
            });
        });

        // Draw things before egui
        draw_graph(&lines, &graph_bbox);

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
