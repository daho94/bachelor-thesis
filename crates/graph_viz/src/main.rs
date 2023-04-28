use std::{collections::HashMap, time::Instant};

use ch_core::graph::Graph;
use eframe::{run_native, App, CreationContext};
use egui::{
    plot::{Line, Plot, PlotPoints},
    CollapsingHeader, Color32, Context, ScrollArea, Ui, Vec2,
};
use egui_graphs::{Edge, Elements, GraphView, Node, SettingsNavigation};

const EARTH_RADIUS: f32 = 6_371_000.;
const EDGE_SCALE_WEIGHT: f32 = 0.5;
const NODE_RADIUS: f32 = 0.0;
const EDGE_TIP_SIZE: f32 = 5.0;
const FPS_LINE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

pub struct BasicApp {
    elements: Elements,
    settings_navigation: SettingsNavigation,

    fps: f64,
    fps_history: Vec<f64>,
    last_update_time: Instant,
    frames_last_time_span: usize,
}

impl BasicApp {
    fn new(_: &CreationContext<'_>, graph: Graph) -> Self {
        let elements = into_elements(graph);
        Self {
            elements,
            settings_navigation: Default::default(),

            fps: 0.,
            fps_history: Default::default(),
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
        }
    }

    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f64 / elapsed.as_secs_f64();
            self.frames_last_time_span = 0;

            self.fps_history.push(self.fps);
            if self.fps_history.len() > 100 {
                self.fps_history.remove(0);
            }
        }
    }
    fn draw_fps(&self, ui: &mut Ui) {
        let points: PlotPoints = self
            .fps_history
            .iter()
            .enumerate()
            .map(|(i, val)| [i as f64, *val])
            .collect();

        let line = Line::new(points).color(FPS_LINE_COLOR);
        Plot::new("my_plot")
            .min_size(Vec2::new(100., 50.))
            .show_x(false)
            .show_background(false)
            .show_axes([false, false])
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}

impl App for BasicApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.update_fps();

        egui::SidePanel::right("right_panel")
        .default_width(300.)
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                CollapsingHeader::new("Widget")
                .default_open(true)
                .show(ui, |ui| {
                        ui.add_space(10.);

                        ui.label("NavigationSettings");
                        ui.separator();

                        if ui
                            .checkbox(&mut self.settings_navigation.fit_to_screen, "autofit")
                            .changed()
                            && self.settings_navigation.fit_to_screen
                        {
                            self.settings_navigation.zoom_and_pan = false
                        };
                        ui.label("Enable autofit to fit the graph to the screen on every frame.");

                        ui.add_space(5.);

                        ui.add_enabled_ui(!self.settings_navigation.fit_to_screen, |ui| {
                            ui.vertical(|ui| {
                                ui.checkbox(&mut self.settings_navigation.zoom_and_pan, "pan & zoom");
                                ui.label("Enable pan and zoom. To pan use LMB + drag and to zoom use Ctrl + Mouse Wheel.");
                            }).response.on_disabled_hover_text("disabled autofit to enable pan & zoom");
                        });

                        ui.add_space(10.);

                        CollapsingHeader::new("Debug")
                    .default_open(false)
                    .show(ui, |ui| {
                            ui.add_space(10.);

                            ui.vertical(|ui| {
                                ui.label(format!("fps: {:.1}", self.fps));
                                ui.add_space(10.);
                                self.draw_fps(ui);
                            });
                    });

                    })
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(GraphView::new(&self.elements).with_navigations(&self.settings_navigation));
        });
    }
}

fn into_elements(graph: Graph) -> Elements {
    let mut nodes = HashMap::new();
    let mut edges = HashMap::new();

    for node in graph.nodes {
        let lat = node.lat as f32;
        let lon = node.lon as f32;

        // Transform latitude and longitude into Pseudo-Mercator projection
        let x = lon.to_radians().sin() * lat.to_radians().cos() * EARTH_RADIUS;
        let y = lon.to_radians().cos() * lat.to_radians().cos() * EARTH_RADIUS;

        let mut node = Node::new(node.id, egui::Vec2::new(x, y));
        node.radius = NODE_RADIUS;

        nodes.insert(node.id, node);
    }

    for graph_edge in graph.edges {
        let key = (graph_edge.from, graph_edge.to);
        edges.entry(key).or_insert_with(Vec::new);

        let edge_list = edges.get_mut(&key).unwrap();
        let list_idx = edge_list.len();

        let mut edge = Edge::new(graph_edge.from, graph_edge.to, list_idx);
        edge.tip_size = EDGE_TIP_SIZE;
        edge_list.push(edge);

        nodes.get_mut(&graph_edge.from).unwrap().radius += EDGE_SCALE_WEIGHT;
        nodes.get_mut(&graph_edge.to).unwrap().radius += EDGE_SCALE_WEIGHT;
    }

    Elements::new(nodes, edges)
}

fn main() {
    let pbf_path = std::env::args().nth(1).expect("No path to PBF file given");
    let graph = Graph::from_pbf(std::path::Path::new(&pbf_path)).unwrap();

    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_basic_demo",
        native_options,
        Box::new(|cc| Box::new(BasicApp::new(cc, graph))),
    )
    .unwrap();
}
