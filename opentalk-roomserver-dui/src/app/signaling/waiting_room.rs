// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use egui::{Button, Label, RichText, Widget as _};
use egui_extras::{Column, TableBuilder};
use opentalk_roomserver_client::api::{
    command::{SignalingCommand, SignalingModuleCommand},
    event::{SignalingEvent, SignalingModuleEvent},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreEvent, LeftWaitingRoom, state::CoreState},
    disconnect_reason::DisconnectReason,
    join::join_success::JoinSuccess,
    room_parameters::WaitingRoom,
    shared_json::SharedJson,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand, event::ModerationEvent, state::ModerationState,
};
use opentalk_types_common::{modules::ModuleId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use rand::Rng as _;

use super::plugin::Received;
use crate::app::{
    shortcuts::TOGGLE_WAITING_ROOM_WINDOW_SHORTCUT, signaling::plugin::SignalingPlugin,
};

const ACCEPT_BUTTON_TEXT: &str = "accept";
const ACCEPTED_TEXT: &str = "✅";
const DENIED_TEXT: &str = "⛔";

#[derive(Debug)]
pub struct WaitingRoomPlugin {
    waiting_room: WaitingRoom,
    waiting: BTreeMap<ParticipantId, WaitingParticipant>,
    in_room: BTreeMap<ParticipantId, InRoomParticipant>,
    rng: rand::prelude::ThreadRng,
}

impl SignalingPlugin for WaitingRoomPlugin {
    fn name(&self) -> &'static str {
        "Waiting Room"
    }

    fn handle_events(
        &mut self,
        _settings: &mut crate::settings::DuiSettings,
        received: &[Received],
    ) -> Vec<String> {
        for event in received {
            match event {
                Received::SignalingEvent(signaling_event) => self.handle_signaling(signaling_event),
                Received::Invalid => {}
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
        });
        messages
    }

    fn shortcut(&self) -> Option<&egui::KeyboardShortcut> {
        Some(&TOGGLE_WAITING_ROOM_WINDOW_SHORTCUT)
    }
}

impl WaitingRoomPlugin {
    pub fn new() -> Self {
        let rng = rand::rng();
        Self {
            waiting: BTreeMap::new(),
            in_room: BTreeMap::new(),
            waiting_room: WaitingRoom::Disabled,
            rng,
        }
    }

    fn control_ui(&mut self, messages: &mut Vec<String>, ui: &mut egui::Ui) {
        if let Some(cmd) = self.waiting_controls_ui(ui) {
            let mut signaling_command = SignalingCommand::from(cmd);
            signaling_command.transaction_id = Some(self.rng.next_u64());

            messages.push(
                serde_json::to_string(&signaling_command)
                    .expect("SignalingCommand must be serializable"),
            );
        }
    }

    fn waiting_controls_ui(&mut self, ui: &mut egui::Ui) -> Option<SignalingModuleCommand> {
        let res = ui
            .horizontal(|ui| {
                let before = self.waiting_room;
                egui::ComboBox::from_label("Waiting Room")
                    .selected_text(format!("{:?}", self.waiting_room))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.waiting_room,
                            WaitingRoom::Disabled,
                            "Disabled",
                        );
                        ui.selectable_value(
                            &mut self.waiting_room,
                            WaitingRoom::ForGuests,
                            "For Guests",
                        );
                        ui.selectable_value(
                            &mut self.waiting_room,
                            WaitingRoom::ForEveryone,
                            "For Everyone",
                        );
                    });

                if before != self.waiting_room {
                    return Some(SignalingModuleCommand::Moderation(
                        ModerationCommand::ChangeWaitingRoomState {
                            new_state: self.waiting_room,
                        },
                    ));
                }

