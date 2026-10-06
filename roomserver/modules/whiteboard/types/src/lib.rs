// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

pub use command::WhiteboardCommand;
pub use edit_restrictions::EditRestrictions;
pub use error::WhiteboardError;
pub use event::WhiteboardEvent;
use opentalk_types_common::modules::{ModuleId, module_id};

pub mod command;
pub mod edit_restrictions;
pub mod error;
pub mod event;

/// The module id for the signaling module.
pub const WHITEBOARD_MODULE_ID: ModuleId = module_id!("whiteboard");
