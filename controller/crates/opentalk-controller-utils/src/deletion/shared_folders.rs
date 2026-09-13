// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_inventory::{EventSharedFolder, SharedFolderProvider};
use opentalk_log::{debug, warn};
use opentalk_nextcloud_client::ShareId;
use snafu::{Report, whatever};

use super::error::Error;
use crate::opencloud::ShareReference;

/// Delete a list of shared folders from the remote system and the database
pub async fn delete_shared_folders(
    logger: &dyn Log,
    settings: &Settings,
    shared_folders: &[EventSharedFolder],
    fail_on_error: bool,
) -> Result<(), Error> {
    if shared_folders.is_empty() {
        debug!(log: logger, "No shared folders to delete");
        return Ok(());
    }

    debug!(log: logger, "Reading shared folder settings");
    let shared_folder_settings = settings
        .shared_folder
        .as_ref()
        .ok_or(Error::SharedFoldersNotConfigured)?;

    match shared_folder_settings {
        opentalk_controller_settings::SharedFolder::Nextcloud {
            url,
            username,
            password,
            ..
        } => {
            delete_nextcloud_shared_folders(
                logger,
                url,
                username,
                password,
                shared_folders,
                fail_on_error,
            )
            .await
        }
        opentalk_controller_settings::SharedFolder::Opencloud {
            url,
            username,
            password,
            ..
        } => {
            delete_opencloud_shared_folders(
                logger,
                url,
                username,
                password,
                shared_folders,
                fail_on_error,
            )
            .await
        }
    }
}

/// Delete a list of shared folders from a NextCloud instance
async fn delete_nextcloud_shared_folders(
    logger: &dyn Log,
    url: &url::Url,
    username: &str,
    password: &str,
    shared_folders: &[EventSharedFolder],
    fail_on_error: bool,
) -> Result<(), Error> {
    debug!(log: logger, "Creating NextCloud client");
    let create_client_result = opentalk_nextcloud_client::Client::new(
        url.clone(),
        username.to_owned(),
        password.to_owned(),
    );
    let client = match create_client_result {
        Ok(c) => c,
        Err(e) => {
            warn!(log: logger, "Error creating NextCloud client: {}", Report::from_error(&e));
            if fail_on_error {
                return Err(e.into());
            }
            return Ok(());
        }
    };
    for shared_folder in shared_folders {
        let path = &shared_folder.path;
        debug!(log: logger, "Deleting shared folder from path {path}");
        if path.trim_matches('/').is_empty() {
            let message = "Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended";
            if fail_on_error {
                whatever!("{message}");
            }
            warn!(log: logger, "{}", message);
            continue;
        }
        let (write_share_id, read_share_id) = match &shared_folder.provider {
            SharedFolderProvider::Nextcloud {
                write_share_id,
                read_share_id,
            } => (write_share_id, read_share_id),
            SharedFolderProvider::OpencloudBasic { .. } => {
                let message = "Skipping NextCloud deletion of a shared folder created by OpenCloud";
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
                continue;
            }
        };
        let user_path = format!("files/{username}/{path}");
        if let Err(e) = client
            .delete_share(ShareId::from(read_share_id.clone()))
            .await
        {
            let message = format!(
                "Could not delete NextCloud read share: {}",
                Report::from_error(e)
            );
            if fail_on_error {
                whatever!("{message}");
            }
            warn!(log: logger, "{}", message);
        } else {
            debug!(
                log: logger,
                "Deleted read share {:?} from NextCloud", read_share_id
            );
        }
        if let Err(e) = client
            .delete_share(ShareId::from(write_share_id.clone()))
            .await
        {
            let message = format!(
                "Could not delete NextCloud write share: {}",
                Report::from_error(e)
            );
            if fail_on_error {
                whatever!("{message}");
            }
            warn!(log: logger, "{}", message);
        } else {
            debug!(
                log: logger,
                "Deleted write share {:?} from NextCloud", write_share_id
            );
        }
        match client.delete(&user_path).await {
            Ok(()) => {
                debug!(
                    log: logger,
                    "Deleted shared folder {user_path:?} from NextCloud"
                );
            }
            Err(opentalk_nextcloud_client::Error::FileNotFound { file_path, .. }) => {
                debug!(
                    log: logger,
                    "Path {file_path:?} not found on the NextCloud instance, skipping deletion"
                );
            }
            Err(e) => {
                let message = format!(
                    "Error deleting folder on NextCloud: {}",
                    Report::from_error(e)
                );
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
            }
        }
    }
    Ok(())
}

