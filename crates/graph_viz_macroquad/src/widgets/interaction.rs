use std::sync::mpsc::Sender;

use ch_core::{graph::NodeIndex, overlay_graph::OverlayGraph, search::bidir_search::BiDirSearch};
use egui::{CollapsingHeader, Context, Window};

use crate::graph_view::{GraphViewOptions, SearchResult};

pub(crate) struct UserInputWidget<'g> {
    source_text: String,
    target_text: String,

    source_node: Option<NodeIndex>,
    target_node: Option<NodeIndex>,

    // dijkstra: &'g mut Dijkstra<'g>,
    overlay_graph: &'g OverlayGraph<'g>,
    tx: Sender<GraphViewOptions>,

    options: GraphViewOptions,
}

impl<'g> UserInputWidget<'g> {
    pub(crate) fn new(
        // dijkstra: &'g mut Dijkstra<'g>,
        overlay_graph: &'g OverlayGraph<'g>,
        tx: Sender<GraphViewOptions>,
    ) -> UserInputWidget<'g> {
        UserInputWidget {
            source_text: "".to_string(),
            target_text: "".to_string(),

            source_node: None,
            target_node: None,

            // dijkstra,
            overlay_graph,
            tx,
            options: Default::default(),
        }
    }
}

impl<'g> super::MyWidget for UserInputWidget<'g> {
    fn update(&mut self, ctx: &Context) {
        Window::new("User Input").show(ctx, |ui| {
            CollapsingHeader::new("Graph Options").show(ui, |ui| {
                ui.checkbox(&mut self.options.draw_shortcuts, "Draw shortcuts");
                ui.checkbox(&mut self.options.draw_nodes, "Draw nodes");
                ui.checkbox(&mut self.options.draw_shortest_path, "Draw Shortest Path");
                ui.checkbox(&mut self.options.draw_graph_upward, "Draw Graph Up");
                ui.checkbox(&mut self.options.draw_graph_downward, "Draw Graph Down");
            });

            ui.add_space(10.);
            ui.separator();

            CollapsingHeader::new("Search")
                .default_open(true)
                .show(ui, |ui| {
                    // Input fields for start and end node
                    ui.horizontal(|ui| {
                        ui.label("Source Node");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.source_text)
                                .hint_text("Enter NodeId"),
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
                        ui.label("Target Node");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.target_text)
                                .hint_text("Enter NodeId"),
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

                        let mut bidir_search = BiDirSearch::new(self.overlay_graph);
                        if let Some(sp) = bidir_search
                            .search(self.source_node.unwrap(), self.target_node.unwrap())
                        {
                            self.options.search_result = Some(SearchResult { sp });
                        }
                    }
                });
        });

        self.tx.send(self.options.clone()).unwrap();
    }
}
