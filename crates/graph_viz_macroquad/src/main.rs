use std::f32::MAX;

use ch_core::graph::Graph;
use egui::gui_zoom::zoom_in;
use macroquad::prelude::*;

const EARTH_RADIUS: f32 = 6378137.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Graph View".to_string(),
        fullscreen: false,
        window_resizable: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn draw_graph(lines: &[(Vec2, Vec2)], bbox: &(Vec2, Vec2), zoom: f32, target: Vec2) {
    let screen_size = screen_width().min(screen_height());
    let Vec2 { x: x_min, y: y_min } = bbox.0;
    let Vec2 { x: x_max, y: y_max } = bbox.1;

    let width = x_max - x_min;
    let height = y_max - y_min;

    // let scale_x = screen_width() / width / zoom;
    // let scale_y = screen_height() / height / zoom;
    let scale_x = screen_width() / width / zoom;
    let scale_y = screen_height() / height / zoom;

    let scale = scale_x.min(scale_y);

    // let graph_center = vec2((x_max + x_min) / 2.0, (y_max + y_min) / 2.0);
    // https://github.com/blitzarx1/egui_graphs/blob/master/src/graph_view.rs#L302

    let offset_x = target.x + (screen_width() - (width) * scale) / 2.0;
    let offset_y = target.y + (screen_height() - (height) * scale) / 2.0;
    // let offset_x = target.x;
    // let offset_y = target.y;

    let mut rendered_lines = 0;
    // Draw all edges and scale according to screen size
    for (from, to) in lines.iter() {
        // Move every point by target
        let from = vec2(
            (from.x - x_min) * scale + offset_x,
            (from.y - y_min) * scale + offset_y,
        );
        let to = vec2(
            (to.x - x_min) * scale + offset_x,
            (to.y - y_min) * scale + offset_y,
        );

        // Only render lines where from or to point is inside screen
        if (from.x < 0.0 || from.x > screen_width() || from.y < 0.0 || from.y > screen_height())
            && (to.x < 0.0 || to.x > screen_width() || to.y < 0.0 || to.y > screen_height())
        {
            continue;
        }

        draw_line(from.x, from.y, to.x, to.y, 1.0, GRAY);

        rendered_lines += 1;
    }
    draw_circle(screen_width() / 2., screen_height() / 2., 5., RED);
}

fn spherical_to_cartesian(lat: f32, lon: f32) -> (f32, f32) {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    (y, x)
}

fn calc_bbox(graph_bbox: &(Vec2, Vec2), target: Vec2, zoom: f32) -> (Vec2, Vec2) {
    // visible_bbox shrinks when zooming
    let old_width = graph_bbox.1.x - graph_bbox.0.x;
    let old_height = graph_bbox.1.y - graph_bbox.0.y;

    let new_width = old_width * zoom;
    let new_height = old_height * zoom;
    (
        vec2(
            graph_bbox.0.x + target.x + (old_width - new_width) / 2.0,
            graph_bbox.0.y + target.y + (old_height - new_height) / 2.0,
        ),
        vec2(
            graph_bbox.1.x + target.x - (old_width - new_width) / 2.0,
            graph_bbox.1.y + target.y - (old_height - new_height) / 2.0,
        ),
    )
}
#[derive(Debug)]
struct Draggable {
    position: Vec2,
    is_dragging: bool,
    last_mouse_position: Vec2,
}

impl Draggable {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            is_dragging: false,
            last_mouse_position: Vec2::default(),
        }
    }

    fn update(&mut self, bbox: &(Vec2, Vec2), target: &mut Vec2, zoom: f32) {
        let (x, y) = mouse_position();
        let mouse_position = vec2(x, y);

        if is_mouse_button_down(MouseButton::Right) {
            if !self.is_dragging {
                self.is_dragging = true;
                self.last_mouse_position = mouse_position;
            } else {
                let displacement = mouse_position - self.last_mouse_position;

                target.x += displacement.x;
                target.y += displacement.y;

                self.position += displacement;
                self.last_mouse_position = mouse_position;
            }
        } else {
            self.is_dragging = false;
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let pbf_path = std::env::args().nth(1).unwrap_or(
        r"F:\Dev\uni\BA\bachelor_thesis\crates\osm_reader\data\vaterstetten_pp.osm.pbf".into(),
    );

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

    // Camera settings
    let mut zoom = 1.0;
    let mut target = Vec2::default();
    let mut visible_bbox = calc_bbox(&graph_bbox, target, zoom);

    let mut draggable = Draggable::new(vec2(0.0, 0.0));
    loop {
        dbg!(&target);
        clear_background(DARKGRAY);

        draggable.update(&visible_bbox, &mut target, zoom);

        let mut move_factor = 5.;
        if is_key_down(KeyCode::W) {
            target.y += move_factor;
        }
        if is_key_down(KeyCode::S) {
            target.y -= move_factor;
        }
        if is_key_down(KeyCode::A) {
            target.x += move_factor;
        }
        if is_key_down(KeyCode::D) {
            target.x -= move_factor;
        }

        if is_key_down(KeyCode::R) {
            zoom = 1.0;
            target = Vec2::default();
        }

        // Zoom in and out with mouse wheel
        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                if is_key_down(KeyCode::LeftControl) {
                    // Increase zoom speed linearly
                    let zoom_factor = 0.05 * zoom;
                    let new_zoom: f32 = zoom - y.signum() * zoom_factor;
                    zoom = new_zoom.clamp(0.0, 1.0);

                    // let (x, y) = mouse_position();
                    // let displacement = mouse_position - draggable.last_mouse_position;

                    // target.x = x;
                    // target.y = y;
                    // target = Vec2::default();

                    // draggable.position += displacement;
                    // draggable.last_mouse_position = mouse_position;
                }
            }
            _ => (),
        }

        // Recalculate visible bbox
        // visible_bbox = calc_bbox(&graph_bbox, target, zoom);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
            });
        });

        // Draw things before egui
        draw_graph(&lines, &visible_bbox, zoom, target);

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
