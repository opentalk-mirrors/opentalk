// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_legal_vote::issue::Issue;
use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "event_details")]
pub enum Event {
    UserJoined {
        name: Option<DisplayName>,
    },
    UserLeft {
        name: Option<DisplayName>,
    },
    Issue {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<DisplayName>,

        #[serde(flatten)]
        issue: Issue,
    },
}
