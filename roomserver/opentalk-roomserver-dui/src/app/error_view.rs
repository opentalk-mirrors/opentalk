// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText};

#[derive(Debug)]
pub struct ErrorView {
    message: RichText,
}

impl ErrorView {
    pub fn new(message: RichText) -> Self {
        Self { message }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("This is a fatal error! Please evacuate the memory immediately")
                .color(Color32::LIGHT_RED)
                .background_color(Color32::DARK_RED),
        );
        ui.spacing();
        ui.label(self.message.clone());
    }
}
