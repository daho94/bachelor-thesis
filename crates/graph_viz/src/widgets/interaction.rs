use ch_core::{
    graph::NodeIndex,
    overlay_graph::OverlayGraph,
    search::{astar::AStar, ch_search::CHSearch, dijkstra::Dijkstra},
    util::math::straight_line,
};
use crossbeam_channel::{Receiver, Sender};
use egui::{CollapsingHeader, Context, Window};
use macroquad::prelude::{is_key_pressed, KeyCode};

use crate::{
    graph_view::{GraphViewOptions, SearchResult},
    COLOR_THEME,
};

pub(crate) struct UserInputWidget<'g> {
    source_text: String,
    target_text: String,

    source_node: Option<NodeIndex>,
    target_node: Option<NodeIndex>,

    // dijkstra: &'g mut Dijkstra<'g>,
    overlay_graph: &'g OverlayGraph,
    tx_options: Sender<GraphViewOptions>,

    rx_search: Receiver<(Option<NodeIndex>, Option<NodeIndex>)>,
    tx_search: Sender<(Option<NodeIndex>, Option<NodeIndex>)>,

    options: GraphViewOptions,
    dark_theme: bool,
}

impl<'g> UserInputWidget<'g> {
    pub(crate) fn new(
        // dijkstra: &'g mut Dijkstra<'g>,
        overlay_graph: &'g OverlayGraph,
        tx_options: Sender<GraphViewOptions>,
        rx_search: Receiver<(Option<NodeIndex>, Option<NodeIndex>)>,
        tx_search: Sender<(Option<NodeIndex>, Option<NodeIndex>)>,
    ) -> UserInputWidget<'g> {
        UserInputWidget {
            source_text: "".to_string(),
            target_text: "".to_string(),

            source_node: None,
            target_node: None,

            // dijkstra,
            overlay_graph,
            tx_options,
            options: Default::default(),
            rx_search,
            tx_search,

            dark_theme: true,
        }
    }
}

impl<'g> super::MyWidget for UserInputWidget<'g> {
    fn update(&mut self, ctx: &Context) {
        if let Ok((source, target)) = self.rx_search.try_recv() {
            match (source, target) {
                (Some(s), None) => {
                    self.source_text = s.index().to_string();
                    self.source_node = Some(s);
                }
                (None, Some(t)) => {
                    self.target_text = t.index().to_string();
                    self.target_node = Some(t);
                }
                _ => unreachable!(),
            }
        }

        if is_key_pressed(KeyCode::F) && self.source_node.is_some() && self.target_node.is_some() {
            let mut bidir_search = CHSearch::new(self.overlay_graph);
            if let Some(sp) =
                bidir_search.search(self.source_node.unwrap(), self.target_node.unwrap())
            {
                self.options.search_result = Some(SearchResult { sp });
            }
        }

        if self.dark_theme {
            COLOR_THEME.lock().unwrap().set_dark_theme();
        } else {
            COLOR_THEME.lock().unwrap().set_light_theme();
        }

        Window::new("User Input").show(ctx, |ui| {
            CollapsingHeader::new("Graph Options").show(ui, |ui| {
                ui.checkbox(&mut self.options.draw_shortcuts, "Draw shortcuts");
                ui.checkbox(&mut self.options.draw_nodes, "Draw nodes");
                ui.checkbox(&mut self.options.draw_shortest_path, "Draw Shortest Path");
                ui.checkbox(&mut self.options.draw_graph_upward, "Draw Graph Up");
                ui.checkbox(&mut self.options.draw_graph_downward, "Draw Graph Down");
                // ui.checkbox(
                //     &mut self.options.draw_top_important_nodes,
                //     "Draw Most Important Nodes",
                // );
                ui.checkbox(&mut self.dark_theme, "Dark Theme");
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

                    // Buttons to start search
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                self.source_node.is_some() && self.target_node.is_some(),
                                egui::Button::new("CH Search"),
                            )
                            .on_disabled_hover_text("Enter valid start and end node first")
                            .clicked()
                        {
                            log::debug!("Search button clicked.");

                            let mut bidir_search = CHSearch::new(self.overlay_graph);
                            if let Some(sp) = bidir_search
                                .search(self.source_node.unwrap(), self.target_node.unwrap())
                            {
                                self.options.search_result = Some(SearchResult { sp });
                            }
                        }
                        if ui
                            .add_enabled(
                                self.source_node.is_some() && self.target_node.is_some(),
                                egui::Button::new("Dijk. Search"),
                            )
                            .on_disabled_hover_text("Enter valid start and end node first")
                            .clicked()
                        {
                            log::debug!("Dijk button clicked.");

                            let mut dijk_search = Dijkstra::new(self.overlay_graph.road_graph());
                            if let Some(sp) = dijk_search
                                .search(self.source_node.unwrap(), self.target_node.unwrap())
                            {
                                self.options.search_result = Some(SearchResult { sp });
                            }
                        }
                        if ui
                            .add_enabled(
                                self.source_node.is_some() && self.target_node.is_some(),
                                egui::Button::new("AStar Search"),
                            )
                            .on_disabled_hover_text("Enter valid start and end node first")
                            .clicked()
                        {
                            log::debug!("AStar button clicked.");

                            let mut astar_search = AStar::new(self.overlay_graph.road_graph());
                            if let Some(sp) = astar_search.search(
                                self.source_node.unwrap(),
                                self.target_node.unwrap(),
                                straight_line,
                            ) {
                                self.options.search_result = Some(SearchResult { sp });
                            }
                        }
                    });
                });
        });

        self.tx_options.send(self.options.clone()).unwrap();
        self.tx_search
            .send((self.source_node, self.target_node))
            .unwrap();
    }
}
