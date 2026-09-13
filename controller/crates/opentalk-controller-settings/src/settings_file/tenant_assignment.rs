// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub(crate) enum TenantAssignment {
    Static {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        static_tenant_id: Option<String>,
    },
    ByExternalTenantId {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        external_tenant_id_user_attribute_name: Option<String>,
    },
}
