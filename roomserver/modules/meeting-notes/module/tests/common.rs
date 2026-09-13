// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_meeting_notes::MeetingNotesModule;
use opentalk_roomserver_room::mocking::room::TestRoom;
use opentalk_roomserver_types_meeting_notes::MeetingNotesSettings;
use opentalk_types_common::rooms::RoomId;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner,
};
use url::Url;

pub const ETHERPAD_PORT: u16 = 9001;
pub const ETHERPAD_API_KEY: &str = "secret123";

const ENV_ETHERPAD_API_KEY: &str = "EP_APIKEY";

pub async fn build_etherpad_room() -> (ContainerAsync<GenericImage>, TestRoom) {
    // The etherpad container is very slow to shut down. This causes tests to fail when running
    // multiple test serial or in parallel. To avoid this, we use a random port for each test so
    // that multiple containers can run at the same time.
    let etherpad_container = GenericImage::new("registry.opencode.de/opentalk/etherpad", "v2.0.2")
        .with_wait_for(WaitFor::message_on_stdout("Etherpad is running"))
        .with_mapped_port(0, ETHERPAD_PORT.tcp())
        .with_env_var(ENV_ETHERPAD_API_KEY, ETHERPAD_API_KEY)
        .with_network("bridge")
        .with_log_consumer(LoggingConsumer::new())
        .start()
        .await
        .unwrap();

    let host = etherpad_container.get_host().await.unwrap();
    let port = etherpad_container
        .get_host_port_ipv4(ETHERPAD_PORT)
        .await
        .unwrap();

    let room = build_room(&host.to_string(), port, ETHERPAD_API_KEY.into());

    (etherpad_container, room)
}

fn build_room(host: &str, port: u16, api_key: String) -> TestRoom {
    let base_url = Url::parse(&format!("http://{host}:{port}")).unwrap();
    // In the gitlab ci the etherpad container is reused for all tests. Use a random room id to
    // ensure each test starts with a clean state.
    let room_id = RoomId::generate();
    TestRoom::builder()
        .room_id(room_id)
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings { base_url, api_key })
        .unwrap()
        .spawn()
}
