// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use crate::Event;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RegisterRoomserver {
    pub rooms: HashSet<RoomId>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomserverEvent {
    RemoveRoom(RoomId),
}

impl From<RoomserverEvent> for Event {
    fn from(event: RoomserverEvent) -> Event {
        Event::Roomserver(event)
    }
}

impl TryFrom<Event> for RoomserverEvent {
    type Error = Event;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Roomserver(roomserver_event) => Ok(roomserver_event),
            event => Err(event),
        }
    }
}
