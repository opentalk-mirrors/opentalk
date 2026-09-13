// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ChoiceId;

/// The choices of a vote
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Choices {
    /// A single choice. Takes precedence over `Multiple` during deserialization
    Single {
        /// The choice id
        choice_id: ChoiceId,
    },
    /// A multiple choice, `choice_ids` might be empty to abstain
    Multiple {
        /// The set of choice ids
        #[serde(default)]
        choice_ids: BTreeSet<ChoiceId>,
    },
}

impl Choices {
    /// Returns the choices as a BTreeSet
    pub fn to_hash_set(self) -> BTreeSet<ChoiceId> {
        match self {
            Self::Single { choice_id } => BTreeSet::from([choice_id]),
            Self::Multiple { choice_ids } => choice_ids,
        }
    }
}
