// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use crate::{RecorderResource, TranscriptionResource};

const PREFIX_ROOMSERVER: &str = "roomserver:";
const PREFIX_RECORDER: &str = "recorder:";
const PREFIX_TRANSCRIPTION: &str = "transcription:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceResource {
    Roomserver(RoomId),
    Recorder(RecorderResource),
    Transcription(TranscriptionResource),
}

impl std::fmt::Display for ServiceResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceResource::Roomserver(room_id) => {
                write!(f, "{PREFIX_ROOMSERVER}{room_id}")
            }
            ServiceResource::Recorder(r) => match r.breakout_id {
                Some(breakout_id) => write!(f, "{PREFIX_RECORDER}{}:{breakout_id}", r.room_id),
                None => write!(f, "{PREFIX_RECORDER}{}", r.room_id),
            },
            ServiceResource::Transcription(r) => match r.breakout_id {
                Some(breakout_id) => {
                    write!(f, "{PREFIX_TRANSCRIPTION}{}:{breakout_id}", r.room_id)
                }
                None => write!(f, "{PREFIX_TRANSCRIPTION}{}", r.room_id),
            },
        }
    }
}

/// Error returned when failing to parse a [`ServiceResource`] from its string representation.
#[derive(Debug, thiserror::Error)]
pub enum ServiceResourceParseError {
    #[error("unknown service resource kind in '{0}'")]
    UnknownKind(String),
    #[error("invalid room ID in '{input}': {source}")]
    InvalidRoomId {
        input: String,
        #[source]
        source: <RoomId as std::str::FromStr>::Err,
    },
    #[error("invalid breakout ID in '{input}': {source}")]
    InvalidBreakoutId {
        input: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

impl TryFrom<String> for ServiceResource {
    type Error = ServiceResourceParseError;

    /// Parses a [`ServiceResource`] from its string representation as produced by
    /// [`Display`](std::fmt::Display).
    ///
    /// Expected formats:
    /// - `"roomserver:{uuid}"`
    /// - `"recorder:{uuid}"` or `"recorder:{uuid}:{breakout_id}"`
    /// - `"transcription:{uuid}"` or `"transcription:{uuid}:{breakout_id}"`
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl TryFrom<&str> for ServiceResource {
    type Error = ServiceResourceParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(rest) = s.strip_prefix(PREFIX_ROOMSERVER) {
            let room_id =
                rest.parse()
                    .map_err(|source| ServiceResourceParseError::InvalidRoomId {
                        input: s.to_string(),
                        source,
                    })?;
            return Ok(Self::Roomserver(room_id));
        }

        if let Some(rest) = s.strip_prefix(PREFIX_RECORDER) {
            let (room_id, breakout_id) = parse_room_and_breakout(s, rest)?;
            return Ok(Self::Recorder(RecorderResource {
                room_id,
                breakout_id,
            }));
        }

        if let Some(rest) = s.strip_prefix(PREFIX_TRANSCRIPTION) {
            let (room_id, breakout_id) = parse_room_and_breakout(s, rest)?;
            return Ok(Self::Transcription(TranscriptionResource {
                room_id,
                breakout_id,
            }));
        }

        Err(ServiceResourceParseError::UnknownKind(s.to_string()))
    }
}

/// Parses `"{room_id}"` or `"{room_id}:{breakout_id}"` from a resource string suffix.
///
/// `full` is the original full string (used only for error messages).
fn parse_room_and_breakout(
    full: &str,
    rest: &str,
) -> Result<(RoomId, Option<u32>), ServiceResourceParseError> {
    let mut parts = rest.splitn(2, ':');
    let room_id = parts.next().unwrap_or_default().parse().map_err(|source| {
        ServiceResourceParseError::InvalidRoomId {
            input: full.to_string(),
            source,
        }
    })?;
    let breakout_id = parts
        .next()
        .map(|b| b.parse())
        .transpose()
        .map_err(|source| ServiceResourceParseError::InvalidBreakoutId {
            input: full.to_string(),
            source,
        })?;
    Ok((room_id, breakout_id))
}

impl From<RoomId> for ServiceResource {
    fn from(room_id: RoomId) -> Self {
        Self::Roomserver(room_id)
    }
}

impl From<RecorderResource> for ServiceResource {
    fn from(recorder_resource: RecorderResource) -> Self {
        Self::Recorder(recorder_resource)
    }
}

impl From<TranscriptionResource> for ServiceResource {
    fn from(transcription_resource: TranscriptionResource) -> Self {
        Self::Transcription(transcription_resource)
    }
}
