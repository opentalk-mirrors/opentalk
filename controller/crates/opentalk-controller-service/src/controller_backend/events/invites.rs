// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event invites

use std::collections::BTreeSet;

use chrono::Utc;
use opentalk_controller_api_authorization::authorization::{AuthorizationChange, Authorizer};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{
    Event, EventInvite, Inventory, InventoryProvider, NewEventEmailInvite, NewEventInvite, Room,
    RoomSipConfig, Tenant, UpdateEventEmailInvite, UpdateEventInvite, User, transaction,
    utils::is_room_guest_access_allowed,
};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        DeleteEventInvitePath, EmailInvite, EventInvitee, EventOptionsQuery, PatchEmailInviteBody,
        PatchInviteBody, PostEventInviteBody, PostEventInviteQuery, UserInvite,
        by_event_id::invites::GetEventsInvitesQuery,
    },
    pagination::PagePaginationQuery,
    users::GetEventInvitesPendingResponseBody,
};
use opentalk_types_common::{
    email::EmailAddress,
    events::{
        EventId,
        invites::{EmailInviteRole, EventInviteStatus, InviteRole},
    },
    pagination::{ItemCount, Page, PageSize},
    rooms::RoomId,
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    tariffs::TariffResource,
    users::{Language, UserId},
};
use snafu::Report;

use crate::{
    ControllerBackend,
    controller_backend::{events::EventInviteeExt, utils::verify_invite_write},
    events::{
        enrich_from_optional_user_search, enrich_invitees_from_optional_user_search,
        get_invited_mail_recipients_for_event, get_tenant_filter,
    },
    services::{
        ExternalMailRecipient, MailRecipient, MailService, RegisteredMailRecipient,
        UnregisteredMailRecipient,
    },
};

