use std::sync::mpsc::Sender;

use ch_core::{
    graph::{DefaultIdx, NodeIndex},
    search::{dijkstra::Dijkstra, shortest_path::ShortestPath},
};
use egui::{Context, Window};

pub(crate) struct UserInputWidget<'g> {
    source_text: String,
    target_text: String,

    source_node: Option<NodeIndex>,
    target_node: Option<NodeIndex>,

    dijkstra: &'g mut Dijkstra<'g>,
    tx: Sender<ShortestPath<DefaultIdx>>,
}

impl<'g> UserInputWidget<'g> {
    pub(crate) fn new(
        dijkstra: &'g mut Dijkstra<'g>,
        tx: Sender<ShortestPath<DefaultIdx>>,
    ) -> UserInputWidget<'g> {
        UserInputWidget {
            source_text: "".to_string(),
            target_text: "".to_string(),

            source_node: None,
            target_node: None,

            dijkstra,
            tx,
        }
    }
}

impl<'g> super::MyWidget for UserInputWidget<'g> {
    fn update(&mut self, ctx: &Context) {
        Window::new("User Input").show(ctx, |ui| {
            ui.label("User Input");

            // Input fields for start and end node
            ui.horizontal(|ui| {
                ui.label("Start Node");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.source_text).hint_text("Enter NodeId"),
                );
                // if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || response.lost_focus()
                    || response.clicked_elsewhere()
                {
                    if let Ok(source) = self.source_text.parse::<usize>() {
                        self.source_node = Some(NodeIndex::new(source));
                    } else {
                        self.source_text = "".to_string();
                        self.source_node = None;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("End Node");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.target_text).hint_text("Enter NodeId"),
                );

                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || response.lost_focus()
                    || response.clicked_elsewhere()
                {
                    if let Ok(target) = self.target_text.parse::<usize>() {
                        self.target_node = Some(NodeIndex::new(target));
                    } else {
                        self.target_text = "".to_string();
                        self.target_node = None;
                    }
                }
            });

            // Button to start search
            if ui
                .add_enabled(
                    self.source_node.is_some() && self.target_node.is_some(),
                    egui::Button::new("Start Search"),
                )
                .on_disabled_hover_text("Enter valid start and end node first")
                .clicked()
            {
                log::debug!("Search button clicked.");
                if let Some(sp) = self
                    .dijkstra
                    .search(self.source_node.unwrap(), self.target_node.unwrap())
                {
                    self.tx.send(sp).unwrap();
                }
            }
        });
    }
}
