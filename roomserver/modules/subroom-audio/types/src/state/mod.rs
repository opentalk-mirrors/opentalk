// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the state of the `subroom-audio` module

mod whisper_group;
mod whisper_participant_state;

pub use whisper_group::WhisperGroup;
pub use whisper_participant_state::WhisperState;
