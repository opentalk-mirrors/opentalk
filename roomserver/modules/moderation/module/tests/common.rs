// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::TestRoom;
use opentalk_roomserver_test_util_livekit as test_util;
use opentalk_types_common::rooms::RoomId;

pub async fn build_room() -> (test_util::ContainerGuard, TestRoom, String) {
    let (container, settings) = test_util::create_livekit_container().await;

    let room = TestRoom::builder()
        .room_id(RoomId::generate())
        .register_module::<LiveKitModule>()
        .register_module::<ModerationModule>()
        .add_init_module_data(&settings)
        .unwrap()
        .spawn();

    (container, room, settings.public_url)
}
