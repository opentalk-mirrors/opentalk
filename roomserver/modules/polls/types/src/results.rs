// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{Item, PollId};

/// Represents the results of a completed poll
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Results {
    /// The id of the poll
    pub id: PollId,

    /// The poll items with their counts
    pub results: Vec<Item>,
}
