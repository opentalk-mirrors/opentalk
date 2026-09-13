// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Jobs that can be run in the OpenTalk job execution system

mod adhoc_event_cleanup;
mod event_cleanup;
mod invite_cleanup;
mod keycloak_account_sync;
mod room_cleanup;
mod self_check;
mod sync_storage_files;
mod user_cleanup;

pub use adhoc_event_cleanup::AdhocEventCleanup;
pub use event_cleanup::EventCleanup;
pub use invite_cleanup::InviteCleanup;
pub use keycloak_account_sync::KeycloakAccountSync;
pub use room_cleanup::RoomCleanup;
pub use self_check::SelfCheck;
pub use sync_storage_files::SyncStorageFiles;
pub use user_cleanup::UserCleanup;

#[cfg(test)]
mod test_utils {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use opentalk_controller_utils::deletion::{StopRoomBackend, StopRoomError};
    use opentalk_inventory::{Event, Inventory, NewEvent, NewRoom, Room, User};
    use opentalk_types_common::rooms::{GuestAccess, RoomId};

    /// A [`StopRoomBackend`] that records the ids of all rooms it is asked to delete.
    #[derive(Default)]
    pub(super) struct RecordingStopRoomBackend {
        pub(super) deleted_rooms: Mutex<Vec<RoomId>>,
    }

    #[async_trait]
    impl StopRoomBackend for RecordingStopRoomBackend {
        async fn stop_room(&self, room_id: RoomId) -> Result<(), StopRoomError> {
            self.deleted_rooms.lock().unwrap().push(room_id);
            Ok(())
        }
    }

    pub(super) async fn create_generic_test_room(
        inventory: &mut dyn Inventory,
        user: &User,
    ) -> Room {
        inventory
            .create_room(NewRoom {
                created_by: user.id,
                alias: None,
                password: None,
                waiting_room: false,
                guest_access: GuestAccess::default(),
                e2e_encryption: false,
                tenant_id: user.tenant_id,
            })
            .await
            .unwrap()
    }

    pub(super) async fn create_generic_test_event(
        inventory: &mut dyn Inventory,
        user: &User,
        is_adhoc: bool,
    ) -> Event {
        let room = create_generic_test_room(inventory, user).await;

        inventory
            .create_event(NewEvent {
                title: "TestEvent".parse().expect("valid event title"),
                description: "A normal event, created by a test"
                    .parse()
                    .expect("valid event description"),
                room: room.id,
                created_by: user.id,
                updated_by: user.id,
                date: None,
                is_adhoc,
                tenant_id: user.tenant_id,
                show_meeting_details: true,
            })
            .await
            .unwrap()
    }
}
