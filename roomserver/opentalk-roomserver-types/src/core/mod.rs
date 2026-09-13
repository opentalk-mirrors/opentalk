// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

mod command;
mod event;
pub mod state;

pub use command::CoreCommand;
pub use event::{CoreError, CoreEvent, JoinBlockedReason, LeftWaitingRoom, RoomCloseReason};
use opentalk_types_common::modules::{ModuleId, module_id};

pub const CORE_MODULE_ID: ModuleId = module_id!("core");
