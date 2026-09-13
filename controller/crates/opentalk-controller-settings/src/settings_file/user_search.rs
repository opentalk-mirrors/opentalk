// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{UserSearchBackend, UsersFindBehavior};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UserSearch {
    #[serde(flatten)]
    pub backend: Option<UserSearchBackend>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub users_find_behavior: Option<UsersFindBehavior>,
}
