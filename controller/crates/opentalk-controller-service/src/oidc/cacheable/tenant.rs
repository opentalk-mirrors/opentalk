// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use rkyv::{Archive, Deserialize, Serialize};
use snafu::{OptionExt as _, ResultExt as _};
use uuid::Uuid;

use super::{
    DecodeFromCacheError,
    decode_from_cache_error::{ConvertSnafu, ParseSnafu},
};

/// Information about a tenant
#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub struct Tenant {
    id: Uuid,
    created_at: i64,
    updated_at: i64,
    oidc_tenant_id: String,
}

impl From<opentalk_inventory::Tenant> for Tenant {
    fn from(
        opentalk_inventory::Tenant {
            id,
            created_at,
            updated_at,
            oidc_tenant_id,
        }: opentalk_inventory::Tenant,
    ) -> Self {
        Self {
            id: id.into(),
            created_at: DateTime::from(created_at).timestamp_millis(),
            updated_at: DateTime::from(updated_at).timestamp_millis(),
            oidc_tenant_id: oidc_tenant_id.to_string(),
        }
    }
}

impl TryFrom<Tenant> for opentalk_inventory::Tenant {
    type Error = DecodeFromCacheError;

    fn try_from(
        Tenant {
            id,
            created_at,
            updated_at,
            oidc_tenant_id,
        }: Tenant,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.into(),
            created_at: DateTime::from_timestamp_millis(created_at)
                .context(ConvertSnafu {
                    field: "created_at",
                    message: format!("Not a valid milliseconds value: {created_at}"),
                })?
                .into(),
            updated_at: DateTime::from_timestamp_millis(updated_at)
                .context(ConvertSnafu {
                    field: "updated_at",
                    message: format!("Not a valid milliseconds value: {updated_at}"),
                })?
                .into(),
            oidc_tenant_id: oidc_tenant_id
                .parse()
                .map_err(Into::into)
                .context(ParseSnafu {
                    field: "oidc_tenant_id",
                })?,
        })
    }
}
