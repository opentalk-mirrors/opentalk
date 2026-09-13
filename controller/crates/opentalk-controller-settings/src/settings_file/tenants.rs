// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::TenantAssignment;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Tenants {
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub assignment: Option<TenantAssignment>,
}
