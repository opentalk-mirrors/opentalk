// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub use command::ReactionCommand;
pub use event::ReactionEvent;
use opentalk_types_common::modules::{ModuleId, module_id};
pub use reaction::Reaction;
pub use state::ReactionState;

pub mod command;
pub mod event;
pub mod reaction;
pub mod state;

/// The module id for the reactions module
pub const REACTION_MODULE_ID: ModuleId = module_id!("reaction");