                None
            })
            .inner;
        if let Some(cmd) = res {
            return Some(cmd);
        }
        if let Some(cmd) = self.send_to_waiting_room_ui(ui) {
            return Some(SignalingModuleCommand::Moderation(cmd));
        }
        if let Some(cmd) = self.accept_ui(ui) {
            return Some(SignalingModuleCommand::Moderation(cmd));
        }
        None
    }

    fn send_to_waiting_room_ui(&mut self, ui: &mut egui::Ui) -> Option<ModerationCommand> {
        if self.in_room.is_empty() {
            None
        } else {
            ui.menu_button("Sent To waiting Room", |ui| {
                for (participant, state) in &self.in_room {
                    if ui.button(state.display_name.to_string()).clicked() {
                        return Some(ModerationCommand::SendToWaitingRoom {
                            target: *participant,
                        });
                    }
                }
                None
            })
            .inner
            .flatten()
        }
    }

    fn accept_ui(&mut self, ui: &mut egui::Ui) -> Option<ModerationCommand> {
        ui.heading("Waiting");

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        let mut accept_participant = None;
        let participants: Vec<_> = self.waiting.keys().copied().collect();

        TableBuilder::new(ui)
            .striped(true)
            .auto_shrink(false)
            .column(Column::exact(text_height))
            .column(Column::auto().clip(true))
            .column(Column::auto().clip(true).resizable(true))
            .column(Column::remainder().clip(true).resizable(true))
            .body(|body| {
                body.rows(text_height, self.waiting.len().max(1), |mut row| {
                    let row_index = row.index();
                    let waiting_participant = participants
                        .get(row_index)
                        .and_then(|p| self.waiting.get(p));

                    // Show a disabled row in case the waiting room is empty. This also ensures the
                    // table columns are sized correctly
                    let Some(waiting_participant) = waiting_participant else {
                        placeholder_row_ui(&mut row);
                        return;
                    };

                    accept_participant = waiting_participant_row_ui(row, waiting_participant);
                });
            });

        accept_participant.map(|accept_me| ModerationCommand::Accept { target: accept_me })
    }

    fn handle_signaling(&mut self, signaling_event: &SignalingEvent) {
        match &signaling_event.payload {
            SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join_success)) => {
                self.joined_room(join_success);
            }

            SignalingModuleEvent::Core(CoreEvent::ParticipantConnected {
                participant_id,
                connection_id,
                peer_data,
                ..
            }) => {
                self.participant_connected(participant_id, connection_id, peer_data);
            }

            SignalingModuleEvent::Core(CoreEvent::ParticipantDisconnected {
                participant_id,
                connection_id,
                reason,
            }) => {
                self.participant_disconnected(*participant_id, *connection_id, *reason);
            }

            SignalingModuleEvent::Core(CoreEvent::JoinedWaitingRoom {
                participant_id,
                display_name,
                ..
            }) => {
                self.participant_joined_waiting_room(participant_id, display_name);
            }
            SignalingModuleEvent::Core(CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                id, ..
            })) => {
                self.waiting.remove(id);
            }
            SignalingModuleEvent::Moderation(ModerationEvent::ParticipantAccepted {
                participant_id,
            }) => {
                if let Some(accepted) = self.waiting.get_mut(participant_id) {
                    accepted.accepted = true;
                } else {
                    log::warn!("Received `ParticipantAccepted` for unknown participant");
                }
            }
            SignalingModuleEvent::Moderation(ModerationEvent::WaitingRoomUpdated { new_state }) => {
                self.waiting_room = *new_state;
            }
            _ => {}
        }
    }

    fn participant_joined_waiting_room(
        &mut self,
        participant_id: &ParticipantId,
        display_name: &DisplayName,
    ) {
        self.waiting
            .entry(*participant_id)
            .or_insert_with(|| WaitingParticipant {
                participant_id: *participant_id,
                display_name: display_name.to_string(),
                accepted: false,
            });
    }

    fn participant_disconnected(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    ) {
        let Entry::Occupied(mut occ) = self.in_room.entry(participant_id) else {
            return;
        };
        let participant = occ.get_mut();
        participant.connections.remove(&connection_id);
        let display_name = participant.display_name.clone();
        if participant.connections.is_empty() {
            occ.remove();
        }
        if reason == DisconnectReason::SentToWaitingRoom {
            self.waiting.insert(
                participant_id,
                WaitingParticipant {
                    participant_id,
                    display_name,
                    accepted: false,
                },
            );
        }
    }

    fn participant_connected(
        &mut self,
        participant_id: &ParticipantId,
        connection_id: &ConnectionId,
        peer_data: &BTreeMap<ModuleId, SharedJson>,
    ) {
        let core_data: CoreState =
            serde_json::from_value(peer_data.get(&CORE_MODULE_ID).unwrap().clone_inner()).unwrap();
        self.in_room
            .entry(*participant_id)
            .and_modify(|in_room| {
                in_room.connections.insert(*connection_id);
            })
            .or_insert_with(|| InRoomParticipant {
                connections: BTreeSet::from([*connection_id]),
                display_name: core_data.display_name.to_string(),
            });
    }

    fn joined_room(&mut self, join_success: &JoinSuccess) {
        let Ok(moderation_data) = join_success.get_module::<ModerationState>() else {
            log::warn!("Invalid moderator state, waiting room might not work properly");
            return;
        };
        let Some(moderation_data) = moderation_data else {
            log::warn!("No moderation data in join success, missing moderation module?");
            return;
        };
        let Some(moderation_data) = moderation_data.moderator_data else {
            log::debug!("No moderation state, participant is not a moderator");
            return;
        };
        self.waiting = moderation_data
            .waiting_room_participants
            .iter()
            .map(|participant| {
                (
                    participant.participant_id,
                    WaitingParticipant {
                        participant_id: participant.participant_id,
                        display_name: participant.display_name.to_string(),
                        accepted: participant.accepted,
                    },
                )
            })
            .collect();
    }
}

fn waiting_participant_row_ui(
    mut row: egui_extras::TableRow<'_, '_>,
    waiting_participant: &WaitingParticipant,
) -> Option<ParticipantId> {
    let mut accept_participant = None;
    row.col(|ui| {
        if waiting_participant.accepted {
            ui.label(ACCEPTED_TEXT);
        } else {
            ui.label(DENIED_TEXT);
        }
    });

    row.col(|ui| {
        if ui.button(ACCEPT_BUTTON_TEXT).clicked() {
            accept_participant = Some(waiting_participant.participant_id);
        }
    });

    row.col(|ui| {
        ui.label(&waiting_participant.display_name);
    });

    row.col(|ui| {
        Label::new(RichText::new(waiting_participant.participant_id.to_string()).monospace())
            .ui(ui);
    });

    accept_participant
}

fn placeholder_row_ui(row: &mut egui_extras::TableRow<'_, '_>) {
    row.col(|ui| {
        ui.label(DENIED_TEXT);
    });
    row.col(|ui| {
        ui.add_enabled(false, Button::new(ACCEPT_BUTTON_TEXT));
    });
    row.col(|ui| {
        ui.add_enabled(false, Label::new("empty"));
    });
    row.col(|ui| {
        ui.label(RichText::new("<participant-id>").weak());
    });
}

#[derive(Debug)]
struct WaitingParticipant {
    participant_id: ParticipantId,
    display_name: String,
    accepted: bool,
}

#[derive(Debug)]
struct InRoomParticipant {
    display_name: String,
    connections: BTreeSet<ConnectionId>,
}
