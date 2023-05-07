use std::f32::{MAX, MIN};

use ch_core::graph::Graph;
use macroquad::prelude::*;

mod widgets;

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

fn spherical_to_cartesian(lat: f32, lon: f32) -> (f32, f32) {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    (y, x)
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

    fn update(&mut self, offset: &mut Vec2, _zoom: f32) {
        let (x, y) = mouse_position();
        let mouse_position = vec2(x, y);

        if is_mouse_button_down(MouseButton::Right) {
            if !self.is_dragging {
                self.is_dragging = true;
                self.last_mouse_position = mouse_position;
            } else {
                let displacement = mouse_position - self.last_mouse_position;

                offset.x += displacement.x;
                offset.y += displacement.y;

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

    let g = Graph::<u32>::from_pbf(std::path::Path::new(&pbf_path)).unwrap();

    let mut lines = Vec::with_capacity(g.edges_out.len());

    let (mut x_min, mut y_min, mut x_max, mut y_max) = (MAX, MAX, MIN, MIN);
    // Transform all coordinates from spherical to cartesian
    // Also find the bounding box of the graph
    for edge in g.edges() {
        let from = edge.source;
        let to = edge.target;
        let (x_from, y_from) = spherical_to_cartesian(
            g.node(from).unwrap().lat as f32,
            g.node(from).unwrap().lon as f32,
        );
        let (x_to, y_to) = spherical_to_cartesian(
            g.node(to).unwrap().lat as f32,
            g.node(to).unwrap().lon as f32,
        );

        x_min = x_min.min(x_from).min(x_to);
        y_min = y_min.min(y_from).min(y_to);
        x_max = x_max.max(x_from).max(x_to);
        y_max = y_max.max(y_from).max(y_to);

        lines.push((vec2(x_from, y_from), vec2(x_to, y_to)));
    }

    let graph_rect = dbg!(Rect::new(x_min, y_min, x_max - x_min, y_max - y_min));

    let mut zoom = 1.0;
    let mut target = Vec2::default();
    let mut current_rect = graph_rect;

    let mut draggable = Draggable::new(vec2(0.0, 0.0));

    // Init egui widgets
    let mut debug_widget = widgets::debug::DebugWidget::new();

    loop {
        // 1b1b1b
        clear_background(Color::from_rgba(27, 27, 27, 255));

        draggable.update(&mut target, zoom);

        let move_factor = 5.;
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
            current_rect = graph_rect;
        }

        // Zoom in and out with mouse wheel
        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                // Increase zoom speed linearly
                // let zoom_factor = 0.05 * zoom;
                // let new_zoom: f32 = zoom - y.signum() * zoom_factor;
                // zoom = new_zoom.clamp(0.0, 1.0);
                let zoom = 1. + y.signum() * 0.05;
                handle_zoom(&mut current_rect, zoom);
            }
            _ => (),
        }

        // Recalculate visible bbox
        // visible_bbox = calc_bbox(&graph_bbox, target, zoom);

        egui_macroquad::ui(|egui_ctx| {
            debug_widget.update(egui_ctx);
        });

        // Draw things before egui

        // Handle pan
        handle_pan(&mut current_rect, &mut target);

        // Handle zoom
        draw_graph(&lines, &current_rect);

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}

fn draw_graph(lines: &[(Vec2, Vec2)], rect: &Rect) {
    let scale_x = screen_width() / rect.w;
    let scale_y = screen_height() / rect.h;

    // Use smallest scale to avoid distortion
    let scale = scale_x.min(scale_y);

    let mut rendered_lines = 0;
    // Draw all edges and scale according to screen size
    for (from, to) in lines.iter() {
        // Move every point by target
        let from = vec2((from.x - rect.x) * scale, (from.y - rect.y) * scale);
        let to = vec2((to.x - rect.x) * scale, (to.y - rect.y) * scale);

        // Only render lines where from or to point is inside screen
        if (from.x < 0.0 || from.x > screen_width() || from.y < 0.0 || from.y > screen_height())
            && (to.x < 0.0 || to.x > screen_width() || to.y < 0.0 || to.y > screen_height())
        {
            continue;
        }

        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            1.0,
            Color::from_rgba(128, 128, 128, 255),
        );

        rendered_lines += 1;
    }
}

fn handle_zoom(rect: &mut Rect, zoom: f32) {
    let center = rect.center();

    rect.scale(zoom, zoom);
    let new_center = rect.center();
    let diff = center - new_center;
    rect.move_to(diff + vec2(rect.x, rect.y));
}

fn handle_pan(rect: &mut Rect, offset: &mut Vec2) {
    let scale_x = screen_width() / rect.w;
    let scale_y = screen_height() / rect.h;

    // Use smallest scale to avoid distortion
    let scale = scale_x.min(scale_y);
    rect.move_to(-*offset / scale + vec2(rect.x, rect.y));
    offset.x = 0.;
    offset.y = 0.;
}
