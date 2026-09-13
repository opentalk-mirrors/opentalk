// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::Display;
use opentalk_inventory as inventory;
use opentalk_types_common::sql_enum;

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    JobType,
    "job_type",
    JobTypeType,
    {
        AdhocEventCleanup = b"adhoc_event_cleanup",
        EventCleanup = b"event_cleanup",
        InviteCleanup = b"invite_cleanup",
        SelfCheck = b"self_check",
        SyncStorageFiles = b"sync_storage_files",
        RoomCleanup = b"room_cleanup",
        KeycloakAccountSync = b"keycloak_account_sync",
        UserCleanup = b"user_cleanup"
    }
);

impl From<JobType> for inventory::JobType {
    fn from(value: JobType) -> Self {
        match value {
            JobType::AdhocEventCleanup => Self::AdhocEventCleanup,
            JobType::EventCleanup => Self::EventCleanup,
            JobType::UserCleanup => Self::UserCleanup,
            JobType::InviteCleanup => Self::InviteCleanup,
            JobType::SelfCheck => Self::SelfCheck,
            JobType::SyncStorageFiles => Self::SyncStorageFiles,
            JobType::RoomCleanup => Self::RoomCleanup,
            JobType::KeycloakAccountSync => Self::KeycloakAccountSync,
        }
    }
}

impl From<inventory::JobType> for JobType {
    fn from(value: inventory::JobType) -> Self {
        use inventory::JobType as Other;
        match value {
            Other::AdhocEventCleanup => Self::AdhocEventCleanup,
            Other::EventCleanup => Self::EventCleanup,
            Other::UserCleanup => Self::UserCleanup,
            Other::InviteCleanup => Self::InviteCleanup,
            Other::SelfCheck => Self::SelfCheck,
            Other::SyncStorageFiles => Self::SyncStorageFiles,
            Other::RoomCleanup => Self::RoomCleanup,
            Other::KeycloakAccountSync => Self::KeycloakAccountSync,
        }
    }
}
