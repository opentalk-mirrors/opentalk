// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides events stuff.

pub(crate) mod notifications;

use opentalk_controller_settings::{Settings, TenantAssignment};
use opentalk_inventory::{EventSharedFolder, Inventory, Tenant};
use opentalk_keycloak_admin::{KeycloakAdminClient, users::TenantFilter};
use opentalk_types_api_v1::{
    events::{EventInvitee, EventInviteeProfile},
    users::UnregisteredUser,
};
use opentalk_types_common::{
    events::EventId,
    pagination::{Page, PageSize},
    shared_folders::{SharedFolder, SharedFolderAccess},
    users::{Language, UserId},
};

use crate::services::{
    ExternalMailRecipient, MailRecipient, RegisteredMailRecipient, UnregisteredMailRecipient,
};

/// Gets the invited mail recipients for an event
pub async fn get_invited_mail_recipients_for_event(
    inventory: &mut dyn Inventory,
    event_id: EventId,
    default_user_language: Language,
) -> opentalk_inventory::Result<Vec<MailRecipient>> {
    // TODO(w.rabl) Further DB access optimization (replacing call to get_for_event_paginated)?
    let (invites_with_user, _) = inventory
        .get_event_invites_paginated(event_id, PageSize::MAX, Page::DEFAULT, None)
        .await?;
    let user_invitees = invites_with_user.into_iter().map(|(_, user)| {
        MailRecipient::Registered(RegisteredMailRecipient::from_inventory_user(
            user,
            default_user_language.clone(),
        ))
    });

    let (email_invites, _) = inventory
        .get_event_email_invites_paginated(event_id, PageSize::MAX, Page::DEFAULT)
        .await?;
    let email_invitees = email_invites.into_iter().map(|invitee| {
        MailRecipient::External(ExternalMailRecipient {
            email: invitee.email,
        })
    });

    let invitees = user_invitees.chain(email_invitees).collect();

    Ok(invitees)
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
pub async fn enrich_invitees_from_optional_user_search(
    settings: &Settings,
    user_search_client: &Option<KeycloakAdminClient>,
    current_tenant: &Tenant,
    invitees: Vec<EventInvitee>,
) -> Vec<EventInvitee> {
    let Some(user_search_client) = user_search_client else {
        return invitees;
    };
    enrich_invitees_from_user_search(settings, user_search_client, current_tenant, invitees).await
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
async fn enrich_invitees_from_user_search(
    settings: &Settings,
    user_search_client: &KeycloakAdminClient,
    current_tenant: &Tenant,
    invitees: Vec<EventInvitee>,
) -> Vec<EventInvitee> {
    let tenant_assignment = &settings.tenants.assignment;
    let invitee_mapping_futures = invitees.into_iter().map(|invitee| async move {
        if let EventInviteeProfile::Email(profile_details) = invitee.profile {
            let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

            let user_for_email = user_search_client
                .get_user_for_email(tenant_filter, profile_details.email.as_ref())
                .await
                .unwrap_or_default();

            if let Some(user) = user_for_email {
                let profile_details = UnregisteredUser {
                    email: profile_details.email,
                    firstname: user.first_name.unwrap_or_default(),
                    lastname: user.last_name.unwrap_or_default(),
                    avatar_url: profile_details.avatar_url,
                };
                EventInvitee {
                    profile: EventInviteeProfile::Unregistered(profile_details),
                    ..invitee
                }
            } else {
                EventInvitee {
                    profile: EventInviteeProfile::Email(profile_details),
                    ..invitee
                }
            }
        } else {
            invitee
        }
    });
    futures::future::join_all(invitee_mapping_futures).await
}

/// Gets a tenat filter
pub fn get_tenant_filter<'a>(
    current_tenant: &'a Tenant,
    tenant_assignment: &'a TenantAssignment,
) -> Option<TenantFilter<'a>> {
    match tenant_assignment {
        TenantAssignment::Static { .. } => None,
        TenantAssignment::ByExternalTenantId {
            external_tenant_id_user_attribute_name,
        } => Some(TenantFilter {
            field_name: external_tenant_id_user_attribute_name,
            id: current_tenant.oidc_tenant_id.as_ref(),
        }),
    }
}

/// Gets the share folder of a user
pub fn shared_folder_for_user(
    shared_folder: Option<EventSharedFolder>,
    event_created_by: UserId,
    current_user: UserId,
) -> Option<SharedFolder> {
    shared_folder.map(|f| {
        let EventSharedFolder {
            write_password,
            write_url,
            read_password,
            read_url,
            ..
        } = f;

        let read_write = if event_created_by == current_user {
            Some(SharedFolderAccess {
                url: write_url,
                password: write_password,
            })
        } else {
            None
        };

        let read = SharedFolderAccess {
            url: read_url,
            password: read_password,
        };

        SharedFolder { read, read_write }
    })
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
pub async fn enrich_from_optional_user_search(
    settings: &Settings,
    recipient: MailRecipient,
    current_tenant: &Tenant,
    user_search_client: &Option<KeycloakAdminClient>,
) -> MailRecipient {
    let Some(user_search_client) = user_search_client else {
        return recipient;
    };
    enrich_from_user_search(settings, recipient, current_tenant, user_search_client).await
}

/// Gets some additional information about invitees from Keycloak and attaches it to them
async fn enrich_from_user_search(
    settings: &Settings,
    recipient: MailRecipient,
    current_tenant: &Tenant,
    user_search_client: &KeycloakAdminClient,
) -> MailRecipient {
    let tenant_assignment = &settings.tenants.assignment;
    if let MailRecipient::External(recipient) = recipient {
        let tenant_filter = get_tenant_filter(current_tenant, tenant_assignment);

        let keycloak_user = user_search_client
            .get_user_for_email(tenant_filter, recipient.email.as_ref())
            .await
            .unwrap_or_default();

        if let Some(keycloak_user) = keycloak_user {
            MailRecipient::Unregistered(UnregisteredMailRecipient {
                email: recipient.email,
                first_name: keycloak_user.first_name.unwrap_or_default(),
                last_name: keycloak_user.last_name.unwrap_or_default(),
            })
        } else {
            MailRecipient::External(recipient)
        }
    } else {
        recipient
    }
}
