// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use openidconnect::AccessToken;
use opentalk_controller_api_authorization::authorization::{AuthorizationChange, Authorizer};
use opentalk_controller_service::{
    oidc::{OidcTokenHandler, OpenIdConnectUserInfo},
    phone_numbers::parse_phone_number,
};
use opentalk_controller_settings::{
    Settings, TariffAssignment, TariffStatusMapping, TenantAssignment,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{
    Inventory, InventoryProvider, NewUser, Tariff, Tenant, UpsertOutcome, User, transaction,
};
use opentalk_types_api_v1::error::{ApiError, AuthenticationError};
use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    features::GUESTS_ALLOWED_MODULE_FEATURE_ID,
    rooms::RoomId,
    tariffs::TariffStatus,
    tenants::TenantId,
    users::{DisplayName, GroupId, GroupName, UserTitle},
};

enum LoginResult {
    UserCreated {
        user: User,
        groups: BTreeSet<GroupId>,
        events: BTreeSet<EventId>,
        rooms: BTreeSet<RoomId>,
    },
    UserUpdated {
        user: User,
        groups_added_to: BTreeSet<GroupId>,
        groups_removed_from: BTreeSet<GroupId>,
    },
}

/// Provisions user data from a valid OIDC access token
/// Synchronizes user data from the access token with the inventory
pub(super) async fn provision_user(
    settings: &Settings,
    authorizer: &Authorizer,
    inventory_provider: &dyn InventoryProvider,
    oidc_ctx: &dyn OidcTokenHandler,
    access_token: &AccessToken,
) -> Result<(Tenant, User), CaptureApiError> {
    let info = oidc_ctx.user_info(access_token.clone()).await?;

    let mut inventory = inventory_provider.get_inventory().await?;

    // Get tariff depending on the configured assignment
    let (tariff, tariff_status) = match &settings.tariffs.assignment {
        TariffAssignment::Static { static_tariff_name } => (
            inventory.get_tariff_by_name(static_tariff_name).await?,
            TariffStatus::Default,
        ),
        TariffAssignment::ByExternalTariffId { status_mapping } => {
            let external_tariff_id = info.tariff_id.clone().ok_or_else(|| {
                ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("tariff_id missing in id_token claims")
            })?;

            let tariff = inventory
                .get_tariff_by_external_tariff_id(external_tariff_id.into())
                .await?
                .ok_or_else(|| {
                    ApiError::bad_request()
                        .with_code("invalid_tariff_id")
                        .with_message("JWT contained unknown tariff_id")
                })?;

            if let Some(mapping) = status_mapping.as_ref() {
                let status_name = info.tariff_status.clone().ok_or_else(|| {
                    ApiError::bad_request()
                        .with_code("invalid_claims")
                        .with_message("tariff_status missing in id_token claims")
                })?;

                let status = map_tariff_status_name(mapping, &status_name);

                let tariff = if matches!(status, TariffStatus::Downgraded) {
                    inventory
                        .get_tariff_by_name(&mapping.downgraded_tariff_name)
                        .await
                        .map_err(|_| {
                            ApiError::internal()
                                .with_code("invalid_configuration")
                                .with_message("Unable to load downgraded tariff")
                        })?
                } else {
                    tariff
                };
                (tariff, status)
            } else {
                (tariff, TariffStatus::Default)
            }
        }
    };

    // Get the tenant_id depending on the configured assignment
    let tenant_id = match &settings.tenants.assignment {
        TenantAssignment::Static { static_tenant_id } => static_tenant_id.clone(),
        TenantAssignment::ByExternalTenantId { .. } => info.tenant_id.clone().ok_or_else(|| {
            tracing::error!("Invalid access token, missing tenant_id");
            ApiError::unauthorized().with_www_authenticate(AuthenticationError::InvalidAccessToken)
        })?,
    };
    let tenant = inventory
        .get_or_create_tenant_by_oidc_id(&tenant_id.into())
        .await?;

    let groups: Vec<(TenantId, GroupName)> = info
        .groups
        .iter()
        .map(|group| (tenant.id, GroupName::from(group.clone())))
        .collect();
    let groups = inventory
        .get_or_create_groups_by_name(&groups)
        .await?
        .into_iter()
        .map(|g| g.id)
        .collect::<Vec<GroupId>>();
    let is_guest_feature_enabled = !tariff.is_feature_disabled(&GUESTS_ALLOWED_MODULE_FEATURE_ID);

    let login_result = {
        let tenant = tenant.clone();
        transaction(inventory.as_mut(), async |inventory| {
            create_or_update_user(
                inventory,
                tenant,
                info,
                settings,
                &groups,
                tariff,
                tariff_status,
            )
            .await
        })
        .await?
    };

    let user = update_core_user_permissions(authorizer, login_result).await?;

    authorizer
        .apply_change(&AuthorizationChange::UpdateUserTariffAssignment {
            user: user.id,
            is_guest_feature_enabled,
        })
        .await;

    Ok((tenant, user))
}

