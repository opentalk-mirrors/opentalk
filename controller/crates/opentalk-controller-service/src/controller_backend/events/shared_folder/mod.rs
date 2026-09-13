// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event shared folders

use log::warn;
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{EventSharedFolder, Inventory};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{DeleteSharedFolderQuery, PutSharedFolderQuery},
};
use opentalk_types_common::{events::EventId, shared_folders::SharedFolder};

use crate::{
    ControllerBackend,
    events::{notifications::notify_event_invitees_about_update, shared_folder_for_user},
};

mod nextcloud;
mod opencloud;

impl ControllerBackend {
    pub(crate) async fn get_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<SharedFolder, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let event = inventory.get_event(event_id).await?;

        let shared_folder = SharedFolder::from(
            inventory
                .get_event_shared_folder(event_id)
                .await?
                .ok_or_else(ApiError::not_found)?,
        );

        let shared_folder = if event.created_by == current_user.id {
            shared_folder
        } else {
            shared_folder.without_write_access()
        };

        Ok(shared_folder)
    }

    pub(crate) async fn put_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PutSharedFolderQuery,
    ) -> Result<(SharedFolder, bool), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let (shared_folder, created) =
            put_shared_folder(&settings, event_id, inventory.as_mut()).await?;

        let (
            event,
            _invite,
            room,
            sip_config,
            _is_favorite,
            _shared_folder,
            tariff,
            _training_participation_report,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;
        let tariff = self.build_tariff_resource(&tariff)?;

        if let Some(mail_service) = &mail_service {
            let shared_folder_for_user = shared_folder_for_user(
                Some(shared_folder.clone()),
                event.created_by,
                current_user.id,
            );

            let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
            let current_user = inventory.get_user(current_user.id).await?;
            let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

            notify_event_invitees_about_update(
                &self.user_search_client,
                &settings,
                mail_service,
                current_tenant,
                current_user,
                inventory.as_mut(),
                event,
                room,
                &tariff,
                sip_config,
                shared_folder_for_user,
                streaming_targets,
            )
            .await?;
        }

        Ok((SharedFolder::from(shared_folder), created))
    }

    pub(crate) async fn delete_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteSharedFolderQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

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
            _training_participation_report,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;
        let tariff = self.build_tariff_resource(&tariff)?;

        if let Some(shared_folder) = shared_folder {
            let shared_folders = std::slice::from_ref(&shared_folder);
            let deletion = delete_shared_folders(&settings, shared_folders).await;

            let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

            match deletion {
                Ok(()) => {
                    inventory.delete_shared_folder_by_event_id(event_id).await?;

                    if let Some(mail_service) = &mail_service {
                        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
                        let current_user = inventory.get_user(current_user.id).await?;

                        notify_event_invitees_about_update(
                            &self.user_search_client,
                            &settings,
                            mail_service,
                            current_tenant,
                            current_user,
                            inventory.as_mut(),
                            event,
                            room,
                            &tariff,
                            sip_config,
                            None,
                            streaming_targets,
                        )
                        .await?;
                    }

                    Ok(())
                }
                Err(e) => {
                    if query.force_delete_reference_if_shared_folder_deletion_fails {
                        warn!(
                            "Deleting local shared folder reference anyway, because \
                        `force_delete_reference_if_shared_folder_deletion_fails` is set to true"
                        );
                        inventory.delete_shared_folder_by_event_id(event_id).await?;

                        if let Some(mail_service) = &mail_service {
                            let current_tenant =
                                inventory.get_tenant(current_user.tenant_id).await?;
                            let current_user = inventory.get_user(current_user.id).await?;

                            notify_event_invitees_about_update(
                                &self.user_search_client,
                                &settings,
                                mail_service,
                                current_tenant,
                                current_user,
                                inventory.as_mut(),
                                event,
                                room,
                                &tariff,
                                sip_config,
                                None,
                                streaming_targets,
                            )
                            .await?;
                        }

                        Ok(())
                    } else {
                        Err(e.into())
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

/// Adds a shared folder to the specified event
pub async fn put_shared_folder(
    settings: &Settings,
    event_id: EventId,
    inventory: &mut dyn Inventory,
) -> Result<(EventSharedFolder, bool), CaptureApiError> {
    let shared_folder = inventory.get_event_shared_folder(event_id).await?;

    if let Some(shared_folder) = shared_folder {
        return Ok((shared_folder, false));
    }
    let shared_folder_settings = settings.shared_folder.as_ref().ok_or_else(|| {
        ApiError::bad_request().with_message("No shared folder configured for this server")
    })?;

    let new_shared_folder = match shared_folder_settings {
        opentalk_controller_settings::SharedFolder::Nextcloud {
            url,
            username,
            password,
            directory,
            expiry,
        } => {
            nextcloud::create_shared_folder(url, username, password, directory, *expiry, event_id)
                .await?
        }
        opentalk_controller_settings::SharedFolder::Opencloud {
            url,
            username,
            password,
            directory,
            expiry,
        } => {
            opencloud::create_shared_folder(url, username, password, directory, *expiry, event_id)
                .await?
        }
    };

    let shared_folder = inventory
        .try_create_event_shared_folder(new_shared_folder)
        .await?
        .ok_or_else(ApiError::internal)?;

    Ok((shared_folder, true))
}

/// Deletes the shared folders for the specified event
pub async fn delete_shared_folders(
    settings: &Settings,
    shared_folders: &[EventSharedFolder],
) -> Result<(), ApiError> {
    if shared_folders.is_empty() {
        return Ok(());
    }

    let shared_folder_settings = if let Some(settings) = settings.shared_folder.as_ref() {
        settings
    } else {
        return Err(
            ApiError::bad_request().with_message("No shared folder configured for this server")
        );
    };

    match shared_folder_settings {
        opentalk_controller_settings::SharedFolder::Nextcloud {
            url,
            username,
            password,
            ..
        } => nextcloud::delete_shared_folders(url, username, password, shared_folders).await,
        opentalk_controller_settings::SharedFolder::Opencloud {
            url,
            username,
            password,
            ..
        } => {
            opencloud::delete_shared_folders(
                url.clone(),
                username.clone(),
                password.clone(),
                shared_folders,
            )
            .await
        }
    }
}
