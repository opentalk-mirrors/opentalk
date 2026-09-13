// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};

pub mod command;
pub mod event;
pub mod state;

/// The module id for the signaling module
pub const RAISE_HANDS_MODULE_ID: ModuleId = module_id!("raise_hands");
