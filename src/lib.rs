#![warn(unreachable_pub)]

pub mod authentication;
pub mod client;
pub mod config;
pub mod http;
pub mod signaling;

mod room;

pub use types;

pub use client::OpenTalkClient;
pub use config::{AuthConfig, ControllerConfig};
pub use room::{
    DisconnectReason, Event, LiveKitApi, OpenTalkEvent, OpenTalkLiveKitEvent,
    OpenTalkRecordingEvent, Participant, RecordingApi, Room,
};
