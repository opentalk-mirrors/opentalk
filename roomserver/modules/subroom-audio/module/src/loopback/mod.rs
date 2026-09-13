// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
pub use crate::loopback::{
    create_room::create_room, destroy_room::destroy_room, remove_participant::remove_participant,
};

mod create_room;
mod destroy_room;
mod remove_participant;

pub enum SubroomAudioLoopback {
    RoomCreated,
    RoomDestroyed,
    ParticipantRemoved,
    ProxySocketClosed,
}
