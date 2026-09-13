// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory::{self as inventory, SharedFolderProvider};
use opentalk_types_common::events::EventId;

use crate::{schema::event_shared_folders, tables::events::Event, utils::Jsonb};

#[derive(Clone, Debug, PartialEq, Eq, Associations, Identifiable, Queryable)]
#[diesel(table_name = event_shared_folders)]
#[diesel(primary_key(event_id))]
#[diesel(belongs_to(Event))]
pub struct EventSharedFolder {
    pub event_id: EventId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub path: String,
    pub write_url: String,
    pub write_password: String,
    pub read_url: String,
    pub read_password: String,
    /// The provider-specific reference data, stored as self-describing
    /// internally-tagged JSON.
    pub provider_data: Jsonb<SharedFolderProvider>,
}

impl From<EventSharedFolder> for inventory::EventSharedFolder {
    fn from(
        EventSharedFolder {
            event_id,
            created_at,
            updated_at,
            path,
            write_url,
            write_password,
            read_url,
            read_password,
            provider_data,
        }: EventSharedFolder,
    ) -> Self {
        Self {
            event_id,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            path,
            write_url,
            write_password,
            read_url,
            read_password,
            provider: provider_data.0,
        }
    }
}

impl From<inventory::EventSharedFolder> for EventSharedFolder {
    fn from(
        inventory::EventSharedFolder {
            event_id,
            created_at,
            updated_at,
            path,
            write_url,
            write_password,
            read_url,
            read_password,
            provider,
        }: inventory::EventSharedFolder,
    ) -> Self {
        Self {
            event_id,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            path,
            write_url,
            write_password,
            read_url,
            read_password,
            provider_data: Jsonb(provider),
        }
    }
}
