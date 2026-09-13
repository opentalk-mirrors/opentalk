// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use super::Choices;
use crate::PollId;

/// Command to vote in the poll
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    /// The id of the poll
    pub poll_id: PollId,

    /// The choices of the participant
    #[serde(flatten)]
    pub choices: Choices,
}
