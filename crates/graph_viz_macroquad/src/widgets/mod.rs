use egui::Context;

pub(crate) mod debug;
pub(crate) mod interaction;

pub(crate) trait MyWidget {
    fn update(&mut self, ctx: &Context);
}