/// Delete a list of shared folders from an OpenCloud instance
async fn delete_opencloud_shared_folders(
    logger: &dyn Log,
    url: &url::Url,
    username: &str,
    password: &str,
    shared_folders: &[EventSharedFolder],
    fail_on_error: bool,
) -> Result<(), Error> {
    debug!(log: logger, "Creating OpenCloud client");
    let create_client_result = opentalk_opencloud_client::Client::new(
        url.clone(),
        username.to_owned(),
        password.to_owned(),
    );
    let client = match create_client_result {
        Ok(c) => c,
        Err(e) => {
            warn!(log: logger, "Error creating OpenCloud client: {}", Report::from_error(&e));
            if fail_on_error {
                return Err(e.into());
            }
            return Ok(());
        }
    };
    for shared_folder in shared_folders {
        let path = &shared_folder.path;
        debug!(log: logger, "Deleting shared folder from path {path}");
        if path.trim_matches('/').is_empty() {
            let message = "Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended";
            if fail_on_error {
                whatever!("{message}");
            }
            warn!(log: logger, "{}", message);
            continue;
        }

        let (write_reference, read_reference) = match &shared_folder.provider {
            SharedFolderProvider::OpencloudBasic { write, read } => (
                Some(ShareReference::from(write.clone())),
                Some(ShareReference::from(read.clone())),
            ),
            SharedFolderProvider::Nextcloud { .. } => {
                let message = "Skipping OpenCloud deletion of a shared folder created by NextCloud";
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
                continue;
            }
        };

        if let Some(reference) = &read_reference {
            if let Err(e) = delete_opencloud_link(&client, reference).await {
                let message = format!(
                    "Could not delete OpenCloud read share: {}",
                    Report::from_error(e)
                );
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
            } else {
                debug!(
                    log: logger,
                    "Deleted read share {:?} from OpenCloud", reference.permission_id
                );
            }
        }
        if let Some(reference) = &write_reference {
            if let Err(e) = delete_opencloud_link(&client, reference).await {
                let message = format!(
                    "Could not delete OpenCloud write share: {}",
                    Report::from_error(e)
                );
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
            } else {
                debug!(
                    log: logger,
                    "Deleted write share {:?} from OpenCloud", reference.permission_id
                );
            }
        }

        let Some(drive_id) = write_reference
            .or(read_reference)
            .map(|reference| reference.drive_id)
        else {
            continue;
        };
        match client.delete_folder(&drive_id, path).await {
            Ok(()) => {
                debug!(
                    log: logger,
                    "Deleted shared folder {path:?} from OpenCloud"
                );
            }
            Err(opentalk_opencloud_client::Error::FolderNotFound { path, .. }) => {
                debug!(
                    log: logger,
                    "Path {path:?} not found on the OpenCloud instance, skipping deletion"
                );
            }
            Err(e) => {
                let message = format!(
                    "Error deleting folder on OpenCloud: {}",
                    Report::from_error(e)
                );
                if fail_on_error {
                    whatever!("{message}");
                }
                warn!(log: logger, "{}", message);
            }
        }
    }
    Ok(())
}

async fn delete_opencloud_link(
    client: &opentalk_opencloud_client::Client,
    reference: &ShareReference,
) -> Result<(), opentalk_opencloud_client::Error> {
    match client
        .delete_link(
            &reference.drive_id,
            &reference.item_id,
            &reference.permission_id,
        )
        .await
    {
        Ok(()) | Err(opentalk_opencloud_client::Error::PermissionNotFound { .. }) => Ok(()),
        Err(e) => Err(e),
    }
}