impl ControllerBackend {
    pub(crate) async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        GetEventsInvitesQuery {
            pagination: PagePaginationQuery { per_page, page },
            status: status_filter,
        }: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, PageSize, Page, ItemCount), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        // FIXME: Preliminary solution, consider using UNION when Diesel supports it.
        // As in #[get("/events")], we simply get all invitees and truncate them afterwards.
        // Note that get_for_event_paginated returns a total record count of 0 when paging beyond the end.

        let (event_invites_with_user, event_invites_total) = inventory
            .get_event_invites_paginated(event_id, PageSize::MAX, Page::DEFAULT, status_filter)
            .await?;

        let event_invitees_iter =
            event_invites_with_user
                .into_iter()
                .map(|(event_invite, user)| {
                    EventInvitee::from_invite_with_user(event_invite, user, &settings)
                });

        let (event_email_invites, event_email_invites_total) = inventory
            .get_event_email_invites_paginated(event_id, PageSize::MAX, Page::DEFAULT)
            .await?;

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;

        drop(inventory);

        let event_email_invitees_iter = event_email_invites.into_iter().map(|event_email_invite| {
            EventInvitee::from_email_invite(event_email_invite, &settings)
        });

        let invitees_to_skip = per_page.first_index_on_page_saturating(page);

        let invitees = event_invitees_iter
            .chain(event_email_invitees_iter)
            .skip(invitees_to_skip.into())
            .take(per_page.into())
            .collect();

        let invitees = enrich_invitees_from_optional_user_search(
            &settings,
            &self.user_search_client,
            &current_tenant,
            invitees,
        )
        .await;

        Ok((
            invitees,
            per_page,
            page,
            event_invites_total.saturating_add(event_email_invites_total),
        ))
    }

    pub(crate) async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let event = inventory.get_event(event_id).await?;
        let settings = self.settings_provider.get();
        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
        let current_user = inventory.get_user(current_user.id).await?;
        let room_tariff = self.get_room_tariff(event.room.into()).await?;

        match create_invite {
            PostEventInviteBody::User(user_invite) => {
                create_user_event_invite(
                    &settings,
                    inventory,
                    self.authorizer.clone(),
                    current_user,
                    event_id,
                    &room_tariff,
                    user_invite,
                    &mail_service,
                )
                .await
            }
            PostEventInviteBody::Email(email_invite) => {
                create_email_event_invite(
                    &settings,
                    self.inventory_provider.as_ref(),
                    self.authorizer.clone(),
                    &self.user_search_client,
                    &current_tenant,
                    &current_user,
                    event_id,
                    &room_tariff,
                    email_invite,
                    &mail_service,
                )
                .await
            }
        }
    }

    pub(crate) async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let event = inventory.get_event(event_id).await?;

        if event.created_by != current_user.id {
            return Err(ApiError::forbidden().into());
        }

        _ = inventory
            .update_event_user_invite(
                event_id,
                user_id,
                UpdateEventInvite {
                    status: None,
                    role: update_invite.role,
                },
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let event = inventory.get_event(event_id).await?;

        if event.created_by != current_user.id {
            return Err(ApiError::forbidden().into());
        }

        _ = inventory
            .update_event_email_invite(
                event_id,
                update_invite.email.as_str(),
                UpdateEventEmailInvite {
                    role: update_invite.role,
                },
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        DeleteEventInvitePath { event_id, user_id }: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let default_user_language = Language(settings.defaults.user_language.clone());

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        // TODO(w.rabl) Further DB access optimization (replacing call to get_with_invite_and_room)?
        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            tariff,
            _training_participation_report_parameter_set,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;
        let room_tariff = self.build_tariff_resource(&tariff)?;
        let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
        let current_user = inventory.get_user(current_user.id).await?;

        let created_by = if event.created_by == current_user.id {
            current_user.clone()
        } else {
            inventory.get_user(event.created_by).await?
        };

        let invited_users = get_invited_mail_recipients_for_event(
            inventory.as_mut(),
            event_id,
            default_user_language,
        )
        .await?;

        let (room_id, invite) = transaction(inventory.as_mut(), async |inventory| {
            // delete invite to the event
            let invite = inventory
                .delete_event_invite_by_invitee(event_id, user_id)
                .await?;

            // user access is going to be removed for the event, remove favorite entry if it exists
            _ = inventory
                .delete_event_favorite_for_user(event_id, current_user.id)
                .await?;

            let event = inventory.get_event(invite.event_id).await?;

            Ok::<_, opentalk_inventory::Error>((event.room, invite))
        })
        .await?;

        drop(inventory);

        if let Some(mail_service) = &mail_service {
            // Notify just the specified user. Currently, unlike the create_invite_to_event counterpart, this endpoint
            // only handles and notifies a single registered user. This somehow contradicts patch_event and delete_event
            // as well.
            // See this issue for more details: https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/499.
            let users_to_notify: Vec<MailRecipient> = invited_users
                .into_iter()
                .filter(|user| match user {
                    MailRecipient::Registered(user) => user.id == user_id,
                    MailRecipient::Unregistered(_) => false,
                    MailRecipient::External(_) => false,
                })
                .collect();

            let notification_values = UninviteNotificationValues {
                tenant: current_tenant,
                created_by,
                event,
                room,
                sip_config,
                users_to_notify,
            };

            notify_invitees_about_uninvite(
                &settings,
                notification_values,
                &room_tariff,
                mail_service,
                &self.user_search_client,
                shared_folder.map(SharedFolder::from),
                streaming_targets,
            )
            .await;
        }

        remove_invitee_permissions(self.authorizer.clone(), event_id, room_id, invite.invitee)
            .await?;

        Ok(())
    }

    pub(crate) async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let default_user_language = Language(settings.defaults.user_language.clone());
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let email = email.to_lowercase().to_string();

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
        let current_user = inventory.get_user(current_user.id).await?;

        let tenant_filter = get_tenant_filter(&current_tenant, &settings.tenants.assignment);

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            tariff,
            _training_participation_report_parameter_set,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;
        let room_tariff = self.build_tariff_resource(&tariff)?;
        let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

        let created_by = if event.created_by == current_user.id {
            current_user.clone()
        } else {
            inventory.get_user(event.created_by).await?
        };

        let user_from_db = inventory
            .get_user_by_email(current_tenant.id, &email)
            .await?;

        let mail_recipient = if let Some(user) = user_from_db {
            let user_id = user.id;

            transaction(inventory.as_mut(), async |inventory| {
                // delete invite to the event
                log::error!("deleting: {event_id}, {user_id}");

                _ = inventory
                    .delete_event_invite_by_invitee(event_id, current_user.id)
                    .await?;

                // user access is going to be removed for the event, remove favorite entry if it exists
                _ = inventory
                    .delete_event_favorite_for_user(event_id, current_user.id)
                    .await?;

                Ok::<_, opentalk_inventory::Error>(())
            })
            .await?;

            remove_invitee_permissions(self.authorizer.clone(), event_id, room.id, user_id).await?;

            MailRecipient::Registered(RegisteredMailRecipient {
                email,
                ..RegisteredMailRecipient::from_inventory_user(user, default_user_language)
            })
        } else if let Ok(Some(user)) = {
            if let Some(user_search_client) = &*self.user_search_client {
                user_search_client
                    .get_user_for_email(tenant_filter, email.as_ref())
                    .await
            } else {
                Ok(None)
            }
        } {
            _ = inventory
                .delete_event_invite_by_email(event_id, &email)
                .await?;

            MailRecipient::Unregistered(UnregisteredMailRecipient {
                email,
                first_name: user.first_name.unwrap_or_default(),
                last_name: user.last_name.unwrap_or_default(),
            })
        } else {
            _ = inventory
                .delete_event_invite_by_email(event_id, &email)
                .await?;

            MailRecipient::External(ExternalMailRecipient { email })
        };

        if let Some(mail_service) = &mail_service {
            let notification_values = UninviteNotificationValues {
                tenant: current_tenant,
                created_by,
                event,
                room,
                sip_config,
                users_to_notify: vec![mail_recipient],
            };

            notify_invitees_about_uninvite(
                &settings,
                notification_values,
                &room_tariff,
                mail_service,
                &self.user_search_client,
                shared_folder.map(SharedFolder::from),
                streaming_targets,
            )
            .await;
        }

        Ok(())
    }

    pub(crate) async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let event_invites = inventory.get_invites_pending_for_user(user_id).await?;

        Ok(GetEventInvitesPendingResponseBody {
            total_pending_invites: event_invites.len() as u32,
        })
    }

    pub(crate) async fn accept_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        _ = inventory
            .update_event_user_invite(
                event_id,
                user_id,
                UpdateEventInvite {
                    status: Some(EventInviteStatus::Accepted),
                    role: None,
                },
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        _ = inventory
            .update_event_user_invite(
                event_id,
                user_id,
                UpdateEventInvite {
                    status: Some(EventInviteStatus::Declined),
                    role: None,
                },
            )
            .await?;

        Ok(())
    }
}

