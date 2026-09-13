// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk legal vote module.

use opentalk_types_common::modules::{ModuleId, module_id};

pub mod cancel;
pub mod command;
pub mod event;
pub mod invalid;
pub mod issue;
pub mod parameters;
pub mod state;
pub mod tally;
pub mod token;
pub mod too_long_error;
pub mod user_parameters;
pub mod vote;

pub use command::LegalVoteCommand;
pub use event::LegalVoteEvent;

/// The module id for the signaling module.
pub const LEGAL_VOTE_MODULE_ID: ModuleId = module_id!("legal_vote");
