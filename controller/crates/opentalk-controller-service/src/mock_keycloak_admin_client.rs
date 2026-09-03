// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Mock for [`opentalk_keycloak_admin::KeycloakAdminClient`].
//!
//! `KeycloakAdminClient` is defined in an external crate, so `mockall::automock`
//! can't be attached to its `impl` block. This replicates the subset of its API
//! used by [`crate::controller_backend`] as a standalone `mockall::mock!`.

use opentalk_keycloak_admin::{
    Result,
    users::{TenantFilter, User},
};

mockall::mock! {
    pub KeycloakAdminClient {
        pub async fn search_user(&self, search_str: &str, max_users: usize) -> Result<Vec<User>>;

        pub async fn search_user_filtered(
            &self,
            tenant_id_user_attribute_name: &str,
            tenant_id: &str,
            search_str: &str,
            max_users: usize,
        ) -> Result<Vec<User>>;

        pub async fn get_user_for_email<'a>(
            &self,
            tenant_filter: Option<TenantFilter<'a>>,
            email: &str,
        ) -> Result<Option<User>>;
    }
}
