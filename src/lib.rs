// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![warn(unreachable_pub)]

mod room;

pub mod authentication;
pub mod client;
pub mod config;
pub mod http;

pub mod upload;

pub use client::{CallInConnectToken, OpenTalkClient};
pub use config::{AuthConfig, BaseUrl};
pub use opentalk_roomserver_types;
pub use opentalk_roomserver_types_livekit;
pub use opentalk_roomserver_types_moderation;
pub use opentalk_roomserver_types_raise_hands;
pub use opentalk_roomserver_types_recording;
pub use opentalk_roomserver_types_transcription;
pub use room::{
    BreakoutApi, BreakoutConfig, BreakoutState, DisconnectReason, LiveKitApi, OpenTalkEvent,
    OpenTalkLiveKitEvent, OpenTalkRecordingEvent, OpenTalkRecordingServiceEvent, Participant,
    RecorderAction, RecordingApi, RecordingServiceApi, Room, TranscriptionApi,
    TranscriptionServiceApi,
};
pub use types;
