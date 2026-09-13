// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_asset_storage::ObjectStorageError;
use opentalk_controller_api_authorization::authorization::AuthorizationError;
use opentalk_types_api_common::error::ApiError;
use snafu::Snafu;

use crate::{CaptureApiError, deletion::StopRoomError};

/// Errors returned when deleting an event
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Inventory error
    #[snafu(display("Inventory error: {source}"), context(false))]
    Inventory {
        /// the cause of the error
        source: opentalk_inventory::Error,
    },

    /// Authorization error
    #[snafu(display("Authorization error"))]
    Authorization {
        /// the cause of the error
        source: AuthorizationError,
    },

    /// Forbidden action by user
    #[snafu(display("Tried to perform an action that is forbidden for the user"))]
    Forbidden,

    /// Conflict error
    #[snafu(display(
        "Conflict error: The requested operation could not be completed due to a conflict: {message}"
    ))]
    Conflict {
        /// Error message
        message: String,
    },

    /// Object deletion error
    #[snafu(display("Object deletion error: {source}"))]
    ObjectDeletion {
        /// the cause of the error
        source: ObjectStorageError,
    },

    /// Shared folders not configured
    #[snafu(display("Shared folders not configured"))]
    SharedFoldersNotConfigured,

    /// Nextcloud client error
    #[snafu(display("Nextcloud client error: {source}"), context(false))]
    NextcloudClient {
        /// the cause of the error
        source: opentalk_nextcloud_client::Error,
    },

    /// OpenCloud client error
    #[snafu(display("OpenCloud client error: {source}"), context(false))]
    OpencloudClient {
        /// the cause of the error
        source: opentalk_opencloud_client::Error,
    },

    /// Room deletion error
    #[snafu(display("Room deletion error: {source}"))]
    RoomDelete {
        /// the cause of the error
        source: StopRoomError,
    },

    /// Race condition during database commit preparation detected
    #[snafu(display("Race condition detected during database commit preparation"))]
    RaceCondition,

    /// Custom error
    #[snafu(display("{message}: "), whatever)]
    Custom {
        /// Error message
        message: String,
        /// the cause of the error
        #[snafu(source(from(Box<dyn std::error::Error + Sync + Send>, Some)))]
        source: Option<Box<dyn std::error::Error + Sync + Send>>,
    },
}

impl From<Error> for CaptureApiError {
    fn from(value: Error) -> Self {
        match value {
            Error::Inventory { source } => source.into(),
            Error::Authorization { source } => {
                log::error!("Permission system error: {source}");
                ApiError::internal()
                    .with_message("Permission system error")
                    .into()
            }
            Error::Forbidden => ApiError::forbidden().into(),
            Error::Conflict { message } => ApiError::conflict().with_message(message).into(),
            Error::ObjectDeletion { source } => {
                log::error!("REST API threw internal error from object storage: {source}");
                ApiError::internal().into()
            }
            Error::SharedFoldersNotConfigured => ApiError::bad_request()
                .with_message("No shared folder configured for this server")
                .into(),
            Error::NextcloudClient { .. } => ApiError::internal()
                .with_message("Error performing actions on the NextCloud")
                .into(),
            Error::OpencloudClient { .. } => ApiError::internal()
                .with_message("Error performing actions on the OpenCloud")
                .into(),
            Error::RoomDelete { .. } => ApiError::internal()
                .with_message("Failed to delete roomserver room")
                .into(),
            Error::RaceCondition => {
                log::error!("Race condition detected during deletion");
                ApiError::internal().into()
            }
            Error::Custom { message, source: _ } => {
                ApiError::internal().with_message(message).into()
            }
        }
    }
}
