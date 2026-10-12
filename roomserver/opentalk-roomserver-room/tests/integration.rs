// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    assert_matches,
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
    time::Duration,
};

use opentalk_roomserver_room::mocking::{
    mock_module::{MockCommand, MockModule},
    participant::{MockParticipantBuilder, MockParticipantJoined, bob_public_user_profile},
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    core::{CORE_MODULE_ID, CoreCommand, CoreError, CoreEvent, RoomCloseReason, state::CoreState},
    error::SignalingError,
    rate_limit::RateLimitSettings,
    room_parameters::WaitingRoom,
    signaling::websocket::CloseFrame,
    tariff_details::TariffDetails,
};
use opentalk_types_common::{
    call_in::{CallInId, CallInInfo, CallInPassword},
    roomserver::DeviceSecret,
    tariffs::{QuotaType, TariffId},
    users::DisplayName,
    utils::ExampleData,
};
use serde_json::json;

const ROOM_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[test_log::test(tokio::test)]
async fn response_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // When no transaction id is sent, the response will not contain one
    alice
        .send_command::<MockModule>(MockCommand::Valid, None)
        .await
        .unwrap();

    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, None);

    // When a transaction id is sent, the response will contain the same one
    alice
        .send_command::<MockModule>(MockCommand::Valid, Some(0))
        .await
        .unwrap();
    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test)]
async fn error_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MockModule>(MockCommand::Invalid, Some(0))
        .await
        .unwrap();
    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test)]
async fn invalid_command_response_contains_transaction_id() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    let command = json!({
        "transaction_id": 0,
        "payload": {
            "invalid": "command"
        }
    });
    alice.send_command_raw(command).await.unwrap();

    let event = alice.receive::<SignalingError>().await.unwrap();
    assert_eq!(event.transaction_id, Some(0));
}

#[test_log::test(tokio::test(start_paused = true))]
async fn room_task_time_limit() {
    // Create a room with a tariff that has a time limit of 10 minutes. Do not use a limit of 0
    // seconds because that would create a race condition where Bob might be unable to join the
    // lobby before the room times out.
    let mut room = TestRoom::builder()
        .tariff(TariffDetails {
            id: TariffId::generate(),
            name: "10 minutes room".into(),
            quotas: BTreeMap::from_iter([(QuotaType::RoomTimeLimitSecs, 10 * 60)]),
            used_quota: BTreeMap::new(),
            disabled_features: BTreeSet::new(),
        })
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.lobby_bob(0).await;

    // Advance time by 10 minutes to trigger the room time limit
    tokio::time::advance(Duration::from_mins(10)).await;

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::Closing {
            reason: RoomCloseReason::TimeLimitReached
        }
    );

    let close_frame = alice.receive_close_frame().await.unwrap();
    assert_eq!(
        close_frame,
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Bob receives the same closing event and close frame in the lobby too
    let event = bob.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::Closing {
            reason: RoomCloseReason::TimeLimitReached
        }
    );

    let close_frame = bob.receive_close_frame().await.unwrap();
    assert_eq!(
        close_frame,
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string()
        })
    );
}

#[test_log::test(tokio::test)]
async fn room_closes_when_last_lobby_participant_leaves() {
    let mut room = TestRoom::builder()
        .room_idle_timeout(ROOM_IDLE_TIMEOUT)
        .spawn();

    // The participant never enters the conference, it only connects to the lobby.
    let participant = room.alice_moderator_lobby(0).await;
    participant.disconnect().await.unwrap();

    tokio::time::timeout(ROOM_IDLE_TIMEOUT * 4, room.join_handle)
        .await
        .expect("room should close after the last lobby participant left")
        .unwrap();
}

#[test_log::test(tokio::test)]
async fn room_closes_when_last_conference_participant_leaves() {
    let mut room = TestRoom::builder()
        .room_idle_timeout(ROOM_IDLE_TIMEOUT)
        .spawn();

    let participant = room.join_alice_moderator(0).await;
    participant.disconnect().await.unwrap();

    tokio::time::timeout(ROOM_IDLE_TIMEOUT * 4, room.join_handle)
        .await
        .expect("room should close after the last participant left")
        .unwrap();
}

#[test_log::test(tokio::test)]
async fn participant_receives_same_display_name() {
    let mut room = TestRoom::builder().spawn();
    let bob_0 = room.join_bob(0).await;

    // Bob tries to join with a different display name
    let join_success = MockParticipantBuilder::new(
        bob_public_user_profile(),
        DeviceSecret::from_str("New device secret").unwrap(),
        Role::User,
    )
    .display_name("Bobby".into())
    .join(&mut room)
    .await
    .unwrap();

    // But he receives the same display name as before
    assert_eq!(join_success.display_name(), bob_0.display_name());
}

