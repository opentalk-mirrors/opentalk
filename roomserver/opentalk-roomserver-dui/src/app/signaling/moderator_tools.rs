// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_client::api::command::{
    MeetingReportCommand, SignalingCommand, SignalingModuleCommand,
};

use super::plugin::Received;
use crate::app::signaling::plugin::SignalingPlugin;

#[derive(Debug)]
pub struct ModeratorToolsPlugin;

impl SignalingPlugin for ModeratorToolsPlugin {
    fn name(&self) -> &'static str {
        "Moderator Tools"
    }

    fn handle_events(
        &mut self,
        _settings: &mut crate::settings::DuiSettings,
        _received: &[Received],
    ) -> Vec<String> {
        Vec::new()
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _settings: &mut crate::settings::DuiSettings,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        moderator_tools_ui(ui, &mut messages);
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        None
    }
}

impl ModeratorToolsPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModeratorToolsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn moderator_tools_ui(ui: &mut egui::Ui, messages: &mut Vec<String>) {
    if ui.button("Generate Report").clicked() {
        messages.push(
            serde_json::to_string(&SignalingCommand::from(
                SignalingModuleCommand::MeetingReport(
                    MeetingReportCommand::GenerateAttendanceReport {
                        include_email_addresses: true,
                    },
                ),
            ))
            .expect("SignalingCommand must be serializable"),
        );
    }
}
