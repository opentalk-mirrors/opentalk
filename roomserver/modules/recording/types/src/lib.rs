// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk recording module.

pub mod command;
pub mod event;
pub mod peer_state;
pub mod service;
pub mod settings;
pub mod state;

mod status;
mod streaming_target;

use opentalk_types_common::{
    features::{FeatureId, feature_id},
    modules::{ModuleId, module_id},
};
pub use settings::RecordingSettings;
pub use status::{RecordingStatus, StreamErrorReason, StreamStatus};
pub use streaming_target::StreamingTarget;

/// The namespace string for the signaling module
pub const RECORDING_MODULE_ID: ModuleId = module_id!("recording");

/// The feature for allowing recording of meetings
pub const RECORD_FEATURE_ID: FeatureId = feature_id!("record");

/// The feature for allowing streaming of meetings
pub const STREAM_FEATURE_ID: FeatureId = feature_id!("stream");
