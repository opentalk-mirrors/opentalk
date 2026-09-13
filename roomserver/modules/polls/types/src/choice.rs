// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::ChoiceId;

/// The choice for a poll
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    /// The id of the choice
    pub id: ChoiceId,
    /// The content of the choice
    pub content: String,
}
