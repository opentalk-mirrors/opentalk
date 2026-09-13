// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenCloud-specific implementation of event shared folders

use chrono::{DateTime, Days, Utc};
use log::warn;
use opentalk_controller_utils::{CaptureApiError, opencloud::ShareReference};
use opentalk_inventory::{EventSharedFolder, NewEventSharedFolder, SharedFolderProvider};
use opentalk_opencloud_client::{
    Client, CreateLinkOptions, CreatedShareLink, DriveId, SharingLinkType,
};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::events::EventId;
use snafu::Report;
use url::Url;

/// Creates a shared folder for the specified event on an OpenCloud instance.
pub(super) async fn create_shared_folder(
    url: &Url,
    username: &str,
    password: &str,
    directory: &str,
    expiry: Option<u64>,
    event_id: EventId,
) -> Result<NewEventSharedFolder, CaptureApiError> {
    let client =
        Client::new(url.clone(), username.to_owned(), password.to_owned()).map_err(|e| {
            warn!("Error creating OpenCloud client: {}", Report::from_error(e));
            ApiError::internal().with_message("Error creating OpenCloud client")
        })?;

    let drive = client.get_personal_drive().await.map_err(|e| {
        warn!(
            "Error retrieving OpenCloud personal drive: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error retrieving OpenCloud personal drive")
    })?;
    let drive_id = drive.id;

    let expire_date = expiry
        .map(|days| {
            Utc::now()
                .checked_add_days(Days::new(days))
                .ok_or_else(|| ApiError::bad_request().with_message("Invalid expiry date"))
        })
        .transpose()?;

    let path = format!(
        "{}/opentalk-event-{}",
        directory.trim_matches('/'),
        event_id
    );
    client.create_folder(&drive_id, &path).await.map_err(|e| {
        warn!(
            "Error creating folder on OpenCloud: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error creating folder on OpenCloud")
    })?;

    let write_password = generate_password(&client).await?;
    let read_password = generate_password(&client).await?;

    let (write_share_id, write_url) = create_link(
        &client,
        drive_id.clone(),
        &path,
        SharingLinkType::Edit,
        "OpenTalk read-write".to_string(),
        write_password.clone(),
        expire_date,
    )
    .await?;
    let (read_share_id, read_url) = create_link(
        &client,
        drive_id,
        &path,
        SharingLinkType::View,
        "OpenTalk read-only".to_string(),
        read_password.clone(),
        expire_date,
    )
    .await?;

    Ok(NewEventSharedFolder {
        event_id,
        path,
        write_url,
        write_password,
        read_url,
        read_password,
        provider: SharedFolderProvider::OpencloudBasic {
            write: write_share_id.into(),
            read: read_share_id.into(),
        },
    })
}

/// Deletes the given shared folders from an OpenCloud instance.
pub(super) async fn delete_shared_folders(
    url: Url,
    username: String,
    password: String,
    shared_folders: &[EventSharedFolder],
) -> Result<(), ApiError> {
    let client = Client::new(url, username, password).map_err(|e| {
        warn!("Error creating OpenCloud client: {}", Report::from_error(e));
        ApiError::internal().with_message("Error creating OpenCloud client")
    })?;

    for shared_folder in shared_folders {
        let path = &shared_folder.path;
        if path.trim_matches('/').is_empty() {
            warn!(
                "Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended"
            );
            return Err(ApiError::internal());
        }

        let (write, read) = match &shared_folder.provider {
            SharedFolderProvider::OpencloudBasic { write, read } => (write, read),
            SharedFolderProvider::Nextcloud { .. } => {
                warn!("Skipping OpenCloud deletion of a shared folder created by NextCloud");
                continue;
            }
        };

        let read_reference = ShareReference::from(read.clone());
        let write_reference = ShareReference::from(write.clone());

        delete_link(&client, &read_reference, "read").await;
        delete_link(&client, &write_reference, "write").await;

        match client.delete_folder(&write_reference.drive_id, path).await {
            Ok(()) | Err(opentalk_opencloud_client::Error::FolderNotFound { .. }) => {}
            Err(e) => {
                warn!(
                    "Error deleting folder on OpenCloud: {}",
                    Report::from_error(e)
                );
                return Err(ApiError::internal().with_message("Error deleting folder on OpenCloud"));
            }
        }
    }
    Ok(())
}

/// Creates a single share link and returns the encoded share id and the public url.
async fn create_link(
    client: &Client,
    drive_id: DriveId,
    path: &str,
    link_type: SharingLinkType,
    display_name: String,
    password: String,
    expiration: Option<DateTime<Utc>>,
) -> Result<(ShareReference, String), ApiError> {
    let CreatedShareLink {
        item_id,
        permission,
    } = client
        .create_link(
            &drive_id,
            path,
            CreateLinkOptions {
                link_type,
                expiration,
                password: Some(password.clone()),
                display_name: Some(display_name),
                quick_link: false,
            },
        )
        .await
        .map_err(|e| {
            warn!(
                "Error creating share on OpenCloud: {}",
                Report::from_error(e)
            );
            ApiError::internal().with_message("Error creating share on OpenCloud")
        })?;

    let reference = ShareReference {
        drive_id,
        item_id,
        permission_id: permission.id,
    };
    let url = permission.link.map(|link| link.web_url).ok_or_else(|| {
        warn!("Missing opencloud share link");
        ApiError::internal()
    })?;

    Ok((reference, url))
}

/// Deletes a single share link, ignoring the case where it no longer exists.
async fn delete_link(client: &Client, reference: &ShareReference, label: &str) {
    match client
        .delete_link(
            &reference.drive_id,
            &reference.item_id,
            &reference.permission_id,
        )
        .await
    {
        Ok(()) | Err(opentalk_opencloud_client::Error::PermissionNotFound { .. }) => {}
        Err(e) => {
            warn!(
                "Could not delete OpenCloud {label} share: {}",
                Report::from_error(e)
            );
        }
    }
}

async fn generate_password(client: &Client) -> Result<String, ApiError> {
    client.generate_password().await.map_err(|e| {
        warn!(
            "Error generating share password on OpenCloud: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error generating share password OpenCloud")
    })
}
