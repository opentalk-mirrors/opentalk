// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, rooms::RoomId};

use super::{EventSharedFolder, NewEventSharedFolder};
use crate::Result;

/// A trait for retrieving and storing event shared folder entities.
#[async_trait::async_trait]
pub trait EventSharedFolderInventory {
    /// Create a new shared folder.
    async fn try_create_event_shared_folder(
        &mut self,
        new_shared_folder: NewEventSharedFolder,
    ) -> Result<Option<EventSharedFolder>>;

    /// Get the shared folder for an event.
    async fn get_event_shared_folder(
        &mut self,
        event_id: EventId,
    ) -> Result<Option<EventSharedFolder>>;

    /// Get the list of shared folders linked to a room id.
    async fn get_event_shared_folders_for_room(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<EventSharedFolder>>;

    /// Delete shared folder by event id.
    async fn delete_shared_folder_by_event_id(&mut self, event_id: EventId) -> Result<()>;

    /// Delete all shared folders by event ids.
    async fn delete_shared_folders_by_event_ids(&mut self, event_ids: &[EventId]) -> Result<()>;
}
