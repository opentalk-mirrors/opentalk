// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use eframe::storage_dir;

#[derive(Debug)]
pub struct AboutView {
    app_id: String,
}

impl AboutView {
    pub fn new(app_id: String) -> Self {
        Self { app_id }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("App ID");
        ui.label(&self.app_id);

        ui.heading("Settings file");
        ui.label(storage_dir(&self.app_id).map_or_else(
            || "Empty".to_owned(),
            |path| path.to_string_lossy().to_string(),
        ));
    }
}
