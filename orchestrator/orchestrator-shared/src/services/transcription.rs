// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use crate::Event;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RegisterTranscription {
    pub rooms: HashSet<TranscriptionResource>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TranscriptionResource {
    pub room_id: RoomId,
    pub breakout_id: Option<u32>,
}

impl super::ResourceType for TranscriptionResource {
    fn kind() -> crate::ServiceKind {
        crate::ServiceKind::Transcription
    }

    fn from_service_resource(resource: super::ServiceResource) -> Option<Self> {
        match resource {
            super::ServiceResource::Transcription(r) => Some(r),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionEvent {
    RemoveTranscription(TranscriptionResource),
}

impl From<TranscriptionEvent> for Event {
    fn from(event: TranscriptionEvent) -> Event {
        Event::Transcription(event)
    }
}

impl TryFrom<Event> for TranscriptionEvent {
    type Error = Event;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Transcription(transcription_event) => Ok(transcription_event),
            event => Err(event),
        }
    }
}