#[expect(clippy::too_many_arguments)]
async fn create_user_event_invite(
    settings: &Settings,
    mut inventory: Box<dyn Inventory>,
    authorizer: Authorizer,
    inviter: User,
    event_id: EventId,
    room_tariff: &TariffResource,
    user_invite: UserInvite,
    mail_service: &Option<MailService>,
) -> Result<bool, CaptureApiError> {
    let (event, room, sip_config) = inventory
        .get_event_with_room_and_sip_config(event_id)
        .await?;
    let invitee = inventory
        .get_user_for_tenant(event.tenant_id, user_invite.invitee)
        .await?;
    let shared_folder = inventory
        .get_event_shared_folder(event_id)
        .await?
        .map(SharedFolder::from);
    let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

    if event.created_by == user_invite.invitee {
        return Ok(false);
    }

    let res = inventory
        .try_create_event_invite(NewEventInvite {
            event_id,
            invitee: user_invite.invitee,
            role: user_invite.role,
            created_by: inviter.id,
            created_at: None,
        })
        .await?;

    drop(inventory);

    match res {
        Some(_invite) => {
            authorizer
                .apply_changes(&[
                    AuthorizationChange::AddUserToEvents {
                        user: invitee.id,
                        role: InviteRole::User,
                        events: BTreeSet::from_iter([event.id]),
                    },
                    AuthorizationChange::AddUserToRooms {
                        user: invitee.id,
                        role: InviteRole::User,
                        rooms: BTreeSet::from_iter([event.room]),
                    },
                ])
                .await
                .map_err(|e| {
                    log::error!("Could not apply changes in the authorization database: {e:?}");
                    ApiError::internal()
                })?;

            if let Some(mail_service) = mail_service {
                mail_service
                    .send_registered_invite(
                        settings,
                        inviter,
                        event,
                        room,
                        room_tariff,
                        sip_config,
                        invitee,
                        shared_folder,
                        streaming_targets,
                    )
                    .await
                    .map_err(|e| {
                        log::warn!("Failed to send with MailService: {}", Report::from_error(e));
                        ApiError::internal()
                    })?;
            }

            Ok(true)
        }
        None => Ok(false),
    }
}

