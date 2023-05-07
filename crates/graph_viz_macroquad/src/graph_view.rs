use ch_core::graph::{Graph, Node};
use macroquad::prelude::*;

use std::f32::{MAX, MIN};
pub(crate) struct GraphView<'a> {
    first_frame: bool,
    rect: Rect,
    g: &'a Graph,
}

impl<'a> GraphView<'a> {
    pub(crate) fn new(g: &'a Graph) -> Self {
        Self {
            first_frame: true,
            rect: Rect::new(0., 0., 0., 0.),
            g,
        }
    }

    pub(crate) fn update(&mut self, zoom: f32, pan: Vec2) {
        if self.first_frame {
            self.first_frame = false;
            self.rect = self.calculate_bbox();
        }

        self.handle_zoom(zoom);
        self.handle_pan(pan);

        self.draw_edges();
        self.draw_nodes();
    }

    pub(crate) fn reset(&mut self) {
        self.first_frame = true;
    }

    fn draw_edges(&self) {
        let scale_x = screen_width() / self.rect.w;
        let scale_y = screen_height() / self.rect.h;

        // Use smallest scale to avoid distortion
        let scale = scale_x.min(scale_y);

        for edge in self.g.edges() {
            let from = node_to_vec(self.g.node(edge.source).unwrap());
            let to = node_to_vec(self.g.node(edge.target).unwrap());

            let from = (from - self.rect.point()) * scale;
            let to = (to - self.rect.point()) * scale;

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
        }
    }

    fn draw_nodes(&self) {
        let scale_x = screen_width() / self.rect.w;
        let scale_y = screen_height() / self.rect.h;

        // Use smallest scale to avoid distortion
        let scale = scale_x.min(scale_y);

        for node in self.g.nodes() {
            let pos = node_to_vec(node);

            let pos = (pos - self.rect.point()) * scale;

            // Only render lines where from or to point is inside screen
            if pos.x < 0.0 || pos.x > screen_width() || pos.y < 0.0 || pos.y > screen_height() {
                continue;
            }

            draw_circle(pos.x, pos.y, 1.0, RED);
        }
    }

    fn calculate_bbox(&self) -> Rect {
        let (mut x_min, mut y_min, mut x_max, mut y_max) = (MAX, MAX, MIN, MIN);
        for node in self.g.nodes() {
            let Vec2 { x, y } = node_to_vec(node);
            x_min = x_min.min(x);
            y_min = y_min.min(y);
            x_max = x_max.max(x);
            y_max = y_max.max(y);
        }

        Rect::new(x_min, y_min, x_max - x_min, y_max - y_min)
    }

    fn handle_pan(&mut self, pan: Vec2) {
        let scale_x = screen_width() / self.rect.w;
        let scale_y = screen_height() / self.rect.h;

        // Use smallest scale to avoid distortion
        let scale = scale_x.min(scale_y);
        self.rect
            .move_to(-pan / scale + vec2(self.rect.x, self.rect.y));
    }

    fn handle_zoom(&mut self, zoom: f32) {
        let center = self.rect.center();

        self.rect.scale(zoom, zoom);
        let new_center = self.rect.center();
        let diff = center - new_center;
        self.rect.move_to(diff + vec2(self.rect.x, self.rect.y));
    }
}

fn spherical_to_cartesian(lat: f64, lon: f64) -> (f32, f32) {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    (y as f32, x as f32)
}

fn node_to_vec(node: &Node) -> Vec2 {
    // let (x, y) = spherical_to_cartesian(node.lat, node.lon);
    let (y, x) = (node.lat, node.lon);
    vec2(x as f32, y as f32)
}
