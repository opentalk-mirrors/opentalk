// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{RichText, TextEdit};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use serde::{Serialize, de::DeserializeOwned};

use super::super::{json_edit::json_editor, style::delete_btn};
use crate::app::style::InvalidInputStyle as _;

#[derive(Debug)]
pub(crate) struct ParameterWidget<T> {
    pub(crate) heading: String,
    pub(crate) new_name: String,
    pub(crate) edit: String,
    pub(crate) parsed: Result<T, serde_json::Error>,
}

impl<T: Clone + Serialize + DeserializeOwned> ParameterWidget<T> {
    pub(crate) fn new(heading: String, edit: String) -> Self {
        Self {
            heading,
            new_name: String::new(),
            parsed: serde_json::from_str(&edit),
            edit,
        }
    }

    pub(crate) fn ui(
        &mut self,
        collection: &mut Vec<(String, String)>,
        selected_index: &mut usize,
        delete_mode: bool,
        builder: StripBuilder<'_>,
    ) {
        builder
            .size(egui_extras::Size::initial(20.))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.heading(&self.heading);
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::relative(0.2))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.vertical(|ui| {
                                    self.table_ui(collection, selected_index, delete_mode, ui);
                                });
                            });

                            strip.cell(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(&mut self.new_name).hint_text("Name"),
                                    );
                                    self.save_ui(ui, collection, selected_index);
                                });

                                // The JSON editor has to be last since it's using all available
                                // space.
                                let res = json_editor(ui, &mut self.edit);
                                if res.changed() {
                                    self.parsed = serde_json::from_str(&self.edit);
                                }
                            });
                        });
                });
            });
    }

    pub(crate) fn table_ui(
        &mut self,
        collection: &mut Vec<(String, String)>,
        selected_index: &mut usize,
        delete_mode: bool,
        ui: &mut egui::Ui,
    ) {
        let available_height = ui.available_height();

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut delete_index = None;
        TableBuilder::new(ui)
            .striped(true)
            .sense(egui::Sense::click())
            .max_scroll_height(available_height)
            .auto_shrink(false)
            .column(Column::auto())
            .column(Column::remainder())
            .body(|body| {
                body.rows(text_height, collection.len(), |mut row| {
                    let row_index = row.index();
                    let Some((name, _)) = collection.get(row_index) else {
                        return;
                    };

                    row.set_selected(*selected_index == row_index);

                    row.col(|ui| {
                        if delete_btn(ui, delete_mode).clicked() {
                            delete_index = Some(row_index);
                        }
                    });

                    row.col(|ui| {
                        ui.label(name);
                    });

                    if row.response().clicked() {
                        *selected_index = row_index;
                        if let Some((_, item)) = collection.get(row_index) {
                            self.parsed = serde_json::from_str(item);
                            self.edit.clone_from(item);
                        }
                    }
                });
            });

        if let Some(index) = delete_index {
            collection.remove(index);
            if *selected_index > index {
                *selected_index = selected_index.saturating_sub(1);
            }
            log::trace!("request repaint: delete in settings `{}`", self.heading);
            ui.ctx().request_repaint();
        }
    }

    fn save_ui(
        &mut self,
        ui: &mut egui::Ui,
        collection: &mut Vec<(String, String)>,
        selected_index: &mut usize,
    ) {
        if ui
            .add_enabled(self.parsed.is_ok(), egui::Button::new("save"))
            .clicked()
        {
            collection.push((self.new_name.clone(), self.edit.clone()));
            self.new_name.clear();
            *selected_index = collection.len() - 1;
        }
        if let Some(e) = self.parsed.as_ref().err() {
            let err_text = RichText::new(e.to_string()).invalid_input_style();
            ui.label(err_text);
        }
    }
}
