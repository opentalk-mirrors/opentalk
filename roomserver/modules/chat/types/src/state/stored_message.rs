// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{MessageId, Scope};

/// Message type stores in redis
///
/// This needs to have a inner timestamp.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoredMessage {
    /// ID of message
    pub id: MessageId,

    /// ID of the participant who sent the message
    pub source: ParticipantId,

    /// Timestamp of when the message was sent
    pub timestamp: Timestamp,

    /// Content of the message
    pub content: String,

    /// Scope of the message
    #[serde(flatten)]
    pub scope: Scope,
}
