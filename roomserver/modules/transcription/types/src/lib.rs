// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk transcription module.

use opentalk_types_common::{
    features::{FeatureId, feature_id},
    modules::{ModuleId, module_id},
};

pub mod command;
pub mod event;
pub mod segment;
pub mod service;
pub mod settings;
pub mod state;

/// The namespace string for the signaling module
pub const TRANSCRIPTION_MODULE_ID: ModuleId = module_id!("transcription");

/// The feature for allowing transcription of meetings
pub const TRANSCRIPTION_FEATURE_ID: FeatureId = feature_id!("transcription");
