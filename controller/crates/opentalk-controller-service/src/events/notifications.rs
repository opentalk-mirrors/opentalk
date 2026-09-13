// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event-related notifications.

use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{Event, EventException, Inventory, Room, RoomSipConfig, Tenant, User};
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_common::{
    rooms::RoomId, shared_folders::SharedFolder, streaming::RoomStreamingTarget,
    tariffs::TariffResource, users::Language,
};
use snafu::Report;

use crate::{
    events::{
        enrich_from_optional_user_search, get_invited_mail_recipients_for_event,
        shared_folder_for_user,
    },
    services::{MailRecipient, MailService, RegisteredMailRecipient},
};

/// Provides information for event update notifications (e.g. via email)
#[derive(Debug)]
pub struct UpdateNotificationValues {
    /// The tenant id
    pub tenant: Tenant,
    /// The user who has created the event
    pub created_by: User,
    /// The event that was updated
    pub event: Event,
    /// The event exception that was updated
    pub event_exception: Option<EventException>,
    /// The room of the updated event
    pub room: Room,
    /// The SIP configuration of the updated event
    pub sip_config: Option<RoomSipConfig>,
    /// The users to notify about the update
    pub users_to_notify: Vec<MailRecipient>,
}

/// Notifies the invitees of an event belonging to the specified room
#[expect(clippy::too_many_arguments)]
pub async fn notify_event_invitees_by_room_about_update(
    user_search_client: &Option<KeycloakAdminClient>,
    settings: &Settings,
    mail_service: &MailService,
    current_tenant: Tenant,
    current_user: User,
    inventory: &mut dyn Inventory,
    room_id: RoomId,
    room_tariff: &TariffResource,
) -> Result<(), CaptureApiError> {
    let event = inventory.get_event_for_room(&room_id.into()).await?;

    if let Some(event) = event {
        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            shared_folder,
            _tariff,
            _training_participation_report,
        ) = inventory
            .get_event_with_related_items(current_user.id, event.id)
            .await?;

        let shared_folder_for_user =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

        notify_event_invitees_about_update(
            user_search_client,
            settings,
            mail_service,
            current_tenant,
            current_user,
            inventory,
            event,
            room,
            room_tariff,
            sip_config,
            shared_folder_for_user,
            streaming_targets,
        )
        .await?;
    }
    Ok(())
}

/// Notifies the invitees of an event about updates
#[expect(clippy::too_many_arguments)]
pub async fn notify_event_invitees_about_update(
    user_search_client: &Option<KeycloakAdminClient>,
    settings: &Settings,
    mail_service: &MailService,
    current_tenant: Tenant,
    current_user: User,
    inventory: &mut dyn Inventory,
    event: Event,
    room: Room,
    room_tariff: &TariffResource,
    sip_config: Option<RoomSipConfig>,
    shared_folder_for_user: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Result<(), CaptureApiError> {
    let default_user_language = Language(settings.defaults.user_language.clone());

    let invited_users =
        get_invited_mail_recipients_for_event(inventory, event.id, default_user_language.clone())
            .await?;
    let current_user_mail_recipient = MailRecipient::Registered(
        RegisteredMailRecipient::from_inventory_user(current_user.clone(), default_user_language),
    );
    let users_to_notify = invited_users
        .into_iter()
        .chain(std::iter::once(current_user_mail_recipient))
        .collect::<Vec<_>>();
    let created_by = if event.created_by == current_user.id {
        current_user
    } else {
        inventory.get_user(event.created_by).await?
    };

    let notification_values = UpdateNotificationValues {
        tenant: current_tenant,
        created_by,
        event,
        event_exception: None,
        room,
        sip_config,
        users_to_notify,
    };

    notify_invitees_about_update(
        settings,
        room_tariff,
        notification_values,
        mail_service,
        user_search_client,
        shared_folder_for_user,
        streaming_targets,
    )
    .await;
    Ok(())
}

/// Notifies the invitees of an event about updates
pub async fn notify_invitees_about_update(
    settings: &Settings,
    room_tariff: &TariffResource,
    notification_values: UpdateNotificationValues,
    mail_service: &MailService,
    user_search_client: &Option<KeycloakAdminClient>,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) {
    for user in notification_values.users_to_notify {
        let invited_user = enrich_from_optional_user_search(
            settings,
            user,
            &notification_values.tenant,
            user_search_client,
        )
        .await;

        if let Err(e) = mail_service
            .send_event_update(
                settings,
                notification_values.created_by.clone(),
                notification_values.event.clone(),
                notification_values.event_exception.clone(),
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
                "Failed to send event update with MailService, {}",
                Report::from_error(e)
            );
        }
    }
}
