use ch_core::{
    graph::{Graph, Node, NodeIndex},
    overlay_graph::OverlayGraph,
    search::shortest_path::ShortestPath,
};
use egui::epaint::ahash::HashSet;
use macroquad::prelude::*;

use crossbeam_channel::{Receiver, Sender};
use rustc_hash::FxHashSet;
use std::{
    collections::BinaryHeap,
    f32::{MAX, MIN},
};

use crate::{
    widgets::debug::{DebugInfo, NodeInfo},
    COLOR_THEME,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchResult {
    pub sp: ShortestPath,
    pub settled_fwd: Option<FxHashSet<NodeIndex>>,
    pub settled_bwd: Option<FxHashSet<NodeIndex>>,
}

#[derive(Debug, Clone)]
pub(crate) struct GraphViewOptions {
    pub draw_shortcuts: bool,
    pub draw_nodes: bool,
    pub draw_graph_upward: bool,
    pub draw_graph_downward: bool,
    pub draw_shortest_path: bool,
    pub search_result: Option<SearchResult>,
    pub node_radius: f32,
}

impl Default for GraphViewOptions {
    fn default() -> Self {
        Self {
            draw_shortcuts: false,
            draw_nodes: false,
            draw_graph_upward: false,
            draw_graph_downward: false,
            draw_shortest_path: true,
            search_result: None,
            node_radius: 2.0,
        }
    }
}

pub(crate) struct GraphView<'a> {
    first_frame: bool,
    rect: Rect,
    g: &'a Graph,
    overlay_graph: &'a OverlayGraph,
    options: GraphViewOptions,

    // Communication channels between widgets
    rx: Receiver<GraphViewOptions>,
    tx_debug: Sender<DebugInfo>,
    tx_search: Sender<(Option<NodeIndex>, Option<NodeIndex>)>,
    rx_search: Receiver<(Option<NodeIndex>, Option<NodeIndex>)>,

    selected_node: Option<NodeIndex>,
    start_node: Option<NodeIndex>,
    target_node: Option<NodeIndex>,
}

impl<'a> GraphView<'a> {
    pub(crate) fn new(
        overlay_graph: &'a OverlayGraph,
        rx: Receiver<GraphViewOptions>,
        tx_debug: Sender<DebugInfo>,
        tx_search: Sender<(Option<NodeIndex>, Option<NodeIndex>)>,
        rx_search: Receiver<(Option<NodeIndex>, Option<NodeIndex>)>,
    ) -> Self {
        Self {
            first_frame: true,
            rect: Rect::new(0., 0., 0., 0.),
            g: overlay_graph.road_graph(),
            overlay_graph,
            rx,
            options: Default::default(),
            tx_debug,
            tx_search,
            rx_search,
            selected_node: None,
            start_node: None,
            target_node: None,
        }
    }

