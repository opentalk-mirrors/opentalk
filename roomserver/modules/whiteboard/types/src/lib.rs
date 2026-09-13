// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk whiteboard module.

use opentalk_types_common::modules::{ModuleId, module_id};

pub mod command;
pub mod event;
pub mod settings;
pub mod state;

pub use command::WhiteboardCommand;
pub use event::{WhiteboardError, WhiteboardEvent};
pub use settings::WhiteboardSettings;
pub use state::WhiteboardState;

/// The module id for the signaling module.
pub const WHITEBOARD_MODULE_ID: ModuleId = module_id!("whiteboard");
