use ch_core::{
    graph::{node_index, Graph},
    node_contraction::NodeContractor,
    util::test_graphs::generate_complex_graph,
};
use graph_view::GraphViewOptions;
use macroquad::prelude::*;
use widgets::MyWidget;

mod graph_view;
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
    env_logger::init();

    let pbf_path = std::env::args().nth(1).unwrap_or(
        r"F:\Dev\uni\BA\bachelor_thesis\crates\osm_reader\data\vaterstetten_pp.osm.pbf".into(),
    );

    let mut g = match pbf_path.as_ref() {
        "test" => generate_complex_graph(),
        _ => Graph::<u32>::from_pbf(std::path::Path::new(&pbf_path)).unwrap(),
    };

    let mut zoom = 1.0;
    let mut pan = Vec2::default();
    let mut draggable = Draggable::new(vec2(0.0, 0.0));

    // let mut g = Graph::<u32>::from_pbf(std::path::Path::new(&pbf_path)).unwrap();

    let mut node_contractor = NodeContractor::new(&mut g);
    let overlay_graph = match pbf_path.as_ref() {
        "test" => node_contractor.run_with_order(&[
            node_index(1),
            node_index(4),
            node_index(8),
            node_index(10),
            node_index(3),
            node_index(6),
            node_index(2),
            node_index(9),
            node_index(7),
            node_index(5),
            node_index(0),
        ]),
        _ => node_contractor.run(),
    };

    // Add channel to communicate between widgets and graph_view
    let (sender, receiver) = std::sync::mpsc::channel::<GraphViewOptions>();

    let mut graph_view = graph_view::GraphView::new(&overlay_graph, receiver);

    // Init egui widgets
    let mut debug_widget = widgets::debug::DebugWidget::new();
    let mut user_input = widgets::interaction::UserInputWidget::new(&overlay_graph, sender);

    loop {
        clear_background(Color::from_rgba(27, 27, 27, 255));

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
            graph_view.reset();
        }

        // Zoom in and out with mouse wheel
        match mouse_wheel() {
            (_x, y) if y != 0.0 => {
                zoom = 1. + y.signum() * 0.05;
            }
            _ => (),
        }

        egui_macroquad::ui(|egui_ctx| {
            debug_widget.update(egui_ctx);
            user_input.update(egui_ctx);
        });

        // Draw things before egui

        graph_view.update(zoom, pan);

        zoom = 1.0;
        pan = Vec2::default();

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}
