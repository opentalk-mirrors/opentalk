// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::AccessToken;
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::{
    TenantAssignment, UserSearchBackend, UserSearchBackendKeycloak,
    settings_file::UsersFindBehavior,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{Inventory, UpdateUser};
use opentalk_types_api_v1::{
    assets::AssetSortingQuery,
    error::ApiError,
    pagination::PagePaginationQuery,
    users::{
        GetFindQuery, GetFindResponseBody, GetFindResponseEntry, GetUserAssetsResponseBody,
        PrivateUserProfile, PublicUserProfile, UnregisteredUser, UserAssetResource,
        me::PatchMeRequestBody,
    },
};
use opentalk_types_common::{
    pagination::{ItemCount, Page, PageSize},
    tariffs::TariffResource,
    time::Timestamp,
    users::UserId,
};
use snafu::{ResultExt, Whatever};

use crate::{
    ControllerBackend, ToUserProfile, email_to_libravatar_url, helpers::asset_to_asset_resource,
};

impl ControllerBackend {
    pub(crate) async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
        access_token: &AccessToken,
    ) -> Result<Option<PrivateUserProfile>, CaptureApiError> {
        if patch.is_empty() {
            return Ok(None);
        }

        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        // Prohibit display name editing, if configured
        if settings.endpoints.disallow_custom_display_name
            && let Some(display_name) = &patch.display_name
            && &current_user.display_name != display_name
        {
            return Err(ApiError::bad_request()
                .with_message("changing the display name is prohibited")
                .into());
        }

        let user = inventory
            .update_user(
                current_user.id,
                UpdateUser {
                    title: patch.title.as_ref(),
                    firstname: None,
                    lastname: None,
                    avatar_url: None,
                    timezone: None,
                    phone: None,
                    email: None,
                    display_name: patch.display_name.as_ref(),
                    language: patch.language.map(|lang| Some(lang.into())),
                    dashboard_theme: patch.dashboard_theme.as_ref().map(Some),
                    conference_theme: patch.conference_theme.as_ref().map(Some),
                    tariff_id: None,
                    tariff_status: None,
                    disabled_since: None,
                    updated_at: Timestamp::now(),
                },
            )
            .await?;
        let used_storage = inventory
            .get_user_storage_used_size_u64(current_user.id)
            .await?;

        let user_profile = user.to_private_user_profile(&settings, used_storage);

        // Update the access token cache as well to reflect the changes immediately.
        let tenant = inventory.get_tenant(user.tenant_id).await?;
        let value = Ok((tenant, user));

        if let Err(e) = self
            .oidc_cache
            .update_access_token(access_token, value)
            .await
        {
            log::warn!("Failed to update user profile in the access token cache: {e}");
            return Err(ApiError::internal()
                .with_message("Failed to update user profile in the access token cache")
                .into());
        }

        Ok(Some(user_profile))
    }

    pub(crate) async fn get_me(
        &self,
        current_user: RequestUser,
    ) -> Result<PrivateUserProfile, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let used_storage = inventory
            .get_user_storage_used_size_u64(current_user.id)
            .await?;

        let user_profile = current_user.to_private_user_profile(&settings, used_storage);

        Ok(user_profile)
    }

    pub(crate) async fn get_my_tariff(
        &self,
        current_user: RequestUser,
    ) -> Result<TariffResource, CaptureApiError> {
        self.get_tariff(current_user.tariff_id).await
    }

    pub(crate) async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, ItemCount), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (owned_assets, asset_count) = get_all_assets_for_room_owner_paginated_ordered(
            inventory.as_mut(),
            current_user.id,
            pagination.per_page,
            pagination.page,
            sorting,
        )
        .await?;

        Ok((GetUserAssetsResponseBody { owned_assets }, asset_count))
    }

    pub(crate) async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let user = inventory
            .get_user_for_tenant(current_user.tenant_id, user_id)
            .await?;

        let user_profile = user.to_public_user_profile(&settings);

        Ok(user_profile)
    }

    pub(crate) async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();

        const MAX_USER_SEARCH_RESULTS: usize = 20;

        if settings.users_find_behavior == UsersFindBehavior::Disabled {
            return Err(ApiError::not_found().into());
        }

        if query.q.len() < 3 {
            return Err(ApiError::bad_request()
                .with_code("query_too_short")
                .with_message("query must be at least 3 characters long")
                .into());
        }

        let mut inventory = self.inventory_provider.get_inventory().await?;

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;

        // Get all users from Keycloak matching the search criteria
        let found_users = if settings.users_find_behavior
            == UsersFindBehavior::FromUserSearchBackend
        {
            let (
                Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                    external_id_user_attribute_name,
                    ..
                })),
                Some(user_search_client),
            ) = (&settings.user_search_backend, &*self.user_search_client)
            else {
                return Err(ApiError::internal()
                    .with_code("misconfiguration")
                    .with_message("search backend not properly configured")
                    .into());
            };
            let mut found_kc_users = match &settings.tenants.assignment {
                TenantAssignment::Static { .. } => {
                    // Do not filter by tenant_id if the assignment is static, since that's used
                    // when Keycloak does not provide any tenant information we can filter over anyway
                    user_search_client
                        .search_user(&query.q, MAX_USER_SEARCH_RESULTS)
                        .await
                        .whatever_context::<&str, Whatever>(
                            "Failed to search for user in Keycloak",
                        )?
                }
                TenantAssignment::ByExternalTenantId {
                    external_tenant_id_user_attribute_name,
                } => {
                    // Keycloak must contain information about the tenancy of a user,
                    // so we pass in the tenant_id to filter the found users
                    user_search_client
                        .search_user_filtered(
                            external_tenant_id_user_attribute_name,
                            current_tenant.oidc_tenant_id.as_ref(),
                            &query.q,
                            MAX_USER_SEARCH_RESULTS,
                        )
                        .await
                        .whatever_context::<&str, Whatever>(
                            "Failed to search for user in Keycloak",
                        )?
                }
            };

            // Get the name of the user attribute providing the Keycloak user ids. If set to None,
            // Keycloak's id field is used for user assignment instead.
            let user_attribute_name = external_id_user_attribute_name.as_deref();

            // Get the Keycloak user ids for Keycloak users found above, omitting users for whom no Keycloak
            // user id is provided
            let keycloak_user_ids: Vec<&str> = found_kc_users
                .iter()
                .filter_map(|kc_user| kc_user.get_keycloak_user_id(user_attribute_name))
                .collect();

            // For all Keycloak users found above, get the according users already registered from the database
            let registered_users = inventory
                .get_users_by_odic_subs(current_tenant.id, &keycloak_user_ids)
                .await?;

            // From the list of Keycloak users found above, remove all users already registered
            found_kc_users.retain(|kc_user| {
                if let Some(keycloak_user_id) = kc_user.get_keycloak_user_id(user_attribute_name) {
                    !registered_users
                        .iter()
                        .any(|user| user.oidc_sub == keycloak_user_id)
                } else {
                    true
                }
            });

            // Build a list of registered and unregistered users resulting from the search
            registered_users
                .into_iter()
                .map(|user| {
                    GetFindResponseEntry::Registered(user.to_public_user_profile(&settings))
                })
                .chain(found_kc_users.into_iter().map(|kc_user| {
                    let avatar_url =
                        email_to_libravatar_url(&settings.avatar.libravatar_url, &kc_user.email);

                    GetFindResponseEntry::Unregistered(UnregisteredUser {
                        email: kc_user.email,
                        firstname: kc_user.first_name.unwrap_or_default(),
                        lastname: kc_user.last_name.unwrap_or_default(),
                        avatar_url,
                    })
                }))
                .collect()
        } else {
            let found_users = inventory
                .find_users(current_tenant.id, &query.q, MAX_USER_SEARCH_RESULTS)
                .await?;

            found_users
                .into_iter()
                .map(|user| {
                    GetFindResponseEntry::Registered(user.to_public_user_profile(&settings))
                })
                .collect()
        };

        Ok(GetFindResponseBody(found_users))
    }
}

#[tracing::instrument(err(level = "debug"), skip_all)]
async fn get_all_assets_for_room_owner_paginated_ordered(
    inventory: &mut dyn Inventory,
    user_id: UserId,
    limit: PageSize,
    page: Page,
    sorting: AssetSortingQuery,
) -> Result<(Vec<UserAssetResource>, ItemCount), opentalk_inventory::Error> {
    let AssetSortingQuery { sort, order } = sorting;

    let (resources, total) = inventory
        .get_all_assets_for_room_owner_paginated_ordered(user_id, limit, page, sort, order)
        .await?;

    let resources = resources
        .into_iter()
        .map(|(asset, room_id, event_id)| {
            UserAssetResource::new(asset_to_asset_resource(asset), room_id, event_id)
        })
        .collect();

    Ok((resources, total))
}
