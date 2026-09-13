// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText, TextEdit};

use crate::app::shortcuts::DELETE_MODE_SHORTCUT;

pub const SECTION_SPACE_HEIGHT: f32 = 20.;

const INVALID_TEXT_COLOR: Color32 = Color32::BLACK;
const INVALID_BACKGROUND_COLOR: Color32 = Color32::LIGHT_RED;

pub trait InvalidInputStyle {
    fn invalid_input_style(self) -> Self;
}

impl InvalidInputStyle for TextEdit<'_> {
    fn invalid_input_style(self) -> Self {
        self.text_color(INVALID_TEXT_COLOR)
            .background_color(INVALID_BACKGROUND_COLOR)
    }
}

impl InvalidInputStyle for RichText {
    fn invalid_input_style(self) -> Self {
        self.color(INVALID_TEXT_COLOR)
            .background_color(INVALID_BACKGROUND_COLOR)
    }
}

pub fn delete_btn(ui: &mut egui::Ui, visible: bool) -> egui::Response {
    let btn = egui::Button::new(RichText::new("❌").color(Color32::WHITE)).fill(Color32::DARK_RED);
    ui.add_visible(visible, btn)
}

pub fn delete_mode_btn(ui: &mut egui::Ui, delete_mode: &mut bool) {
    let btn_text = if *delete_mode {
        "Exit Delete Mode"
    } else {
        "Enter Delete Mode"
    };
    let btn =
        egui::Button::new(btn_text).shortcut_text(ui.ctx().format_shortcut(&DELETE_MODE_SHORTCUT));

    if ui.add(btn).clicked()
        || ui
            .ctx()
            .input_mut(|i| i.consume_shortcut(&DELETE_MODE_SHORTCUT))
    {
        *delete_mode = !*delete_mode;
    }
}