#[test_log::test(tokio::test)]
async fn exceeding_rate_limit_sends_slow_down() {
    let mut room = TestRoom::builder()
        .ws_rate_limit(RateLimitSettings {
            tokens_per_second: 1,
            token_bucket_size: 2,
        })
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    // Second command should pass. This is the second command because `CoreCommand::EnterRoom` is
    // sent in `join_alice_moderator`.
    // Using CoreCommand because it is available in this module, the actual command does not matter.
    alice
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom));

    // Third command should trigger slow down
    alice
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();

    // Alice receives the slow down event first
    let event = alice.receive::<SignalingError>().await.unwrap().payload;
    assert_eq!(event, SignalingError::SlowDown);

    // Alice receives the actual response to her command afterwards
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom));
}

#[test_log::test(tokio::test)]
async fn moderator_enter_waiting_room_already_accepted() {
    let mut room = TestRoom::builder().spawn();

    // Alice joins the lobby.
    // Moderators are accepted automatically, so they never need to wait.
    let mut alice = room.alice_moderator_lobby(0).await;

    // Trying to enter the waiting room is rejected because she is already accepted.
    alice
        .send_core_command(
            CoreCommand::EnterWaitingRoom {
                display_name: Some(DisplayName::from_str_lossy("Alice")),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::AlreadyAccepted));
}

#[test_log::test(tokio::test)]
async fn signaling_error_only_reaches_originating_connection() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
    let mut alice_device_0 = room.join_alice_moderator(0).await;
    let mut alice_device_1 = room.join_alice_moderator(1).await;
    flush_connected_events(&mut [&mut alice_device_0]).await;

    // Alice sends a payload that cannot be deserialized into MockCommand from device 0
    alice_device_0
        .send_command_raw(json!({ "namespace": "mock", "payload": { "invalid": "payload" } }))
        .await
        .unwrap();

    // Alice receives the corresponding error on device 0
    let event = alice_device_0
        .receive::<SignalingError>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, SignalingError::InvalidJson { message: _ });

    // Alice does not receive an error on device 1
    assert!(
        alice_device_1.received_nothing(),
        "the error must not be broadcast to the participant's other connections",
    );
}

