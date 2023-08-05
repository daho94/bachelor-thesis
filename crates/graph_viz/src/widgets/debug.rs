use std::time::Instant;

use ch_core::graph::Node;
use crossbeam_channel::Receiver;
use egui::{
    plot::{Line, Plot, PlotPoints},
    Color32, Context, Ui, Vec2, Window,
};
use macroquad::texture::get_screen_data;

const FPS_LINE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

pub(crate) struct DebugWidget {
    fps: f64,
    fps_history: Vec<f64>,
    last_update_time: Instant,
    frames_last_time_span: usize,
    debug_info: DebugInfo,
    rx: Receiver<DebugInfo>,
}

pub(crate) struct NodeInfo {
    pub node: Node,
    pub rank: usize,
    // Neighbors...
}

#[derive(Default)]
pub(crate) struct DebugInfo {
    pub node_info: Option<NodeInfo>,
}

impl DebugWidget {
    pub(crate) fn new(rx: Receiver<DebugInfo>) -> Self {
        Self {
            fps: 0.,
            fps_history: Default::default(),
            last_update_time: Instant::now(),
            frames_last_time_span: 0,
            debug_info: Default::default(),
            rx,
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
            .height(50.)
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

    fn draw_node_info(&self, ui: &mut Ui) {
        ui.label("Selected Node");
        ui.separator();
        if let Some(node_info) = &self.debug_info.node_info {
            ui.label(format!("Node (OSMID): {}", node_info.node.id));
            ui.label(format!("Lat: {:.7}°", node_info.node.lat));
            ui.label(format!("Lon: {:.7}°", node_info.node.lon));
            ui.label(format!("Level: {}", node_info.rank));
        } else {
            ui.label("Nothing selected");
        }
    }
}

impl super::MyWidget for DebugWidget {
    fn update(&mut self, ctx: &Context) {
        self.update_fps();

        if let Ok(debug_info) = self.rx.try_recv() {
            self.debug_info = debug_info;
        }

        Window::new("Debug").default_open(false).show(ctx, |ui| {
            if (ui.button("Screenshot")).clicked() {
                let image = get_screen_data();
                image.export_png("screenshot.png");
            }
            ui.add_space(10.);

            self.draw_node_info(ui);

            ui.add_space(10.);

            ui.label(format!("FPS: {:.2}", self.fps));
            ui.separator();

            self.draw_fps(ui);
        });
    }
}
