use std::time::Instant;

use egui::{
    plot::{Line, Plot, PlotPoints},
    Color32, Context, Ui, Vec2, Window,
};

const FPS_LINE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

pub(crate) struct DebugWidget {
    fps: f64,
    fps_history: Vec<f64>,
    last_update_time: Instant,
    frames_last_time_span: usize,
}

impl DebugWidget {
    pub(crate) fn new() -> Self {
        Self {
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

    pub(crate) fn update(&mut self, ctx: &Context) {
        self.update_fps();

        Window::new("Debug").show(ctx, |ui| {
            ui.label(format!("FPS: {:.2}", self.fps));
            self.draw_fps(ui);
        });
    }
}
