// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_types_livekit::{Credentials, LiveKitState, MicrophoneRestrictionState};
use opentalk_roomserver_types_moderation::event::ModerationEvent;
use opentalk_types::signaling::ParticipantId;

use crate::{Room, room::OpenTalkEvent};

/// OpenTalk livekit module events
#[derive(Debug)]
pub enum OpenTalkLiveKitEvent {
    MicrophoneRestrictionEnabled {
        unrestricted_participants: BTreeSet<ParticipantId>,
    },
    MicrophoneRestrictionDisabled,

    /// Participant was muted by a moderator
    MutedBy(ParticipantId),
}

pub struct LiveKitApi<'a> {
    pub(super) room: &'a mut Room,
}

impl LiveKitApi<'_> {
    #[must_use]
    fn state(&self) -> &RoomLiveKitState {
        self.room.livekit_state.as_ref().unwrap()
    }

    #[must_use]
    pub fn credentials(&self) -> &Credentials {
        &self.state().credentials
    }

    #[must_use]
    pub fn microphone_restriction_state(&self) -> &MicrophoneRestrictionState {
        &self.state().microphone_restriction_state
    }
}

#[derive(Debug)]
pub(super) struct RoomLiveKitState {
    pub(super) credentials: Credentials,

    // Set of participants that are allowed to talk, if the value is set
    microphone_restriction_state: MicrophoneRestrictionState,
}

impl RoomLiveKitState {
    pub(super) fn new(state: LiveKitState) -> Self {
        Self {
            credentials: state.credentials,
            microphone_restriction_state: state.microphone_restriction_state,
        }
    }

    pub(super) fn handle_moderation_event(
        &mut self,
        event: ModerationEvent,
    ) -> Option<OpenTalkEvent> {
        match event {
            ModerationEvent::MicrophoneRestrictionsEnabled {
                unrestricted_participants,
            } => {
                self.microphone_restriction_state = MicrophoneRestrictionState::Enabled {
                    unrestricted_participants: unrestricted_participants.clone(),
                };

                Some(OpenTalkEvent::LiveKit(
                    OpenTalkLiveKitEvent::MicrophoneRestrictionEnabled {
                        unrestricted_participants,
                    },
                ))
            }
            ModerationEvent::MicrophoneRestrictionsDisabled => {
                self.microphone_restriction_state = MicrophoneRestrictionState::Disabled;

                Some(OpenTalkEvent::LiveKit(
                    OpenTalkLiveKitEvent::MicrophoneRestrictionDisabled,
                ))
            }
            ModerationEvent::Muted { moderator } => Some(OpenTalkEvent::LiveKit(
                OpenTalkLiveKitEvent::MutedBy(moderator),
            )),
            _ => None,
        }
    }

    pub(super) fn can_publish(&self, id: ParticipantId) -> bool {
        match &self.microphone_restriction_state {
            MicrophoneRestrictionState::Disabled => true,
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => unrestricted_participants.contains(&id),
        }
    }
}
