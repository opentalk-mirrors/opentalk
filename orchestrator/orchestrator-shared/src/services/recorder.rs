// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use crate::Event;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RegisterRecorder {
    pub rooms: HashSet<RecorderResource>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct RecorderResource {
    pub room_id: RoomId,
    pub breakout_id: Option<u32>,
}

impl super::ResourceType for RecorderResource {
    fn kind() -> crate::ServiceKind {
        crate::ServiceKind::Recorder
    }

    fn from_service_resource(resource: super::ServiceResource) -> Option<Self> {
        match resource {
            super::ServiceResource::Recorder(r) => Some(r),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecorderEvent {
    RemoveRecording(RecorderResource),
}

impl From<RecorderEvent> for Event {
    fn from(event: RecorderEvent) -> Event {
        Event::Recorder(event)
    }
}

impl TryFrom<Event> for RecorderEvent {
    type Error = Event;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Recorder(recorder_event) => Ok(recorder_event),
            event => Err(event),
        }
    }
}
