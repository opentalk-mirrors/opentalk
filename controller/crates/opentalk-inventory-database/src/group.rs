// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{Group, GroupInventory};
use opentalk_types_common::{tenants::TenantId, users::GroupName};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl GroupInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_or_create_groups_by_name(
        &mut self,
        groups: &[(TenantId, GroupName)],
    ) -> Result<Vec<Group>> {
        Ok(
            db::queries::groups::get_or_create_groups_by_name(&mut self.inner, groups)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }
}
