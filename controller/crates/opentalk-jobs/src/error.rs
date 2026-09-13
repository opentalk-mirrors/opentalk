// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::num::TryFromIntError;

use opentalk_asset_storage::ObjectStorageError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum EtcdError {
    #[snafu(display("Failed to connect to etcd"))]
    ConnectError { source: etcd_client::Error },

    #[snafu(display("Failed to parse value for key `{key}`"))]
    ParseError {
        key: String,
        source: etcd_client::Error,
    },

    #[snafu(display("Failed to add key `{key}`"))]
    Add {
        key: String,
        source: etcd_client::Error,
    },

    #[snafu(display("Failed to get key(s) `{key}`"))]
    Get {
        key: String,
        source: etcd_client::Error,
    },

    #[snafu(display("Failed to remove key(s) `{key}`"))]
    Remove {
        key: String,
        source: etcd_client::Error,
    },

    #[snafu(display("Failed to create a lease"))]
    Lease { source: etcd_client::Error },

    #[snafu(display("Failed to create keep alive stream"))]
    CreateKeepAlive { source: etcd_client::Error },

    #[snafu(display("Failed to keep alive"))]
    KeepAlive { source: etcd_client::Error },

    #[snafu(display("Failed to create watch stream"))]
    CreateWatch { source: etcd_client::Error },

    #[snafu(display("Failed to request progress for the watch stream"))]
    WatchProgress { source: etcd_client::Error },

    #[snafu(display("The watch stream threw an error"))]
    WatchStream { source: etcd_client::Error },

    #[snafu(display("The watch stream was closed by etcd"))]
    WatchClosed,
}

/// Errors that can occur during job execution
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    /// Could not load job parameters
    ParameterLoading {
        /// The error source
        source: serde_json::Error,
    },

    /// Could not serialize job parameters
    ParameterSerializing {
        /// The error source
        source: serde_json::Error,
    },

    /// Invalid value for parameter {parameter_name:?}. Required: {expected_requirement}
    InvalidParameterValue {
        /// The name of the parameter
        parameter_name: String,

        /// The expected requirement which was not fulfilled
        expected_requirement: String,
    },

    /// Job execution timed out
    #[snafu(context(false))]
    Timeout {
        /// The error source
        source: tokio::time::error::Elapsed,
    },

    /// Inventory error
    #[snafu(context(false))]
    Inventory {
        /// The error source
        source: opentalk_inventory::Error,
    },

    /// Job execution failed
    JobExecutionFailed,

    /// Permission system error
    #[snafu()]
    Permissions {},

    /// Found shared folders in the database, but the configuration file contains no shared folder settings
    SharedFoldersNotConfigured,

    /// Error communicating with NextCloud instance: {source}
    #[snafu(context(false))]
    NextcloudClient {
        /// The error source
        source: opentalk_nextcloud_client::Error,
    },

    /// Error performing changes in the object storage
    #[snafu(context(false))]
    ObjectStorage {
        /// The error source
        source: ObjectStorageError,
    },

    /// Event deletion failed
    EventDeletionFailed,

    /// File error
    #[snafu(context(false))]
    FileError { source: std::io::Error },

    /// User search backend is not Keycloak
    UserSearchBackendIsNotKeycloak,

    /// Error communicating with Keycloak instance
    #[snafu(context(false))]
    KeycloakClient {
        source: opentalk_keycloak_admin::Error,
    },

    /// Received invalid user count {returned_user_count} from Keycloak API.
    KeycloakApiReturnedInvalidUserCount {
        returned_user_count: i32,
        source: TryFromIntError,
    },

    /// Invalid settings
    #[snafu(context(false))]
    Settings {
        source: opentalk_controller_settings::SettingsError,
    },
}
