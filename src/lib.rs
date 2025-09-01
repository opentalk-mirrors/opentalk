// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![warn(unreachable_pub)]

mod room;

pub mod authentication;
pub mod client;
pub mod config;
pub mod http;
pub mod signaling;

#[cfg(feature = "recording-service")]
pub mod upload;

pub use client::OpenTalkClient;
pub use config::{AuthConfig, ControllerConfig};
#[cfg(feature = "chat")]
pub use room::{ChatApi, Message, OpenTalkChatEvent};
pub use room::{
    DisconnectReason, Event, LiveKitApi, OpenTalkEvent, OpenTalkLiveKitEvent,
    OpenTalkRecordingEvent, Participant, RecordingApi, Room,
};
#[cfg(feature = "recording-service")]
pub use room::{OpenTalkRecordingServiceEvent, RecordingServiceApi};
#[cfg(feature = "transcription-service")]
pub use room::{OpenTalkTranscriptionServiceEvent, TranscriptionServiceApi};
pub use types;
#[cfg(feature = "transcription-service")]
pub use types::signaling::transcription::segment::Segment as TranscriptionSegment;
