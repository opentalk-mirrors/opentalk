// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_legal_vote::cancel::CancelReason;
use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedCancel {
    /// The display name of the user who canceled the vote
    pub user: DisplayName,
    /// The reason for the cancel
    #[serde(flatten)]
    pub reason: CancelReason,
}
