// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk moderation module.

pub mod command;
pub mod event;
pub mod state;

mod kick_scope;

pub use kick_scope::KickScope;
use opentalk_types_common::modules::{ModuleId, module_id};

/// The module id for the signaling module
pub const MODERATION_MODULE_ID: ModuleId = module_id!("moderation");
