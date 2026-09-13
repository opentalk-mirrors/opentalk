// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{Color32, RichText, Widget};
use egui_json_tree::{JsonTree, JsonTreeStyle, ToggleButtonsState};
use uuid::Uuid;

use crate::client::{RunnerEvent, RunnerEventType};

mod event_widget_layout;

pub use event_widget_layout::{EventWidgetLayout, Expand};

use super::signaling::filtered_vec::{Filter, Filterable};

#[derive(Debug)]
pub(crate) struct EventWidget {
    event: RunnerEvent,
    /// Whether the message was parsed successfully with the opentalk-types.
    known_type: bool,
    json: Option<serde_json::Value>,
    timestamp: String,

    /// Reset the json tree
    reset_expanded: bool,
    id: Uuid,
}

impl EventWidget {
    pub fn new(event: RunnerEvent) -> Self {
        let json = match &event.event_type {
            RunnerEventType::SendSuccess { message } | RunnerEventType::Received { message } => {
                let value = serde_json::from_str(message);
                value.ok()
            }
            _ => None,
        };
        let timestamp = event.timestamp.format("%T %3f").to_string();
        Self {
            event,
            known_type: false,
            json,
            timestamp,
            reset_expanded: false,
            id: Uuid::new_v4(),
        }
    }

    pub(crate) fn control_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::Label::new(RichText::new(&self.timestamp).color(Color32::GRAY))
                .wrap_mode(egui::TextWrapMode::Extend)
                .ui(ui);

            match &self.event.event_type {
                RunnerEventType::Received { .. } => {
                    if self.known_type {
                        ui.label(RichText::new("✅").color(Color32::GREEN))
                            .on_hover_text("This message was understood by DUI");
                    } else {
                        ui.label(RichText::new("🚫").color(Color32::RED))
                            .on_hover_text("Unknown, unprocessed message");
                    }
                    ui.label(
                        RichText::new("⬇")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_BLUE),
                    );
                    if self.json.is_some() && ui.button("Reset").clicked() {
                        self.reset_expanded = true;
                    }
                }
                RunnerEventType::SendSuccess { .. } => {
                    ui.label(
                        RichText::new("⬆")
                            .color(Color32::WHITE)
                            .background_color(Color32::DARK_GREEN),
                    );
                    if self.json.is_some() && ui.button("Reset").clicked() {
                        self.reset_expanded = true;
                    }
                }
                _ => {}
            }
        });
    }

    pub fn content_ui(
        &mut self,
        ui: &mut egui::Ui,
        layout: &event_widget_layout::EventWidgetLayout,
        show_plain: bool,
    ) {
        ui.vertical(|ui| {
            match &self.event.event_type {
                RunnerEventType::Disconnected => {
                    ui.label(RichText::new("Disconnected").color(Color32::RED));
                }
                RunnerEventType::Connected => {
                    ui.label(RichText::new("Connected").color(Color32::GREEN));
                }
                RunnerEventType::Received { message }
                | RunnerEventType::SendSuccess { message } => match &self.json {
                    Some(json) if !show_plain => {
                        let res = JsonTree::new(self.id, json)
                            .style(
                                JsonTreeStyle::new()
                                    .toggle_buttons_state(ToggleButtonsState::VisibleEnabled),
                            )
                            .default_expand(layout.expanded.into())
                            .show(ui);
                        if self.reset_expanded {
                            res.reset_expanded(ui);
                            self.reset_expanded = false;
                        }
                    }
                    _ => {
                        ui.code(message);
                    }
                },
                RunnerEventType::ReceiveError { error } => {
                    ui.label(error.to_string());
                }

                RunnerEventType::SendError { error } => {
                    ui.label(error.to_string());
                }
            }
            // fill the rest of the available space so that it doesn't grow with
            // more content, but already fills everything that is available.
            ui.allocate_space(ui.available_size());
        });
    }

    pub(crate) fn set_type_known(&mut self, known_type: bool) {
        self.known_type = known_type;
    }
}

impl From<RunnerEvent> for EventWidget {
    fn from(value: RunnerEvent) -> Self {
        Self::new(value)
    }
}

impl Filterable for EventWidget {
    fn apply(&self, filter: &mut Filter) -> bool {
        match &self.event.event_type {
            RunnerEventType::SendSuccess { message } | RunnerEventType::Received { message } => {
                filter.apply(message)
            }
            _ => true,
        }
    }
}
