// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

const REDIS_PORT: u16 = 6379;
const REDIS_VERSION: &str = "7.2.4";

pub(crate) struct TestContainer {
    pub(crate) _container: ContainerAsync<GenericImage>,
    pub(crate) url: String,
}

impl TestContainer {
    /// Create a new single instance redis container
    pub(crate) async fn new_redis_instance() -> Self {
        let redis_container = GenericImage::new("redis", REDIS_VERSION)
            .with_exposed_port(REDIS_PORT.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .with_network("bridge")
            .with_env_var("DEBUG", "1")
            .start()
            .await
            .expect("Failed to start Redis test container");

        let host = redis_container
            .get_host()
            .await
            .expect("Failed to get Redis container host")
            .to_string();

        let host_port = redis_container
            .get_host_port_ipv4(REDIS_PORT)
            .await
            .unwrap();

        let redis_address = format!("redis://{}:{}", host, host_port);

        Self {
            _container: redis_container,
            url: redis_address.clone(),
        }
    }
}