    pub(crate) fn update(&mut self, zoom: f32, pan: Vec2) {
        if self.first_frame {
            self.first_frame = false;
            self.rect = self.calculate_bbox();
        }

        self.handle_zoom(zoom);
        self.handle_pan(pan);
        self.handle_click();

        if let Ok(options) = self.rx.try_recv() {
            self.options = options;
        }

        if let Ok((start, target)) = self.rx_search.try_recv() {
            self.start_node = start;
            self.target_node = target;
        }

        self.draw_edges();

        if self.options.draw_nodes {
            self.draw_nodes();
        }

        // Draw selected node
        if let Some(node_idx) = self.selected_node {
            let scale = self.scale();
            self.draw_node(
                self.g.node(node_idx).unwrap(),
                self.scale(),
                self.options.node_radius,
                ORANGE,
            );

            // Draw connected edges
            for (edge_idx, edge) in self.g.neighbors_incoming(node_idx) {
                let from = node_to_vec(self.g.node(edge.source).unwrap());
                let to = node_to_vec(self.g.node(edge.target).unwrap());

                let from = (from - self.rect.point()) * scale;
                let to = (to - self.rect.point()) * scale;

                if edge_idx.index() >= self.g.edges.len() - self.g.num_shortcuts {
                    draw_line_with_arrow(from.x, from.y, to.x, to.y, 1.0, RED);
                } else {
                    draw_line_with_arrow(from.x, from.y, to.x, to.y, 1.0, ORANGE);
                }
            }

            for (edge_idx, edge) in self.g.neighbors_outgoing(node_idx) {
                let from = node_to_vec(self.g.node(edge.source).unwrap());
                let to = node_to_vec(self.g.node(edge.target).unwrap());

                let from = (from - self.rect.point()) * scale;
                let to = (to - self.rect.point()) * scale;

                if edge_idx.index() >= self.g.edges.len() - self.g.num_shortcuts {
                    draw_line_with_arrow(from.x, from.y, to.x, to.y, 1.0, RED);
                } else {
                    draw_line_with_arrow(from.x, from.y, to.x, to.y, 1.0, ORANGE);
                }
            }
        }

        // Draw shortest path, Upward and Downward graph
        if let Some(SearchResult {
            sp,
            settled_fwd,
            settled_bwd,
        }) = &self.options.search_result
        {
            let scale = self.scale();

            // Draw upward graph
            if self.options.draw_graph_upward {
                if let Some(settled_fwd) = settled_fwd {
                    for node_idx in settled_fwd {
                        let node = self.g.node(*node_idx).unwrap();
                        self.draw_node(
                            node,
                            scale,
                            self.options.node_radius,
                            COLOR_THEME.lock().unwrap().graph_up_color(),
                        );
                    }
                }
            }

            // Draw downward graph
            if self.options.draw_graph_downward {
                if let Some(settled_bwd) = settled_bwd {
                    for node_idx in settled_bwd {
                        let node = self.g.node(*node_idx).unwrap();
                        self.draw_node(
                            node,
                            scale,
                            self.options.node_radius,
                            COLOR_THEME.lock().unwrap().graph_down_color(),
                        );
                    }
                }
            }

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
                        // Color::from_rgba(255, 0, 255, 255),
                        COLOR_THEME.lock().unwrap().sp_color(),
                    );
                }
            }
        }
        // Draw start and target node
        if let Some(node_idx) = self.start_node {
            let mut color = COLOR_THEME.lock().unwrap().graph_up_color();
            color.a = 1.0;
            self.draw_node(self.g.node(node_idx).unwrap(), self.scale(), 4.0, RED);
        }
        if let Some(node_idx) = self.target_node {
            let mut color = COLOR_THEME.lock().unwrap().graph_down_color();
            color.a = 1.0;
            self.draw_node(self.g.node(node_idx).unwrap(), self.scale(), 4.0, BLUE);
        }
    }

    pub(crate) fn reset(&mut self) {
        self.first_frame = true;
    }

    fn closest_node(&self) -> Option<(NodeIndex, &Node)> {
        let (x, y) = mouse_position();
        let mouse_position = vec2(x, y);

        let scale = self.scale();

        let mut closest_node = None;
        let mut closest_node_id = 0;
        let mut closest_distance = MAX;
        for (i, node) in self.g.nodes().enumerate() {
            let pos = node_to_vec(node);

            let pos = (pos - self.rect.point()) * scale;

            let distance = (pos - mouse_position).length();

            if distance < closest_distance {
                closest_distance = distance;
                closest_node = Some(node);
                closest_node_id = i;
            }
        }

        Some((ch_core::graph::node_index(closest_node_id), closest_node?))
    }

    fn handle_click(&mut self) {
        if is_mouse_button_pressed(MouseButton::Middle) {
            if let Some((node_idx, node)) = self.closest_node() {
                log::debug!(
                    "Clicked node: {:?}, Rank: {}",
                    node,
                    self.overlay_graph.node_order[node_idx.index()]
                );
                self.tx_debug
                    .send(DebugInfo {
                        node_info: Some(NodeInfo {
                            node: node.clone(),
                            node_index: node_idx.index(),
                            rank: self.overlay_graph.node_order[node_idx.index()],
                        }),
                    })
                    .unwrap();

                self.selected_node = Some(node_idx);
            }
        }

        if is_key_pressed(KeyCode::R) {
            self.selected_node = None;
            self.start_node = None;
            self.target_node = None;
            self.options.search_result = None;
        }

        if is_key_pressed(KeyCode::Q) {
            // Set start node
            if let Some((node_idx, _)) = self.closest_node() {
                self.tx_search.send((Some(node_idx), None)).unwrap();
            }
        }

        if is_key_pressed(KeyCode::E) {
            // Set end node
            if let Some((node_idx, _)) = self.closest_node() {
                self.tx_search.send((None, Some(node_idx))).unwrap();
            }
        }
    }

    fn draw_edges(&self) {
        let scale = self.scale();

        let num_elements = if self.options.draw_shortcuts {
            self.g.edges.len()
        } else {
            self.g.edges.len() - self.g.num_shortcuts
        };

        let mut rendered_edges = 0;

        for (idx, edge) in self.g.edges().take(num_elements).enumerate() {
            rendered_edges += 1;
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
                    // Color::from_rgba(255, 20, 20, 125)
                    COLOR_THEME.lock().unwrap().shortcut_color()
                } else {
                    // Color::from_rgba(128, 128, 128, 255)
                    COLOR_THEME.lock().unwrap().line_color()
                },
            );
        }
        log::debug!("Rendered edges: {}", rendered_edges);
    }

    fn draw_nodes(&self) {
        let scale = self.scale();

        for node in self.g.nodes() {
            self.draw_node(
                node,
                scale,
                self.options.node_radius,
                COLOR_THEME.lock().unwrap().node_color(),
            );
        }
    }

    fn draw_node(&self, node: &Node, scale: f32, r: f32, color: Color) {
        let pos = node_to_vec(node);

        let pos = (pos - self.rect.point()) * scale;

        // Only render lines where from or to point is inside screen
        if pos.x < 0.0 || pos.x > screen_width() || pos.y < 0.0 || pos.y > screen_height() {
            return;
        }

        draw_circle(pos.x, pos.y, r, color);
    }

    fn scale(&self) -> f32 {
        let scale_x = screen_width() / self.rect.w;
        let scale_y = screen_height() / self.rect.h;

        // Use smallest scale to avoid distortion
        scale_x.min(scale_y)
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
    let (y, x) = (-node.lat, node.lon);
    vec2(x as f32, y as f32)
}

fn draw_line_with_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    draw_line(x1, y1, x2, y2, thickness, color);

    // Draw arrow at the 0.9 end of the line
    let arrow_length = 5.0;
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
