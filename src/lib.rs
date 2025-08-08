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
#[cfg(feature = "transcription")]
pub use room::TranscriptionApi;
#[cfg(feature = "chat")]
pub use room::{ChatApi, Message, OpenTalkChatEvent};
pub use room::{
    DisconnectReason, Event, LiveKitApi, OpenTalkEvent, OpenTalkLiveKitEvent,
    OpenTalkRecordingEvent, Participant, RecordingApi, Room,
};
#[cfg(feature = "recording-service")]
pub use room::{OpenTalkRecordingServiceEvent, RecordingServiceApi};
pub use types;
#[cfg(feature = "transcription")]
pub use types::signaling::transcription::segment::Segment as TranscriptionSegment;