#[test_log::test(tokio::test)]
async fn entering_lobby_does_not_notify_moderators() {
    let mut room = TestRoom::builder().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let _bob = room.lobby_bob(0).await;

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn joined_lobby_does_not_contain_display_name_for_first_connection() {
    let mut room = TestRoom::builder().spawn();
    let alice = room.alice_moderator_lobby(0).await;

    assert_eq!(alice.display_name(), None);
}

#[test_log::test(tokio::test)]
async fn joined_lobby_contains_display_name_for_second_connection() {
    let mut room = TestRoom::builder().spawn();
    let alice_device_0 = room.join_alice_moderator(0).await;
    let alice_device_1 = room.alice_moderator_lobby(1).await;

    assert_eq!(
        alice_device_1
            .display_name()
            .expect("Display name must be present"),
        alice_device_0.display_name()
    )
}

/// Joins the lobby with the given client kind and sends an `EnterRoom` command with the provided
/// display name, returning the resulting [`CoreEvent`] (either a `JoinSuccess` or an `Error`).
async fn enter_room_with_display_name(
    room: &mut TestRoom,
    kind: ClientKind,
    role: Role,
    display_name: Option<DisplayName>,
) -> CoreEvent {
    let mut participant = room
        .enter_lobby(ClientParameters {
            device_secret: DeviceSecret::from_str("Display Name Test Device Secret")
                .expect("Hardcoded device secret must be valid"),
            kind,
            role,
        })
        .await
        .expect("Failed to join lobby");
    participant
        .send_core_command(CoreCommand::EnterRoom { display_name }, None)
        .await
        .expect("Failed to send EnterRoom command");
    participant
        .receive::<CoreEvent>()
        .await
        .expect("Failed to receive event")
        .payload
}

#[test_log::test(tokio::test)]
async fn guest_must_provide_display_name_to_enter_room() {
    let mut room = TestRoom::builder().spawn();
    let event = enter_room_with_display_name(&mut room, ClientKind::Guest, Role::User, None).await;
    assert_matches!(event, CoreEvent::Error(CoreError::DisplayNameRequired));
}

#[test_log::test(tokio::test)]
async fn guest_can_provide_display_name() {
    let mut room = TestRoom::builder().spawn();
    let event = enter_room_with_display_name(
        &mut room,
        ClientKind::Guest,
        Role::User,
        Some(DisplayName::from_str_lossy("Gustav")),
    )
    .await;
    assert_matches!(
        event,
        CoreEvent::JoinSuccess(success)
            if success.display_name == DisplayName::from_str_lossy("Gustav")
    );
}

#[test_log::test(tokio::test)]
async fn guest_must_provide_display_name_to_enter_waiting_room() {
    let mut room = TestRoom::builder()
        .waiting_room(WaitingRoom::ForGuests)
        .spawn();
    let mut gustav = room.lobby_gustav_guest().await;

    gustav
        .send_core_command(CoreCommand::EnterWaitingRoom { display_name: None }, None)
        .await
        .expect("Failed to send EnterWaitingRoom command");
    let event = gustav.receive::<CoreEvent>().await.unwrap().payload;

    assert_matches!(event, CoreEvent::Error(CoreError::DisplayNameRequired));
}

/// Reads the next [`CoreEvent::ParticipantConnected`] event received by `moderator` and returns
/// the [`CoreState`] carried in its `peer_data` for the core module.
async fn receive_participant_connected_core_state(
    moderator: &mut MockParticipantJoined,
) -> CoreState {
    let event = moderator.receive::<CoreEvent>().await.unwrap().payload;
    let CoreEvent::ParticipantConnected { peer_data, .. } = event else {
        panic!("expected ParticipantConnected, got {event:?}");
    };
    let core = peer_data
        .get(&CORE_MODULE_ID)
        .expect("peer_data must contain the core module state");
    serde_json::from_value(core.clone_inner()).expect("peer_data.core must be a valid CoreState")
}

#[test_log::test(tokio::test)]
async fn call_in_phone_number_is_masked_by_default() {
    let mut room = TestRoom::builder().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = enter_room_with_display_name(
        &mut room,
        ClientKind::CallIn,
        Role::User,
        Some(DisplayName::from_str_lossy("+491234567890")),
    )
    .await;

    assert_matches!(
        event,
        CoreEvent::JoinSuccess(success)
            if success.display_name == DisplayName::from_str_lossy("+491***890")
    );

    let core_state = receive_participant_connected_core_state(&mut alice).await;
    assert_eq!(
        core_state.display_name,
        DisplayName::from_str_lossy("+491***890")
    );
}

#[test_log::test(tokio::test)]
async fn registered_call_in_phone_number_is_masked_by_default() {
    let mut room = TestRoom::builder().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = enter_room_with_display_name(
        &mut room,
        ClientKind::RegisteredCallIn {
            profile: bob_public_user_profile(),
        },
        Role::User,
        Some(DisplayName::from_str_lossy("+491234567890")),
    )
    .await;

    assert_matches!(
        event,
        CoreEvent::JoinSuccess(success)
            if success.display_name == DisplayName::from_str_lossy("+491***890")
    );

    let core_state = receive_participant_connected_core_state(&mut alice).await;
    assert_eq!(
        core_state.display_name,
        DisplayName::from_str_lossy("+491***890")
    );
}

#[test_log::test(tokio::test)]
async fn call_in_phone_number_is_not_masked_when_disabled() {
    let mut room = TestRoom::builder()
        .call_in(CallInInfo {
            tel: String::new(),
            id: CallInId::example_data(),
            password: CallInPassword::example_data(),
            mask_unmapped_phone_numbers: false,
            default_country_code: "DE".to_string(),
        })
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = enter_room_with_display_name(
        &mut room,
        ClientKind::CallIn,
        Role::User,
        Some(DisplayName::from_str_lossy("+491234567890")),
    )
    .await;

    assert_matches!(
        event,
        CoreEvent::JoinSuccess(success)
            if success.display_name == DisplayName::from_str_lossy("+491234567890")
    );

    let core_state = receive_participant_connected_core_state(&mut alice).await;
    assert_eq!(
        core_state.display_name,
        DisplayName::from_str_lossy("+491234567890")
    );
}

#[test_log::test(tokio::test)]
async fn registered_call_in_phone_number_is_not_masked_when_disabled() {
    let mut room = TestRoom::builder()
        .call_in(CallInInfo {
            tel: String::new(),
            id: CallInId::example_data(),
            password: CallInPassword::example_data(),
            mask_unmapped_phone_numbers: false,
            default_country_code: "DE".to_string(),
        })
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = enter_room_with_display_name(
        &mut room,
        ClientKind::RegisteredCallIn {
            profile: bob_public_user_profile(),
        },
        Role::User,
        Some(DisplayName::from_str_lossy("+491234567890")),
    )
    .await;

    assert_matches!(
        event,
        CoreEvent::JoinSuccess(success)
            if success.display_name == DisplayName::from_str_lossy("+491234567890")
    );

    let core_state = receive_participant_connected_core_state(&mut alice).await;
    assert_eq!(
        core_state.display_name,
        DisplayName::from_str_lossy("+491234567890")
    );
}
