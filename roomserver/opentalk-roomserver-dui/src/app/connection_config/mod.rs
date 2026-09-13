// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Button, Label, RichText, TextEdit, Widget};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_types_common::rooms::RoomId;

use super::{
    TransitionToView,
    shortcuts::SUBMIT_SHORTCUT,
    style::{InvalidInputStyle as _, delete_btn, delete_mode_btn},
};
use crate::{app::connection_config::parameter_widget::ParameterWidget, settings::DuiSettings};

mod parameter_widget;

#[derive(Debug)]
pub struct ConnectionConfigView {
    new_room_id: String,
    new_room_id_name: String,

    room_parameters_select: ParameterWidget<RoomParameters>,
    client_parameters_select: ParameterWidget<ClientParameters>,
}

impl ConnectionConfigView {
    pub fn new(settings: &DuiSettings) -> Self {
        let room_param = settings
            .room_parameters
            .get(settings.selected_room_parameters)
            .map(|(_, param)| param)
            .cloned()
            .unwrap_or_else(String::new);
        let client_param = settings
            .client_parameters
            .get(settings.selected_client_parameters)
            .map(|(_, param)| param)
            .cloned()
            .unwrap_or_else(String::new);

        Self {
            new_room_id: RoomId::generate().to_string(),
            new_room_id_name: String::new(),

            room_parameters_select: ParameterWidget::new("Room Parameters".to_string(), room_param),
            client_parameters_select: ParameterWidget::new(
                "Client Parameters".to_string(),
                client_param,
            ),
        }
    }

    pub fn menu_ui(settings: &mut DuiSettings, ui: &mut egui::Ui) {
        delete_mode_btn(ui, &mut settings.delete_mode);
    }

    pub fn center_ui(&mut self, settings: &mut DuiSettings, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(egui_extras::Size::relative(0.2))
            .size(egui_extras::Size::relative(0.4))
            .size(egui_extras::Size::relative(0.4))
            .vertical(|mut strip| {
                strip.strip(|builder| {
                    self.room_id_ui(settings, builder);
                });

                strip.strip(|builder| {
                    self.room_parameters_select.ui(
                        &mut settings.room_parameters,
                        &mut settings.selected_room_parameters,
                        settings.delete_mode,
                        builder,
                    );
                });

                strip.strip(|builder| {
                    self.client_parameters_select.ui(
                        &mut settings.client_parameters,
                        &mut settings.selected_client_parameters,
                        settings.delete_mode,
                        builder,
                    );
                });
            });
    }

    pub fn ui_bottom(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut DuiSettings,
    ) -> Option<TransitionToView> {
        let mut transition_request = None;
        let connect_btn =
            Button::new("Connect").shortcut_text(ui.ctx().format_shortcut(&SUBMIT_SHORTCUT));
        if let (Some(room_id), Ok(room_parameters), Ok(client_parameters)) = (
            settings
                .room_ids
                .get(settings.selected_room_id)
                .map(|(_, r)| *r),
            &self.room_parameters_select.parsed,
            &self.client_parameters_select.parsed,
        ) {
            if ui.add(connect_btn).clicked()
                || ui.ctx().input_mut(|i| i.consume_shortcut(&SUBMIT_SHORTCUT))
            {
                transition_request = Some(TransitionToView::Connecting {
                    room_id,
                    client_parameters: client_parameters.clone().into(),
                    room_parameters: room_parameters.clone().into(),
                });
            }
        } else {
            ui.add_enabled(false, connect_btn);
        }

        transition_request
    }

    fn room_id_ui(&mut self, settings: &mut DuiSettings, builder: StripBuilder<'_>) {
        let parsed_room_id = self.new_room_id.parse::<RoomId>();
        builder
            .size(egui_extras::Size::initial(20.))
            .size(Size::remainder())
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.heading("Room ID");
                });

                strip.strip(|builder| {
                    builder
                        .size(Size::relative(0.2))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.vertical(|ui| {
                                    Self::room_id_select_table_ui(settings, ui);
                                });
                            });

                            strip.cell(|ui| {
                                ui.add(
                                    TextEdit::singleline(&mut self.new_room_id_name)
                                        .hint_text("Room Name"),
                                );
                                let mut room_id_input = TextEdit::singleline(&mut self.new_room_id)
                                    .hint_text("Room ID");
                                if parsed_room_id.is_err() {
                                    room_id_input = room_id_input.invalid_input_style();
                                }
                                let res = ui.add(room_id_input);
                                if let Err(e) = &parsed_room_id {
                                    res.on_hover_ui(|ui| {
                                        ui.label(e.to_string());
                                    });
                                }
                                if ui.button("🎲").clicked() {
                                    self.new_room_id = RoomId::generate().to_string();
                                }
                                if ui
                                    .add_enabled(parsed_room_id.is_ok(), egui::Button::new("save"))
                                    .clicked()
                                    && let Ok(room_id) = parsed_room_id
                                {
                                    settings
                                        .room_ids
                                        .push((self.new_room_id_name.clone(), room_id));
                                    self.new_room_id_name.clear();
                                    self.new_room_id = RoomId::generate().to_string();
                                    settings.selected_room_id = settings.room_ids.len() - 1;
                                }
                            });
                        });
                });
            });
    }

    fn room_id_select_table_ui(settings: &mut DuiSettings, ui: &mut egui::Ui) {
        let available_height = ui.available_height();

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut delete_room_id = None;
        TableBuilder::new(ui)
            .striped(true)
            .sense(egui::Sense::click())
            .max_scroll_height(available_height)
            .auto_shrink(false)
            .column(Column::auto())
            .column(Column::exact(280.))
            .column(Column::remainder().clip(true))
            .body(|body| {
                body.rows(text_height, settings.room_ids.len(), |mut row| {
                    let row_index = row.index();
                    let Some((name, room_id)) = settings.room_ids.get(row_index) else {
                        return;
                    };

                    row.set_selected(settings.selected_room_id == row_index);

                    row.col(|ui| {
                        if delete_btn(ui, settings.delete_mode).clicked() {
                            delete_room_id = Some(row_index);
                        }
                    });

                    row.col(|ui| {
                        Label::new(RichText::new(room_id.to_string()).monospace()).ui(ui);
                    });

                    row.col(|ui| {
                        ui.label(name);
                    });
                    if row.response().clicked() {
                        settings.selected_room_id = row_index;
                    }
                });
            });

        if let Some(index) = delete_room_id {
            settings.room_ids.remove(index);

            // when we delete something, the selected index might "shift" and the selection changed.
            // Decrease by 1 to avoid that unwanted change.
            if settings.selected_room_id > index {
                settings.selected_room_id = settings.selected_room_id.saturating_sub(1);
            }
            log::trace!("request repaint: room_id deleted");
            ui.ctx().request_repaint();
        }
    }
}
