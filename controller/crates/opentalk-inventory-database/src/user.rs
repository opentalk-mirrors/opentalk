// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_db_storage as db;
use opentalk_inventory::{NewUser, UpdateUser, UpsertOutcome, User, UserInventory};
use opentalk_types_common::{
    tenants::TenantId,
    time::Timestamp,
    users::{GroupId, UserId},
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl UserInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_user(&mut self, new_user: NewUser) -> Result<User> {
        Ok(
            db::queries::users::create_user(&mut self.inner, new_user.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_user(&mut self, user_id: UserId) -> Result<User> {
        Ok(db::queries::users::get_user(&mut self.inner, user_id)
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_user<'a>(&mut self, user_id: UserId, user: UpdateUser<'a>) -> Result<User> {
        Ok(
            db::queries::users::update_user(&mut self.inner, user.into(), user_id)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_user(&mut self, user_id: UserId) -> Result<()> {
        Ok(db::queries::users::delete_user(&mut self.inner, user_id)
            .await
            .context(DatabaseSnafu)?)
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn set_last_authenticated_at_to_now(&mut self, user_id: UserId) -> Result<()> {
        Ok(
            db::queries::users::set_last_authenticated_at_to_now(&mut self.inner, user_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_users(&mut self) -> Result<Vec<User>> {
        Ok(db::queries::users::get_all_users(&mut self.inner)
            .await
            .context(DatabaseSnafu)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_users_by_ids(&mut self, user_ids: &[UserId]) -> Result<Vec<User>> {
        Ok(
            db::queries::users::get_users_by_ids(&mut self.inner, user_ids)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_user_ids_disabled_before(&mut self, timestamp: Timestamp) -> Result<Vec<UserId>> {
        Ok(
            db::queries::users::get_user_ids_disabled_before(&mut self.inner, timestamp.into())
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_user_by_email(
        &mut self,
        tenant_id: TenantId,
        email_address: &str,
    ) -> Result<Option<User>> {
        Ok(
            db::queries::users::get_user_by_email(&mut self.inner, tenant_id, email_address)
                .await
                .context(DatabaseSnafu)?
                .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_or_update_user_by_oidc_sub(
        &mut self,
        user: NewUser,
        enforce_display_name_on_update: bool,
    ) -> Result<UpsertOutcome<User>> {
        let user = db::queries::users::create_or_update_user_by_oidc_sub(
            &mut self.inner,
            user.into(),
            enforce_display_name_on_update,
        )
        .await
        .context(DatabaseSnafu)?;
        if user.created_at == user.updated_at {
            Ok(UpsertOutcome::Inserted(user.into()))
        } else {
            Ok(UpsertOutcome::Updated(user.into()))
        }
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_users_by_odic_subs(
        &mut self,
        tenant_id: TenantId,
        subs: &[&str],
    ) -> Result<Vec<User>> {
        Ok(
            db::queries::users::get_all_by_oidc_subs(&mut self.inner, tenant_id, subs)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_user_for_tenant(&mut self, tenant_id: TenantId, user_id: UserId) -> Result<User> {
        Ok(
            db::queries::users::get_user_by_tenant(&mut self.inner, tenant_id, user_id)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn add_user_to_groups(
        &mut self,
        user_id: UserId,
        groups: &[GroupId],
    ) -> Result<BTreeSet<GroupId>> {
        Ok(
            db::queries::groups::insert_user_into_groups(&mut self.inner, user_id, groups)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn remove_user_from_all_groups_except(
        &mut self,
        user_id: UserId,
        group_ids_to_keep: &[GroupId],
    ) -> Result<BTreeSet<GroupId>> {
        Ok(db::queries::groups::remove_user_from_all_groups_except(
            &mut self.inner,
            user_id,
            group_ids_to_keep,
        )
        .await
        .context(DatabaseSnafu)?)
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn remove_user_from_all_groups(&mut self, user_id: UserId) -> Result<()> {
        Ok(
            db::queries::groups::remove_user_from_all_groups(&mut self.inner, user_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_user_storage_used_size_u64(&mut self, user_id: UserId) -> Result<u64> {
        Ok(
            db::queries::users::get_used_storage_used_size_u64(&mut self.inner, &user_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn find_users(
        &mut self,
        tenant_id: TenantId,
        search_string: &str,
        limit: usize,
    ) -> Result<Vec<User>> {
        Ok(
            db::queries::users::find_users(&mut self.inner, tenant_id, search_string, limit)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }
}
