// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::Widget;
use opentalk_roomserver_client::api::command::{SignalingCommand, SignalingModuleCommand};
use opentalk_roomserver_types_chat::{Scope, command::ChatCommand};
use rand::Rng as _;

use super::plugin::Received;
use crate::app::{
    shortcuts::TOGGLE_SPAM_AMOUNT_WINDOW_SHORTCUT, signaling::plugin::SignalingPlugin,
};

/// Plugin that allows to spam an amount of chat messages.
#[derive(Debug)]
pub struct SpamAmountPlugin {
    message_size: usize,
    message_count: usize,
    running: bool,
    message: String,
    rng: rand::prelude::ThreadRng,
}

impl SignalingPlugin for SpamAmountPlugin {
    fn name(&self) -> &'static str {
        "Spam Amount"
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
        ui.vertical(|ui| {
            self.message_count_ui(ui);
            self.message_size_ui(ui);
            if let Some(msg) = self.command_ui(ui) {
                messages.push(msg);
            }
        });
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_SPAM_AMOUNT_WINDOW_SHORTCUT)
    }
}

impl SpamAmountPlugin {
    pub fn new() -> Self {
        let message_size = 1024;
        let rng = rand::rng();
        let mut this = Self {
            message_size,
            message_count: 10,
            running: false,
            message: String::new(),
            rng,
        };
        this.next_message();
        this
    }

    fn message_count_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Count");
        egui::DragValue::new(&mut self.message_count).ui(ui);
    }

    fn message_size_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Size (in bytes + chat message overhead)");
        if egui::DragValue::new(&mut self.message_size)
            .ui(ui)
            .changed()
        {
            self.next_message();
        }
    }

    fn command_ui(&mut self, ui: &mut egui::Ui) -> Option<String> {
        ui.horizontal(|ui| {
            if !self.running && ui.button("Start").clicked() {
                self.running = true;
            } else if ui.button("Stop").clicked() || self.message_count == 0 {
                self.running = false;
            } else if self.running {
                self.message_count -= 1;
                log::trace!("request repaint: message count decreased");
                ui.ctx().request_repaint();
                return Some(self.message.clone());
            }
            None
        })
        .inner
    }

    fn next_message(&mut self) {
        let content: String = std::iter::repeat_n('~', self.message_size).collect();
        let mut signaling_command =
            SignalingCommand::from(SignalingModuleCommand::Chat(ChatCommand::SendMessage {
                content,
                scope: Scope::Global,
            }));
        signaling_command.transaction_id = Some(self.rng.next_u64());
        self.message =
            serde_json::to_string(&signaling_command).expect("ChatCommand must be serializable");
    }
}
