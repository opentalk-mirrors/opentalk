// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `subroom_audio` namespace

mod error;
mod subroom_audio_events;
mod whisper_group_outgoing;
mod whisper_participant_info;

pub use error::SubroomAudioError;
pub use subroom_audio_events::SubroomAudioEvent;
pub use whisper_group_outgoing::WhisperGroupOutgoing;
pub use whisper_participant_info::WhisperParticipantInfo;