/// Create an invite to an event via email address
///
/// Checks first if a user exists with the email address in our database and creates a regular invite,
/// else checks if the email is registered with the Keycloak (or external invitee support is configured)
/// and then creates an email invite
#[expect(clippy::too_many_arguments)]
async fn create_email_event_invite(
    settings: &Settings,
    inventory_provider: &dyn InventoryProvider,
    authorizer: Authorizer,
    user_search_client: &Option<KeycloakAdminClient>,
    current_tenant: &Tenant,
    current_user: &User,
    event_id: EventId,
    room_tariff: &TariffResource,
    email_invite: EmailInvite,
    mail_service: &Option<MailService>,
) -> Result<bool, CaptureApiError> {
    #[expect(clippy::large_enum_variant)]
    enum UserState {
        ExistsAndIsAlreadyInvited,
        ExistsAndWasInvited {
            event: Event,
            room: Room,
            invitee: User,
            sip_config: Option<RoomSipConfig>,
            invite: EventInvite,
            shared_folder: Option<SharedFolder>,
            streaming_targets: Vec<RoomStreamingTarget>,
        },
        DoesNotExist {
            event: Event,
            room: Room,
            sip_config: Option<RoomSipConfig>,
            shared_folder: Option<SharedFolder>,
            streaming_targets: Vec<RoomStreamingTarget>,
            email: EmailAddress,
        },
    }

    let state = {
        let mut inventory = inventory_provider.get_inventory().await?;

        let email = email_invite.email.to_lowercase();
        let (event, room, sip_config) = inventory
            .get_event_with_room_and_sip_config(event_id)
            .await?;
        let shared_folder = inventory
            .get_event_shared_folder(event_id)
            .await?
            .map(SharedFolder::from);
        let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;
        let invitee_user = inventory
            .get_user_by_email(current_user.tenant_id, email.as_ref())
            .await?;

        if let Some(invitee_user) = invitee_user {
            if event.created_by == invitee_user.id {
                UserState::ExistsAndIsAlreadyInvited
            } else {
                let res = inventory
                    .try_create_event_invite(NewEventInvite {
                        event_id,
                        invitee: invitee_user.id,
                        role: email_invite.role.into(),
                        created_by: current_user.id,
                        created_at: None,
                    })
                    .await?;

                match res {
                    Some(invite) => UserState::ExistsAndWasInvited {
                        event,
                        room,
                        invitee: invitee_user,
                        sip_config,
                        invite,
                        shared_folder,
                        streaming_targets,
                    },
                    None => UserState::ExistsAndIsAlreadyInvited,
                }
            }
        } else {
            UserState::DoesNotExist {
                event,
                room,
                sip_config,
                shared_folder,
                streaming_targets,
                email,
            }
        }
    };

    match state {
        UserState::ExistsAndIsAlreadyInvited => Ok(false),
        UserState::ExistsAndWasInvited {
            event,
            room,
            invite,
            sip_config,
            invitee,
            shared_folder,
            streaming_targets,
        } => {
            authorizer
                .apply_changes(&[
                    AuthorizationChange::AddUserToRooms {
                        user: invite.invitee,
                        role: InviteRole::User,
                        rooms: BTreeSet::from_iter([room.id]),
                    },
                    AuthorizationChange::AddUserToEvents {
                        user: invite.invitee,
                        role: InviteRole::User,
                        events: BTreeSet::from_iter([event_id]),
                    },
                ])
                .await
                .map_err(|e| {
                    log::error!("Could not apply changes in the authorization database: {e:?}");
                    ApiError::internal()
                })?;

            if let Some(mail_service) = mail_service {
                mail_service
                    .send_registered_invite(
                        settings,
                        current_user.clone(),
                        event,
                        room,
                        room_tariff,
                        sip_config,
                        invitee,
                        shared_folder,
                        streaming_targets,
                    )
                    .await
                    .map_err(|e| {
                        log::warn!("Failed to send with MailService: {}", Report::from_error(e));
                        ApiError::internal()
                    })?;
            }

            Ok(true)
        }
        UserState::DoesNotExist {
            event,
            room,
            sip_config,
            shared_folder,
            streaming_targets,
            email,
        } => {
            create_invite_to_non_matching_email(
                settings,
                inventory_provider,
                user_search_client,
                mail_service,
                current_tenant,
                current_user.clone(),
                event,
                room,
                room_tariff,
                sip_config,
                email,
                email_invite.role,
                shared_folder,
                streaming_targets,
            )
            .await
        }
    }
}

