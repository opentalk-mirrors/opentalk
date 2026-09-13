// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_echo::EchoModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_echo::{command::EchoCommand, event::EchoEvent};
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn ping_sends_response_to_all_connections() {
    let mut room = TestRoom::builder().register_module::<EchoModule>().spawn();

    let mut alice_1 = room.join_alice_moderator(1).await;
    let mut alice_2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice_1]).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice_1, &mut alice_2]).await;

    alice_1
        .send_command::<EchoModule>(EchoCommand::Ping, None)
        .await
        .unwrap();

    assert_eq!(
        alice_1.receive_event::<EchoModule>().await.unwrap().payload,
        EchoEvent::Pong
    );
    assert_eq!(
        alice_2.receive_event::<EchoModule>().await.unwrap().payload,
        EchoEvent::Pong
    );
    assert!(bob.received_nothing());
}
