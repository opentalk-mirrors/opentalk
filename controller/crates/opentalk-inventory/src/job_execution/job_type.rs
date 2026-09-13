// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Maintenance job types that can be executed by OpenTalk
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::AsRefStr,
    strum::Display,
    strum::EnumCount,
    strum::EnumIter,
    strum::EnumString,
    strum::VariantNames,
    strum::IntoStaticStr,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// A simple self-check of the job execution system
    SelfCheck,

    /// A job for cleaning up events that ended at minimum a defined duration ago
    EventCleanup,

    /// A job for cleaning up users that were disabled at minimum a defined duration ago
    UserCleanup,

    /// A job to cleanup adhoc events a certain duration after they were created
    AdhocEventCleanup,

    /// A job for cleaning up expired invites
    InviteCleanup,

    /// A job to synchronize database assets and storage files
    SyncStorageFiles,

    /// A job to remove all rooms that have no event associated with them
    RoomCleanup,

    /// A job to synchronize the user account states with Keycloak
    KeycloakAccountSync,
}
