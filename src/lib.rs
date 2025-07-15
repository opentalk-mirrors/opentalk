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
pub use room::OpenTalkChatEvent;
#[cfg(feature = "recording-service")]
pub use room::OpenTalkRecordingServiceEvent;
pub use room::{
    DisconnectReason, Event, LiveKitApi, OpenTalkEvent, OpenTalkLiveKitEvent,
    OpenTalkRecordingEvent, Participant, RecordingApi, Room,
};
pub use types;
