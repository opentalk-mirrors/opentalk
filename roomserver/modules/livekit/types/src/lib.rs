// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk livekit module.

mod command;
mod credentials;
mod error;
mod event;
mod internal;
mod microphone_restriction_state;
mod settings;
mod state;

use opentalk_types_common::modules::{ModuleId, module_id};

pub use crate::{
    command::LiveKitCommand,
    credentials::Credentials,
    error::LiveKitError,
    event::LiveKitEvent,
    internal::{
        LiveKitInternal, MicrophoneRestrictionError, MicrophoneRestrictionErrorKind,
        ParticipantsMuted,
    },
    microphone_restriction_state::MicrophoneRestrictionState,
    settings::LiveKitSettings,
    state::LiveKitState,
};

/// The module id for the signaling module
pub const LIVEKIT_MODULE_ID: ModuleId = module_id!("livekit");
