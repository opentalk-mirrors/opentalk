// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Nextcloud-specific implementation of event shared folders

use std::collections::HashSet;

use chrono::{Days, NaiveDate, Utc};
use log::warn;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{EventSharedFolder, NewEventSharedFolder, SharedFolderProvider};
use opentalk_nextcloud_client::{Client, ShareId, SharePermission, ShareType};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{events::EventId, shared_folders::SharedFolderAccess};
use snafu::Report;
use url::Url;

/// Creates a shared folder for the specified event on a Nextcloud instance.
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
            warn!("Error creating NextCloud client: {}", Report::from_error(e));
            ApiError::internal().with_message("Error creating NextCloud client")
        })?;
    let path = format!(
        "{}/opentalk-event-{}",
        directory.trim_matches('/'),
        event_id
    );
    let user_path = format!("files/{username}/{path}");
    client.create_folder(&user_path).await.map_err(|e| {
        warn!(
            "Error creating folder on NextCloud: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error creating folder on NextCloud")
    })?;

    let expire_date = expiry.map(|days| Utc::now().date_naive() + Days::new(days));

    let write_permissions = HashSet::from([
        SharePermission::Read,
        SharePermission::Create,
        SharePermission::Update,
        SharePermission::Delete,
    ]);
    let read_permissions = HashSet::from([SharePermission::Read]);

    let write_password = generate_password(&client).await?;
    let read_password = generate_password(&client).await?;

    let (
        write_share_id,
        SharedFolderAccess {
            url: write_url,
            password: write_password,
        },
    ) = create_share(
        &client,
        &path,
        write_permissions,
        "OpenTalk read-write",
        write_password,
        expire_date,
    )
    .await?;
    let (
        read_share_id,
        SharedFolderAccess {
            url: read_url,
            password: read_password,
        },
    ) = create_share(
        &client,
        &path,
        read_permissions,
        "OpenTalk read-only",
        read_password,
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
        provider: SharedFolderProvider::Nextcloud {
            write_share_id: write_share_id.to_string(),
            read_share_id: read_share_id.to_string(),
        },
    })
}

/// Deletes the given shared folders from a Nextcloud instance.
pub(super) async fn delete_shared_folders(
    url: &Url,
    username: &str,
    password: &str,
    shared_folders: &[EventSharedFolder],
) -> Result<(), ApiError> {
    let client =
        Client::new(url.clone(), username.to_owned(), password.to_owned()).map_err(|e| {
            warn!("Error creating NextCloud client: {}", Report::from_error(e));
            ApiError::internal().with_message("Error creating NextCloud client")
        })?;
    for shared_folder in shared_folders {
        let path = &shared_folder.path;
        if path.trim_matches('/').is_empty() {
            warn!(
                "Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended"
            );
            return Err(ApiError::internal());
        }
        let (write_share_id, read_share_id) = match &shared_folder.provider {
            SharedFolderProvider::Nextcloud {
                write_share_id,
                read_share_id,
            } => (write_share_id, read_share_id),
            SharedFolderProvider::OpencloudBasic { .. } => {
                warn!("Skipping NextCloud deletion of a shared folder created by OpenCloud");
                continue;
            }
        };
        let user_path = format!("files/{username}/{path}");
        if let Err(e) = client
            .delete_share(ShareId::from(read_share_id.clone()))
            .await
        {
            warn!(
                "Could not delete NextCloud read share: {}",
                Report::from_error(e)
            );
        }
        if let Err(e) = client
            .delete_share(ShareId::from(write_share_id.clone()))
            .await
        {
            warn!(
                "Could not delete NextCloud write share: {}",
                Report::from_error(e)
            );
        }
        match client.delete(&user_path).await {
            Ok(()) | Err(opentalk_nextcloud_client::Error::FileNotFound { .. }) => {}
            Err(e) => {
                warn!(
                    "Error deleting folder on NextCloud: {}",
                    Report::from_error(e)
                );
                return Err(ApiError::internal().with_message("Error deleting folder on NextCloud"));
            }
        };
    }
    Ok(())
}

async fn create_share(
    client: &Client,
    path: &str,
    permissions: HashSet<SharePermission>,
    label: &str,
    password: String,
    expire_date: Option<NaiveDate>,
) -> Result<(ShareId, SharedFolderAccess), ApiError> {
    let mut creator = client
        .create_share(path, ShareType::PublicLink)
        .password(&password)
        .label(label);
    for permission in &permissions {
        creator = creator.permission(*permission);
    }
    if let Some(expire_date) = expire_date {
        creator = creator.expire_date(expire_date);
    }
    let share = creator.send().await.map_err(|e| {
        warn!(
            "Error creating share on NextCloud: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error creating share on NextCloud")
    })?;

    // Workaround for NextCloud up to version 25 not processing the share permissions
    // on folder creation. We just need to change them with a subsequent update request.
    //
    // See: https://github.com/nextcloud/server/issues/32611
    if share.data.permissions != permissions {
        _ = client
            .update_share(share.data.id.clone())
            .permissions(permissions)
            .await
            .map_err(|e| {
                warn!(
                    "Error setting permissions for share on NextCloud: {}",
                    Report::from_error(e)
                );
                ApiError::internal()
                    .with_message("Error setting permissions for share on NextCloud")
            })?;
    }

    Ok((
        share.data.id,
        SharedFolderAccess {
            url: share.data.url,
            password,
        },
    ))
}

async fn generate_password(client: &Client) -> Result<String, ApiError> {
    client.generate_password().await.map_err(|e| {
        warn!(
            "Error generating share password on NextCloud: {}",
            Report::from_error(e)
        );
        ApiError::internal().with_message("Error generating share password NextCloud")
    })
}
