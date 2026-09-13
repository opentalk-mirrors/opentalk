// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains shared types that are used by the signaling communication
//! (typically through websockets)

use crate::ParticipantId;

/// The target participant for specific commands or events
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TargetParticipant {
    /// The id of the target participant
    pub target: ParticipantId,
}
