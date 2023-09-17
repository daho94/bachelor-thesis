use std::{path::Path, sync::Mutex};

use ch_core::{graph::Graph, node_contraction::NodeContractor, util::cli};
use color_theme::ActiveTheme;
use crossbeam_channel::bounded;
use egui::{Style, Visuals};
use macroquad::prelude::*;
use once_cell::sync::Lazy;
use widgets::MyWidget;

mod color_theme;
mod graph_view;
mod widgets;

fn window_conf() -> Conf {
    Conf {
        window_title: "GraphViz".to_string(),
        fullscreen: false,
        window_resizable: true,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
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

static COLOR_THEME: Lazy<Mutex<ActiveTheme>> = Lazy::new(|| Mutex::new(ActiveTheme::default()));

#[macroquad::main(window_conf)]
async fn main() {
    egui_logger::init().unwrap();

    let cfg = cli::parse();

    let mut zoom = 1.0;
    let mut pan = Vec2::default();
    let mut draggable = Draggable::new(vec2(0.0, 0.0));

    println!("Config: {:?}", &cfg);

    // Add channels to communicate between widgets and view
    let (tx_graph, rx_graph) = bounded(1);

    let (tx_debug, rx_debug) = bounded(1);

    let (tx_search, rx_search) = bounded(2);

    let mut graph_view: Option<graph_view::GraphView> = None;
    let mut overlay_graph;

    // Channel for node contraction process. The sender will notify the main thread when the contraction is done.
    let (tx_contraction, rx_contraction) = bounded(1);

    std::thread::spawn(move || {
        let pbf_path = cfg.pbf_file;

        let mut g = if cfg.simplify {
            Graph::from_pbf_with_simplification(Path::new(&pbf_path)).unwrap()
        } else {
            Graph::from_pbf(Path::new(&pbf_path)).unwrap()
        };

        let mut node_contractor = NodeContractor::new_with_params(&mut g, cfg.params);

        let overlay_graph = node_contractor.run_with_strategy(cfg.strategy);
        tx_contraction.send(overlay_graph).unwrap();
    });

    // Init egui widgets
    let mut debug_widget = widgets::debug::DebugWidget::new(rx_debug);
    let mut user_input: Option<widgets::interaction::UserInputWidget> = None;

    loop {
        clear_background(COLOR_THEME.lock().unwrap().bg_color());

        draggable.update(&mut pan, zoom);

        let move_factor = 5.;
        if is_key_down(KeyCode::W) {
            pan.y += move_factor;
        }
        if is_key_down(KeyCode::S) {
            pan.y -= move_factor;
        }
        if is_key_down(KeyCode::A) {
            pan.x += move_factor;
        }
        if is_key_down(KeyCode::D) {
            pan.x -= move_factor;
        }

        if is_key_down(KeyCode::R) {
            zoom = 1.0;
            pan = Vec2::default();

            if let Some(graph_view) = graph_view.as_mut() {
                graph_view.reset();
            }
        }

        // Zoom in and out with mouse wheel
        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                zoom = 1. + y.signum() * 0.05;
            }
            _ => (),
        }

        if let Some(graph_view) = graph_view.as_mut() {
            // Render graph if node contraction is done
            graph_view.update(zoom, pan);
        } else if let Ok(og) = rx_contraction.try_recv() {
            // If thread is done, create graph view and user input widget
            overlay_graph = Some(og);
            graph_view = Some(graph_view::GraphView::new(
                overlay_graph.as_ref().unwrap(),
                rx_graph.clone(),
                tx_debug.clone(),
                tx_search.clone(),
                rx_search.clone(),
            ));
            user_input = Some(widgets::interaction::UserInputWidget::new(
                overlay_graph.as_ref().unwrap(),
                tx_graph.clone(),
                rx_search.clone(),
                tx_search.clone(),
            ));
        }

        egui_macroquad::ui(|egui_ctx| {
            let style = egui::Style {
                visuals: if COLOR_THEME.lock().unwrap().is_dark_theme {
                    Visuals::dark()
                } else {
                    Visuals::light()
                },
                ..Style::default()
            };
            egui_ctx.set_style(style);
            debug_widget.update(egui_ctx);
            if let Some(user_input) = &mut user_input {
                user_input.update(egui_ctx);
            }

            egui::Window::new("Log").show(egui_ctx, |ui| {
                egui_logger::logger_ui(ui);
            });
        });

        // Draw things before egui

        zoom = 1.0;
        pan = Vec2::default();

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
