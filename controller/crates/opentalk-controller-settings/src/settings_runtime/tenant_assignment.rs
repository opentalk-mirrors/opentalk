// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// The default tenant id for static tenant assignment.
pub const DEFAULT_STATIC_TENANT_ID: &str = "OpenTalkDefaultTenant";

/// The default name for the external tenant id attribute from OIDC.
pub const DEFAULT_EXTERNAL_TENANT_ID_USER_ATTRIBUTE_NAME: &str = "tenant_id";

/// The tenant assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantAssignment {
    /// Static tenant assignment to a static preconfigured tenant id.
    Static {
        /// The static preconfigured tenant id.
        /// Default value is defined in [`DEFAULT_STATIC_TENANT_ID`].
        static_tenant_id: String,
    },
    /// Assignment to a tenant id that is handed over by the OIDC provider in an attribute.
    ByExternalTenantId {
        /// The name of the attribute from which the tenant id is read.
        /// Default value is defined in [`DEFAULT_EXTERNAL_TENANT_ID_USER_ATTRIBUTE_NAME`].
        external_tenant_id_user_attribute_name: String,
    },
}

impl From<settings_file::TenantAssignment> for TenantAssignment {
    fn from(value: settings_file::TenantAssignment) -> Self {
        match value {
            settings_file::TenantAssignment::Static { static_tenant_id } => Self::Static {
                static_tenant_id: static_tenant_id
                    .unwrap_or_else(|| DEFAULT_STATIC_TENANT_ID.to_string()),
            },
            settings_file::TenantAssignment::ByExternalTenantId {
                external_tenant_id_user_attribute_name,
            } => Self::ByExternalTenantId {
                external_tenant_id_user_attribute_name: external_tenant_id_user_attribute_name
                    .unwrap_or_else(|| DEFAULT_EXTERNAL_TENANT_ID_USER_ATTRIBUTE_NAME.to_string()),
            },
        }
    }
}

impl Default for TenantAssignment {
    fn default() -> Self {
        TenantAssignment::Static {
            static_tenant_id: DEFAULT_STATIC_TENANT_ID.to_string(),
        }
    }
}
