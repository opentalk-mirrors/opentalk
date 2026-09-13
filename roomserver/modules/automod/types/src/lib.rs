// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk automod module.

pub mod command;
pub mod config;
pub mod event;
pub mod state;

pub use opentalk_types_common::modules::{ModuleId, module_id};

/// The module id for the signaling module
pub const AUTOMOD_MODULE_ID: ModuleId = module_id!("automod");
