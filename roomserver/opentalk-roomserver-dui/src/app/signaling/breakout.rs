// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::{InnerResponse, RichText, Widget as _};
use opentalk_roomserver_client::api::{
    command::{SignalingCommand, SignalingModuleCommand},
    event::SignalingModuleEvent,
};
use opentalk_roomserver_types::{
    breakout::{
        BreakoutRoom,
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        command::BreakoutCommand,
        event::BreakoutEvent,
        module_data::BreakoutModuleData,
    },
    core::CoreEvent,
    join::join_success::JoinSuccess,
    room_kind::RoomKind,
};
use rand::Rng;

use super::plugin::Received;
use crate::app::{shortcuts::TOGGLE_BREAKOUT_WINDOW_SHORTCUT, signaling::plugin::SignalingPlugin};

#[derive(Debug)]
pub struct BreakoutPlugin {
    /// List of error events
    errors: Vec<String>,

    current_room: RoomKind,

    state: BreakoutState,

    num_breakout_rooms: usize,

    rng: rand::prelude::ThreadRng,
}

impl SignalingPlugin for BreakoutPlugin {
    fn name(&self) -> &'static str {
        "Breakout"
    }

    fn handle_events(
        &mut self,
        _settings: &mut crate::settings::DuiSettings,
        received: &[Received],
    ) -> Vec<String> {
        for msg in received {
            let Received::SignalingEvent(event) = msg else {
                continue;
            };
            match &event.payload {
                SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join_success)) => {
                    self.handle_join(join_success);
                }
                SignalingModuleEvent::Breakout(event) => {
                    self.handle_breakout_event(event);
                }
                _ => {}
            }
        }

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

            ui.separator();
            self.event_ui(ui);
        });
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_BREAKOUT_WINDOW_SHORTCUT)
    }
}

impl BreakoutPlugin {
    pub fn new() -> Self {
        let rng = rand::rng();
        Self {
            errors: Vec::new(),
            current_room: RoomKind::Main,
            state: BreakoutState::NoBreakout,
            num_breakout_rooms: 3,
            rng,
        }
    }

    fn event_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_width(ui.available_width())
            .show(ui, |ui| {
                egui::Grid::new("Message Grid")
                    .striped(true)
                    .num_columns(1)
                    .show(ui, |ui| {
                        for msg in &self.errors {
                            egui::Label::new(msg).wrap().ui(ui);
                            ui.end_row();
                        }
                    });
            });
    }

    fn control_ui(&mut self, messages: &mut Vec<String>, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(cmd) = self.room_status_ui(ui) {
                let mut cmd = SignalingCommand::from(SignalingModuleCommand::Breakout(cmd));
                cmd.transaction_id = Some(self.rng.next_u64());
                messages.push(
                    serde_json::to_string(&cmd).expect("SignalingCommand must be serializable"),
                );
            }
        });
    }

    fn room_status_ui(&mut self, ui: &mut egui::Ui) -> Option<BreakoutCommand> {
        self.room_name_ui(ui);
        match &self.state {
            BreakoutState::NoBreakout => {
                if let Some(msg) = self.start_breakout_ui(ui) {
                    return Some(msg);
                }
            }
            BreakoutState::BreakoutStarted { .. } => {
                if let Some(msg) = self.switch_room_ui(ui) {
                    return Some(msg);
                }
                if ui.button("Stop").clicked() {
                    return Some(BreakoutCommand::Stop { delay: None });
                }
            }
        }

        None
    }

    fn start_breakout_ui(&mut self, ui: &mut egui::Ui) -> Option<BreakoutCommand> {
        ui.label("start rooms...");
        ui.add(egui::DragValue::new(&mut self.num_breakout_rooms));
        if ui.button("Start").clicked() {
            return Some(BreakoutCommand::Start(BreakoutConfig {
                rooms: (0..self.num_breakout_rooms)
                    .map(|i| BreakoutRoomConfig {
                        name: format!("breakout room {i}"),
                        assignments: vec![],
                    })
                    .collect(),
                duration: None,
            }));
        }
        None
    }

    fn room_name_ui(&mut self, ui: &mut egui::Ui) {
        let room_name = match self.current_room {
            RoomKind::Breakout(id) => id.to_string(),
            RoomKind::Main => "Main".to_string(),
        };

        ui.label(format!("Current room: {room_name:?}"));
    }

    fn switch_room_ui(&mut self, ui: &mut egui::Ui) -> Option<BreakoutCommand> {
        if let BreakoutState::BreakoutStarted { rooms } = &self.state {
            ui.label("switch room");
            Self::room_select_ui(ui, rooms)
        } else {
            None
        }
    }

    fn room_select_ui(ui: &mut egui::Ui, rooms: &[BreakoutRoom]) -> Option<BreakoutCommand> {
        let res = ui.menu_button("switch room", |ui| {
            if ui.button(RichText::new("main").strong()).clicked() {
                return Some(BreakoutCommand::SwitchRoom(RoomKind::Main));
            }
            for room in rooms {
                if ui
                    .button(&room.name)
                    .on_hover_text(room.id.to_string())
                    .clicked()
                {
                    return Some(BreakoutCommand::SwitchRoom(RoomKind::Breakout(room.id)));
                }
            }
            None
        });

        if let InnerResponse {
            inner: Some(Some(msg)),
            ..
        } = res
        {
            return Some(msg);
        }
        None
    }

    fn handle_breakout_event(&mut self, event: &BreakoutEvent) {
        match event {
            BreakoutEvent::Started { rooms, .. } => {
                self.state = BreakoutState::BreakoutStarted {
                    rooms: rooms.clone(),
                };
            }
            BreakoutEvent::SwitchedRoom { new_room, .. } => {
                self.current_room = *new_room;
            }
            BreakoutEvent::Closed => {
                self.state = BreakoutState::NoBreakout;
            }

            _ => {}
        }
    }

    fn handle_join(&mut self, join_success: &JoinSuccess) {
        match join_success.module_data.get::<BreakoutModuleData>() {
            Ok(Some(BreakoutModuleData { room, rooms, .. })) => {
                self.current_room = room;
                self.state = BreakoutState::BreakoutStarted { rooms };
            }
            Ok(None) => {}
            Err(e) => {
                self.errors
                    .push(format!("ERROR: received invalid module data: {e}"));
            }
        }
    }
}

#[derive(Debug)]
enum BreakoutState {
    NoBreakout,
    BreakoutStarted { rooms: Vec<BreakoutRoom> },
}
