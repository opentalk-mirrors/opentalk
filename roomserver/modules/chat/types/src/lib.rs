// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk chat module.

pub mod command;
pub mod event;
pub mod peer_state;
pub mod state;

mod message_id;
mod scope;
mod settings;

pub use message_id::MessageId;
use opentalk_types_common::modules::{ModuleId, module_id};
pub use scope::Scope;
pub use settings::{ChatSettings, RateLimitSettings};

/// The module id for the signaling module
pub const CHAT_MODULE_ID: ModuleId = module_id!("chat");
