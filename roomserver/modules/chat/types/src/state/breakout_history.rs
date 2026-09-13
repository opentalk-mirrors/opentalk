// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;
use serde::{Deserialize, Serialize};

use crate::state::ChatChunk;

/// Group chat history
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakoutHistory {
    /// Id of the breakout room
    pub breakout_id: BreakoutId,

    /// Group chat history
    pub history: ChatChunk,
}
