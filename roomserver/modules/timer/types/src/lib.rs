// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};

/// The module id for the signaling module
pub const TIMER_MODULE_ID: ModuleId = module_id!("timer");

pub mod command;
pub mod event;
pub mod kind;
pub mod peer_state;
pub mod state;

pub use command::{timer_command::TimerCommand, timer_config::TimerConfig};
pub use event::{TimerEvent, error::TimerError, stop_kind::StopKind};
pub use kind::Kind;
