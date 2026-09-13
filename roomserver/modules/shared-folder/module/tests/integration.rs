// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_shared_folder::SharedFolderModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::room_parameters::EventContext;
use opentalk_roomserver_types_shared_folder::state::{SharedFolderAccess, SharedFolderState};
use opentalk_types_common::events::EventId;
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn join_info() {
    let admin_state = SharedFolderState {
        read: SharedFolderAccess {
            url: "http://opencloud.folder.example.com".to_string(),
            password: "pass".to_string(),
        },
        read_write: Some(SharedFolderAccess {
            url: "http://admin.opencloud.folder.example.com".to_string(),
            password: "admin.pass".to_string(),
        }),
    };
    let mut room = TestRoom::builder()
        .register_module::<SharedFolderModule>()
        .event(EventContext {
            id: EventId::from_u128(1),
            title: "Event 1".parse().unwrap(),
            description: "This is the first event in the shared folder test"
                .parse()
                .unwrap(),
            is_adhoc: false,
            starts_at: None,
            ends_at: None,
            shared_folder: Some(admin_state.clone().into()),
        })
        .spawn();

    let ronly_state = admin_state.clone().without_write_access();

    let mut alice = room.join_alice_moderator(1).await;
    let sf_state = alice
        .join_success()
        .module_data
        .get::<SharedFolderState>()
        .expect("SharedFolder state must be valid")
        .expect("SharedFolder state must be present");
    assert_eq!(sf_state, admin_state);

    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let sf_state = bob
        .join_success()
        .module_data
        .get::<SharedFolderState>()
        .expect("SharedFolder state must be valid")
        .expect("SharedFolder state must be present");
    assert_eq!(sf_state, ronly_state);
}
