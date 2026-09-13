// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_client::api::command::{SignalingCommand, SignalingModuleCommand};
use opentalk_roomserver_types_timer::{TimerCommand, command::Kind};
use rand::Rng as _;

use super::plugin::Received;
use crate::app::{shortcuts::TOGGLE_TIMER_WINDOW_SHORTCUT, signaling::plugin::SignalingPlugin};

#[derive(Debug)]
pub struct TimerPlugin {
    new_timer_kind: Kind,
    new_timer_ready_check: bool,
    mark_me_ready: bool,
    rng: rand::prelude::ThreadRng,
}

impl SignalingPlugin for TimerPlugin {
    fn name(&self) -> &'static str {
        "Timer"
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
            self.control_ui(&mut messages, ui);
        });
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_TIMER_WINDOW_SHORTCUT)
    }
}

impl TimerPlugin {
    pub fn new() -> Self {
        let rng = rand::rng();
        Self {
            new_timer_kind: Kind::Stopwatch,
            new_timer_ready_check: false,
            mark_me_ready: false,
            rng,
        }
    }

    fn control_ui(&mut self, messages: &mut Vec<String>, ui: &mut egui::Ui) {
        if let Some(cmd) = self.timer_controls_ui(ui) {
            let mut signaling_command = SignalingCommand::from(SignalingModuleCommand::Timer(cmd));
            signaling_command.transaction_id = Some(self.rng.next_u64());

            messages.push(
                serde_json::to_string(&signaling_command)
                    .expect("SignalingCommand must be serializable"),
            );
        }
    }

    fn timer_controls_ui(&mut self, ui: &mut egui::Ui) -> Option<TimerCommand> {
        if let Some(cmd) = self.start_command_ui(ui) {
            return Some(cmd);
        }
        if let Some(cmd) = self.ready_command_ui(ui) {
            return Some(cmd);
        }
        if ui.button("Stop").clicked() {
            return Some(TimerCommand::Stop { reason: None });
        }
        None
    }

    fn ready_command_ui(&mut self, ui: &mut egui::Ui) -> Option<TimerCommand> {
        ui.horizontal(|ui| {
            if ui.button("Ready").clicked() {
                return Some(TimerCommand::UpdateReadyStatus {
                    status: self.mark_me_ready,
                });
            }
            ui.checkbox(&mut self.mark_me_ready, "I'm ready");
            None
        })
        .inner
    }

    fn start_command_ui(&mut self, ui: &mut egui::Ui) -> Option<TimerCommand> {
        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                return Some(TimerCommand::Start {
                    kind: self.new_timer_kind,
                    style: None,
                    title: None,
                    enable_ready_check: self.new_timer_ready_check,
                });
            }
            let kind_label = match self.new_timer_kind {
                Kind::Stopwatch => "Stopwatch",
                Kind::Countdown { .. } => "Countdown",
            };
            ui.menu_button(kind_label, |ui| {
                if ui.button("Stopwatch").clicked() {
                    self.new_timer_kind = Kind::Stopwatch;
                }
                if ui.button("Countdown").clicked() {
                    self.new_timer_kind = Kind::Countdown { duration: 0 };
                }
            });
            ui.checkbox(&mut self.new_timer_ready_check, "Ready Check");
            if let Kind::Countdown { duration } = &mut self.new_timer_kind {
                ui.add(egui::DragValue::new(duration));
            }
            None
        })
        .inner
    }
}
