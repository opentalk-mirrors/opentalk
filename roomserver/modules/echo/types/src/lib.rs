// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! The echo module is used for connectivity checks. When a client sends a
//! [`Ping`](command::EchoCommand::Ping) command, the module responds with a
//! [`Pong`](event::EchoEvent::Pong) event.

use opentalk_types_common::modules::{ModuleId, module_id};

pub const ECHO_MODULE_ID: ModuleId = module_id!("echo");

pub mod command;
pub mod event;

pub use command::EchoCommand;
pub use event::EchoEvent;