fn map_tariff_status_name(mapping: &TariffStatusMapping, name: &String) -> TariffStatus {
    if mapping.default.contains(name) {
        TariffStatus::Default
    } else if mapping.paid.contains(name) {
        TariffStatus::Paid
    } else if mapping.downgraded.contains(name) {
        TariffStatus::Downgraded
    } else {
        tracing::error!("Invalid tariff status value found: \"{name}\"");
        TariffStatus::Default
    }
}

async fn create_or_update_user(
    inventory: &mut dyn Inventory,
    tenant: Tenant,
    info: OpenIdConnectUserInfo,
    settings: &Settings,
    groups: &[GroupId],
    tariff: Tariff,
    tariff_status: TariffStatus,
) -> Result<LoginResult, CaptureApiError> {
    let display_name = build_info_display_name(&info);
    let enforce_display_name = settings.endpoints.disallow_custom_display_name;

    let phone_number = if let Some((call_in, phone_number)) =
        settings.call_in.as_ref().zip(info.phone_number.as_deref())
    {
        parse_phone_number(phone_number, call_in.default_country_code)
            .map(|p| p.format().mode(phonenumber::Mode::E164).to_string())
    } else {
        None
    };

    let outcome = inventory
        .create_or_update_user_by_oidc_sub(
            NewUser {
                oidc_sub: info.sub,
                email: info.email,
                title: UserTitle::new(),
                display_name,
                firstname: info.firstname,
                lastname: info.lastname,
                avatar_url: info.avatar_url,
                language: info.locale,
                phone: phone_number,
                tenant_id: tenant.id,
                tariff_id: tariff.id,
                tariff_status,
                timezone: info.timezone,
            },
            enforce_display_name,
        )
        .await?;

    // Set last_authenticated_at field to now for each new access token
    inventory
        .set_last_authenticated_at_to_now(outcome.clone().into_inner().id)
        .await?;

    match outcome {
        UpsertOutcome::Inserted(user) => {
            let groups = inventory.add_user_to_groups(user.id, groups).await?;

            let (events, rooms) = inventory
                .migrate_event_email_invites_to_user_invites(user.clone())
                .await?
                .into_iter()
                .unzip();

            Ok(LoginResult::UserCreated {
                user,
                groups,
                events,
                rooms,
            })
        }
        UpsertOutcome::Updated(user) => {
            let groups_added_to = inventory.add_user_to_groups(user.id, groups).await?;
            let groups_removed_from = inventory
                .remove_user_from_all_groups_except(user.id, groups)
                .await?;

            Ok(LoginResult::UserUpdated {
                user,
                groups_added_to,
                groups_removed_from,
            })
        }
    }
}

async fn update_core_user_permissions(
    authorizer: &Authorizer,
    db_result: LoginResult,
) -> Result<User, CaptureApiError> {
    match db_result {
        LoginResult::UserUpdated {
            user,
            groups_added_to,
            groups_removed_from,
        } => {
            authorizer
                .apply_changes(&[
                    AuthorizationChange::AddUserToGroups {
                        user: user.id,
                        groups: groups_added_to,
                    },
                    AuthorizationChange::RemoveUserFromGroups {
                        user: user.id,
                        groups: groups_removed_from,
                    },
                ])
                .await?;

            Ok(user)
        }
        LoginResult::UserCreated {
            user,
            groups,
            events,
            rooms,
        } => {
            authorizer
                .apply_changes(&[
                    AuthorizationChange::CreateUser { user: user.id },
                    AuthorizationChange::AddUserToGroups {
                        user: user.id,
                        groups,
                    },
                    AuthorizationChange::AddUserToEvents {
                        user: user.id,
                        role: InviteRole::User,
                        events,
                    },
                    AuthorizationChange::AddUserToRooms {
                        user: user.id,
                        role: InviteRole::User,
                        rooms,
                    },
                ])
                .await
                .map_err(|e| {
                    tracing::error!("Could not apply changes in the authorization database: {e:?}");
                    ApiError::internal()
                })?;

            Ok(user)
        }
    }
}

fn build_info_display_name(info: &OpenIdConnectUserInfo) -> DisplayName {
    DisplayName::from_str_lossy(
        &info
            .display_name
            .clone()
            .unwrap_or_else(|| format!("{} {}", info.firstname, info.lastname)),
    )
}
