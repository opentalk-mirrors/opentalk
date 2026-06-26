// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::kick_reason::KickReason;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug)]
pub enum Instruction {
    Kick {
        participants: Vec<ParticipantId>,
        reason: KickReason,
    },
    Ban {
        participant: ParticipantId,
    },
    MoveToWaitingRoom {
        participant: ParticipantId,
    },
}
