// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

use super::ResolvedCancel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum StopReason {
    ByUser { user: DisplayName },
    Auto,
    Expired,
    Canceled(ResolvedCancel),
}
