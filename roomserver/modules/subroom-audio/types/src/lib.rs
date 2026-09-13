// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk subroom audio module.

pub mod command;
pub mod event;
pub mod internal;
pub mod state;

mod whisper_id;

use opentalk_types_common::modules::{ModuleId, module_id};
pub use whisper_id::WhisperId;

/// The module id for the signaling module
pub const SUBROOM_AUDIO_MODULE_ID: ModuleId = module_id!("subroom_audio");