/// Invite a given email to the event.
/// Will check if the email exists in Keycloak and sends an "unregistered" email invite
/// or (if configured) sends an "external" email invite to the given email address
#[expect(clippy::too_many_arguments)]
async fn create_invite_to_non_matching_email(
    settings: &Settings,
    inventory_provider: &dyn InventoryProvider,
    user_search_client: &Option<KeycloakAdminClient>,
    mail_service: &Option<MailService>,
    current_tenant: &Tenant,
    current_user: User,
    event: Event,
    room: Room,
    room_tariff: &TariffResource,
    sip_config: Option<RoomSipConfig>,
    email: EmailAddress,
    role: EmailInviteRole,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Result<bool, CaptureApiError> {
    let tenant_filter = get_tenant_filter(current_tenant, &settings.tenants.assignment);

    let invitee_user = if let Some(user_search_client) = user_search_client {
        user_search_client
            .get_user_for_email(tenant_filter, email.as_ref())
            .await
            .map_err(|e| {
                log::error!("Failed to query user for email: {}", Report::from_error(e));
                ApiError::internal()
            })?
    } else {
        None
    };

    if invitee_user.is_some()
        || (settings.endpoints.event_invite_external_email_address
            && is_room_guest_access_allowed(&room, room_tariff))
    {
        let inviter = current_user.clone();
        let invitee_email = email.clone();

        let mut inventory = inventory_provider.get_inventory().await?;

        let res = {
            let event_id = event.id;
            let current_user_id = current_user.id;

            inventory
                .try_create_event_email_invite(NewEventEmailInvite {
                    event_id,
                    email: email.into(),
                    role,
                    created_by: current_user_id,
                })
                .await?
        };

        match res {
            Some(_) => {
                if let (Some(invitee_user), Some(mail_service)) = (invitee_user, mail_service) {
                    mail_service
                        .send_unregistered_invite(
                            settings,
                            inviter,
                            event,
                            room,
                            room_tariff,
                            sip_config,
                            invitee_user,
                            shared_folder,
                            streaming_targets,
                        )
                        .await
                        .map_err(|e| {
                            log::warn!(
                                "Failed to send with MailService: {}",
                                Report::from_error(e)
                            );
                            ApiError::internal()
                        })?;
                } else {
                    // # DO NOT REMOVE
                    //
                    // Super duper important (for now).
                    //
                    // This is part of a hopefully soon-to-be legacy implementation, that aims to
                    // ensure that no event invite codes are created when e2ee is enabled.
                    //
                    // There is going to be refactor that addresses this in a more idiomatic manner,
                    // but in the meantime this check **cannot** be removed.
                    verify_invite_write(room_tariff, &room)?;

                    if let Some(mail_service) = mail_service {
                        mail_service
                            .send_external_invite(
                                settings,
                                inviter,
                                event,
                                room,
                                room_tariff,
                                sip_config,
                                invitee_email.as_ref(),
                                shared_folder,
                                streaming_targets,
                            )
                            .await
                            .map_err(|e| {
                                log::warn!(
                                    "Failed to send with MailService: {}",
                                    Report::from_error(e)
                                );
                                ApiError::internal()
                            })?;
                    }
                }

                Ok(true)
            }
            None => Ok(false),
        }
    } else {
        Err(ApiError::conflict()
            .with_code("unknown_email")
            .with_message(
                "Only emails registered with the systems are allowed to be used for invites",
            )
            .into())
    }
}

struct UninviteNotificationValues {
    pub tenant: Tenant,
    pub created_by: User,
    pub event: Event,
    pub room: Room,
    pub sip_config: Option<RoomSipConfig>,
    pub users_to_notify: Vec<MailRecipient>,
}

async fn remove_invitee_permissions(
    authorizer: Authorizer,
    event: EventId,
    room: RoomId,
    user: UserId,
) -> Result<(), ApiError> {
    authorizer
        .apply_changes(&[
            AuthorizationChange::RemoveUserFromRooms {
                user,
                rooms: BTreeSet::from_iter([room]),
            },
            AuthorizationChange::RemoveUserFromEvents {
                user,
                events: BTreeSet::from_iter([event]),
            },
        ])
        .await
        .map_err(|e| {
            log::error!("Could not apply changes in the authorization database: {e:?}");
            ApiError::internal()
        })?;

    Ok(())
}

/// Part of `DELETE /events/{event_id}/invites/{user_id}` (see [`delete_invite_to_event`]).
///
/// Notify invited users about the event deletion.
async fn notify_invitees_about_uninvite(
    settings: &Settings,
    notification_values: UninviteNotificationValues,
    room_tariff: &TariffResource,
    mail_service: &MailService,
    user_search_client: &Option<KeycloakAdminClient>,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) {
    // Don't send mails for past events.
    if let Some(date) = notification_values.event.date()
        && date.ends_at < Utc::now().into()
    {
        return;
    }

    for user in notification_values.users_to_notify {
        let invited_user = enrich_from_optional_user_search(
            settings,
            user,
            &notification_values.tenant,
            user_search_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_uninvite(
                settings,
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.room.clone(),
                room_tariff,
                notification_values.sip_config.clone(),
                invited_user,
                shared_folder.clone(),
                streaming_targets.clone(),
            )
            .await
        {
            log::error!(
                "Failed to send event uninvite with MailService, {}",
                Report::from_error(e)
            );
        }
    }
}
