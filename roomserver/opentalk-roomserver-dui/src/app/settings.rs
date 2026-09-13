// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::TextEdit;
use url::Url;

use super::{
    event_widget::Expand,
    style::{InvalidInputStyle, SECTION_SPACE_HEIGHT},
};
use crate::settings::DuiSettings;

#[derive(Debug)]
pub struct SettingsView {
    /// Temporary roomserver URL. This might be an invalid URL.
    roomserver_url: String,
}

impl SettingsView {
    pub fn new(settings: &DuiSettings) -> Self {
        Self {
            roomserver_url: settings.roomserver_url.to_string(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, settings: &mut DuiSettings) {
        settings.mark_custom();
        let valid_url = if let Ok(url) = self.roomserver_url.parse::<Url>() {
            settings.roomserver_url = url;
            true
        } else {
            false
        };
        ui.heading("Settings");

        ui.add_space(SECTION_SPACE_HEIGHT);
        self.server(ui, settings, valid_url);

        ui.add_space(SECTION_SPACE_HEIGHT);
        Self::theme(ui, settings);

        ui.add_space(SECTION_SPACE_HEIGHT);
        Self::event_widget_layout(ui, settings);

        ui.add_space(SECTION_SPACE_HEIGHT);
        Self::message_history(ui, &mut settings.history);
    }

    fn server(&mut self, ui: &mut egui::Ui, settings: &mut DuiSettings, valid_url: bool) {
        ui.heading("RoomServer");
        egui::Grid::new("roomserver-settings").show(ui, |ui| {
            let name_label = ui.label("Address: ");
            let mut edit =
                TextEdit::singleline(&mut self.roomserver_url).min_size([240., 0.].into());
            if !valid_url {
                edit = edit.invalid_input_style();
            }
            ui.add(edit).labelled_by(name_label.id);
            ui.end_row();

            let name_label = ui.label("Api key id: ");
            ui.text_edit_singleline(&mut settings.roomserver_api_key.id.0)
                .labelled_by(name_label.id);

            let name_label = ui.label("Api secret: ");
            ui.text_edit_singleline(&mut settings.roomserver_api_key.secret.0)
                .labelled_by(name_label.id);

            ui.end_row();
        });
    }

    fn theme(ui: &mut egui::Ui, settings: &mut DuiSettings) {
        let mut theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.heading("Theme");
        theme.ui(ui);
        theme.clone().store_in_memory(ui.ctx());
        ui.ctx()
            .options(|options| settings.theme = options.theme_preference.into());
    }

    fn event_widget_layout(ui: &mut egui::Ui, settings: &mut DuiSettings) {
        ui.heading("Event Widget");

        ui.label("JSON Tree Expansion");
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut settings.event_widget_layout.expanded,
                Expand::All,
                "Expanded",
            );
            ui.radio_value(
                &mut settings.event_widget_layout.expanded,
                Expand::None,
                "Collapsed",
            );

            let enabled = matches!(settings.event_widget_layout.expanded, Expand::ToLevel(_));
            if ui.add(egui::RadioButton::new(enabled, "Level")).clicked()
                && !settings.event_widget_layout.expanded.is_to_level()
            {
                settings.event_widget_layout.expanded = Expand::ToLevel(1);
            }
            let level = match &mut settings.event_widget_layout.expanded {
                Expand::ToLevel(level) => level,
                _ => &mut 0,
            };
            ui.add_enabled(
                enabled,
                egui::DragValue::new(level)
                    .speed(1)
                    .range(1..=20)
                    .fixed_decimals(0),
            )
            .on_hover_text("Drag to set the level of expansion");
        });
    }

    fn message_history(ui: &mut egui::Ui, history: &mut crate::settings::MessageHistory) {
        ui.heading("Message History");
        ui.horizontal(|ui| {
            let label = ui.label("Number of stored messages");
            ui.add(
                egui::DragValue::new(&mut history.limit)
                    .speed(1)
                    .range(1..=10000)
                    .fixed_decimals(0),
            )
            .on_hover_text("Drag to set the level of expansion")
            .labelled_by(label.id);
        });
    }
}
