// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::Widget;

/// JSON EDITOR MUST BE LAST SINCE ITS USING ALL AVAILABLE SPACE!
pub(crate) fn json_editor(ui: &mut egui::Ui, text: &mut String) -> egui::Response {
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut layout_job = egui_extras::syntax_highlighting::highlight(
            ui.ctx(),
            ui.style(),
            &theme,
            buf.as_str(),
            "json",
        );
        layout_job.wrap.max_width = wrap_width;
        ui.fonts_mut(|f| f.layout_job(layout_job))
    };

    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            egui::TextEdit::multiline(text)
                .font(egui::TextStyle::Monospace)
                .min_size(ui.available_size())
                .desired_width(ui.available_width())
                .code_editor()
                .lock_focus(true)
                .layouter(&mut layouter)
                .ui(ui)
        })
        .inner
}
