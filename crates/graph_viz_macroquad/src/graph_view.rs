use ch_core::{
    graph::{DefaultIdx, Graph, Node},
    overlay_graph::OverlayGraph,
    search::shortest_path::ShortestPath,
};
use egui::epaint::ahash::HashSet;
use macroquad::prelude::*;

use std::{
    collections::BinaryHeap,
    f32::{MAX, MIN},
    sync::mpsc::Receiver,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchResult {
    pub sp: ShortestPath<DefaultIdx>,
}

#[derive(Debug, Clone)]
pub(crate) struct GraphViewOptions {
    pub draw_shortcuts: bool,
    pub draw_nodes: bool,
    pub draw_graph_upward: bool,
    pub draw_graph_downward: bool,
    pub draw_shortest_path: bool,
    pub search_result: Option<SearchResult>,
}

impl Default for GraphViewOptions {
    fn default() -> Self {
        Self {
            draw_shortcuts: false,
            draw_nodes: true,
            draw_graph_upward: false,
            draw_graph_downward: false,
            draw_shortest_path: true,
            search_result: None,
        }
    }
}

pub(crate) struct GraphView<'a> {
    first_frame: bool,
    rect: Rect,
    g: &'a Graph,
    overlay_graph: &'a OverlayGraph,
    options: GraphViewOptions,
    rx: Receiver<GraphViewOptions>,
}

impl<'a> GraphView<'a> {
    pub(crate) fn new(overlay_graph: &'a OverlayGraph, rx: Receiver<GraphViewOptions>) -> Self {
        Self {
            first_frame: true,
            rect: Rect::new(0., 0., 0., 0.),
            g: &overlay_graph.road_graph(),
            overlay_graph,
            rx,
            options: Default::default(),
        }
    }

    pub(crate) fn update(&mut self, zoom: f32, pan: Vec2) {
        if self.first_frame {
            self.first_frame = false;
            self.rect = self.calculate_bbox();
        }

        self.handle_zoom(zoom);
        self.handle_pan(pan);

        // dbg!(start.elapsed());
        if let Ok(options) = self.rx.try_recv() {
            self.options = options;
        }
        self.draw_edges();
        if self.options.draw_nodes {
            self.draw_nodes();
        }

        // Draw shortest path
        if let Some(SearchResult { sp }) = &self.options.search_result {
            let scale_x = screen_width() / self.rect.w;
            let scale_y = screen_height() / self.rect.h;

            // Use smallest scale to avoid distortion
            let scale = scale_x.min(scale_y);

            // Draw SHORTEST PATH
            if self.options.draw_shortest_path {
                let mut windows = sp.nodes.windows(2);

                while let Some(&[source, target]) = windows.next() {
                    let from = node_to_vec(self.g.node(source).unwrap());
                    let to = node_to_vec(self.g.node(target).unwrap());

                    let from = (from - self.rect.point()) * scale;
                    let to = (to - self.rect.point()) * scale;

                    draw_line(
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        2.0,
                        Color::from_rgba(255, 0, 255, 255),
                    );
                }
            }

            // Draw upward
            if self.options.draw_graph_upward {
                let mut edges_to_draw = HashSet::default();
                let mut queue = BinaryHeap::new();
                queue.push(sp.nodes[0]);
                while !queue.is_empty() {
                    let node = queue.pop().unwrap();
                    for (edge_idx, edge) in self.overlay_graph.edges_fwd(node) {
                        if edges_to_draw.insert(edge_idx) {
                            queue.push(edge.target);
                        }
                    }
                }
                for edge_idx in &edges_to_draw {
                    let edge = self.overlay_graph.edge(*edge_idx);
                    let from = node_to_vec(self.g.node(edge.source).unwrap());
                    let to = node_to_vec(self.g.node(edge.target).unwrap());

                    let from = (from - self.rect.point()) * scale;
                    let to = (to - self.rect.point()) * scale;

                    draw_line_with_arrow(
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        1.0,
                        Color::from_rgba(0, 255, 255, 125),
                    );
                }
            }

            // Draw downward graph
            if self.options.draw_graph_downward {
                let mut edges_to_draw = HashSet::default();
                let mut queue = BinaryHeap::new();
                queue.push(sp.nodes[sp.nodes.len() - 1]);
                while !queue.is_empty() {
                    let node = queue.pop().unwrap();
                    for (edge_idx, edge) in self.overlay_graph.edges_bwd(node) {
                        if edges_to_draw.insert(edge_idx) {
                            queue.push(edge.source);
                        }
                    }
                }
                for edge_idx in &edges_to_draw {
                    let edge = self.overlay_graph.edge(*edge_idx);
                    let from = node_to_vec(self.g.node(edge.source).unwrap());
                    let to = node_to_vec(self.g.node(edge.target).unwrap());

                    let from = (from - self.rect.point()) * scale;
                    let to = (to - self.rect.point()) * scale;

                    draw_line_with_arrow(
                        to.x,
                        to.y,
                        from.x,
                        from.y,
                        1.0,
                        Color::from_rgba(255, 255, 0, 125),
                    );
                }
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        self.first_frame = true;
    }

    fn draw_edges(&self) {
        let scale_x = screen_width() / self.rect.w;
        let scale_y = screen_height() / self.rect.h;

        // Use smallest scale to avoid distortion
        let scale = scale_x.min(scale_y);

        let num_elements = if self.options.draw_shortcuts {
            self.g.edges.len()
        } else {
            self.g.edges.len() - self.g.num_shortcuts
        };

        for (idx, edge) in self.g.edges().take(num_elements).enumerate() {
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
                if idx > self.g.edges.len() - self.g.num_shortcuts {
                    Color::from_rgba(255, 20, 20, 125)
                } else {
                    Color::from_rgba(128, 128, 128, 255)
                },
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

            draw_circle(pos.x, pos.y, 2.0, WHITE);
            // draw_rectangle(pos.x - 0.5, pos.y - 0.5, 1.0, 1.0, WHITE);
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

#[allow(dead_code)]
fn spherical_to_cartesian(lat: f64, lon: f64) -> (f32, f32) {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    let x = lat.cos() * lon.cos();
    let y = lat.cos() * lon.sin();
    (y as f32, x as f32)
}

fn node_to_vec(node: &Node) -> Vec2 {
    // let (x, y) = spherical_to_cartesian(node.lat, node.lon);
    let (y, x) = (-node.lat, node.lon);
    vec2(x as f32, y as f32)
}

fn draw_line_with_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    draw_line(x1, y1, x2, y2, thickness, color);

    // Draw arrow at the 0.9 end of the line
    let arrow_length = 10.0;
    let arrow_angle = (y2 - y1).atan2(x2 - x1);

    let line_length = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

    // Calculate arrow endpoints
    let x2 = x2 - 0.1 * line_length * arrow_angle.cos();
    let y2 = y2 - 0.1 * line_length * arrow_angle.sin();

    // Calculate arrow endpoints
    let arrow_x1 = x2;
    let arrow_y1 = y2;
    let arrow_x2 = x2 - arrow_length * (arrow_angle + 0.5).cos();
    let arrow_y2 = y2 - arrow_length * (arrow_angle + 0.5).sin();
    let arrow_x3 = x2 - arrow_length * (arrow_angle - 0.5).cos();
    let arrow_y3 = y2 - arrow_length * (arrow_angle - 0.5).sin();

    // Draw the arrow
    draw_triangle(
        vec2(arrow_x1, arrow_y1),
        vec2(arrow_x2, arrow_y2),
        vec2(arrow_x3, arrow_y3),
        color,
    );
}
