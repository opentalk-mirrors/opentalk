// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Specifies if the chat message is global, private or group message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "scope", content = "target", rename_all = "snake_case")]
pub enum Scope {
    /// Global scope for chat
    Global,

    /// Breakout scope for chat
    Breakout(BreakoutId),

    /// Private scope for chat
    Private(ParticipantId),
}
