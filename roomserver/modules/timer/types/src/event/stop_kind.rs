// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// The stop reason
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "participant_id")]
pub enum StopKind {
    /// The timer has been stopped by a moderator
    ByModerator(ParticipantId),
    /// The timers duration has expired
    Expired,
}
