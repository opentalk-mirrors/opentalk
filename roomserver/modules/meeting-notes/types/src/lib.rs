// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk meeting-notes module.

use opentalk_types_common::modules::{ModuleId, module_id};

pub mod command;
pub mod event;
pub mod peer_state;
pub mod settings;

pub use command::MeetingNotesCommand;
pub use event::{MeetingNotesError, MeetingNotesEvent};
pub use peer_state::MeetingNotesPeerState;
pub use settings::MeetingNotesSettings;

/// The module id for the signaling module
pub const MEETING_NOTES_MODULE_ID: ModuleId = module_id!("meeting_notes");
