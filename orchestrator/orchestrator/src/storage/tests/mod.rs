// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, time::Duration};

use opentalk_orchestrator_shared::{
    Metrics, RecorderResource, RegisterRecorder, RegisterRoomserver, RegisterTranscription,
    RegisterType, ServiceKind, ServiceResource, TranscriptionResource, error::RegistrationError,
};
use opentalk_service_auth::ApiKeyId;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use url::Url;

use crate::{
    service_instance::registration::ServiceRegistration,
    storage::{
        self, AddInstanceError, OrchestratorStorage,
        local::LocalStorage,
        redis::{RedisStorage, keys},
        tests::container::TestContainer,
    },
    tasks::Tasks,
};

pub(crate) mod container;

const ROOM_ZERO: RoomId = RoomId::from_u128(0);
const ROOM_ONE: RoomId = RoomId::from_u128(1);
const TEST_ROOMSERVER_TOKEN_EXPIRY: Duration = Duration::from_secs(1);

/// A struct to test common behaviour of all orchestrator storage implementations
pub(crate) struct StorageTester<T: OrchestratorStorage> {
    /// Optional containers for storage implementations that depend on an external service (e.g.
    /// redis)
    _containers: Vec<TestContainer>,
    /// The storage
    storage: T,
}

impl StorageTester<LocalStorage> {
    pub(crate) async fn new_local_storage() -> Self {
        let mut local_storage = LocalStorage::new();
        local_storage
            .set_roomserver_token_expiry(TEST_ROOMSERVER_TOKEN_EXPIRY)
            .await;

        Self {
            _containers: vec![],
            storage: local_storage,
        }
    }
}

impl StorageTester<RedisStorage> {
    pub(crate) async fn new_redis_storage() -> Self {
        let container = TestContainer::new_redis_instance().await;

        let mut tasks = Tasks::new();
        let mut storage = RedisStorage::init(&mut tasks, &container.url)
            .await
            .expect("Failed to create Redis storage");

        storage
            .set_roomserver_token_expiry(TEST_ROOMSERVER_TOKEN_EXPIRY)
            .await;

        Self {
            _containers: vec![container],
            storage,
        }
    }

    pub(crate) async fn cleanup(&self) {
        let client = &self.storage.client;
        let orchestrator_id = self.storage.orchestrator_id;
        let mut con = client.get_multiplexed_async_connection().await.unwrap();

        // Manually delete the alive key to simulate the orchestrator being dead, so that the
        // cleanup script can run
        redis::cmd("DEL")
            .arg(keys::OrchestratorAliveKey {
                id: orchestrator_id.to_string(),
            })
            .query_async::<()>(&mut con)
            .await
            .unwrap();

        let cleaned_up_orchestrators = storage::redis::scripts::cleanup(client, &[orchestrator_id])
            .await
            .unwrap();

        if cleaned_up_orchestrators.is_empty() {
            panic!("Expected orchestrator to be cleaned up");
        }

        self.storage.assert_empty().await.unwrap();
    }
}

impl<T: OrchestratorStorage> StorageTester<T> {
    /// Helper function to reduce boilerplate in other tests
    ///
    /// Adds a new instance with the given `RegisterType` and some hardcoded values
    async fn add_instance(
        &self,
        url: &Url,
        register_type: RegisterType,
    ) -> Result<(), AddInstanceError> {
        let storage = &self.storage;

        let kind = match register_type {
            RegisterType::Roomserver(_) => ServiceKind::Roomserver,
            RegisterType::Recorder(_) => ServiceKind::Recorder,
            RegisterType::Transcription(_) => ServiceKind::Transcription,
        };

        let metrics = Metrics {
            load: 5,
            accepting_jobs: true,
        };

        let api_key_ids = vec![ApiKeyId::from("key1"), ApiKeyId::from("key2")];

        let registration = ServiceRegistration {
            address: url.clone(),
            register_type,
            api_key_ids: api_key_ids.clone(),
            metrics,
        };

        storage.add_instance(registration).await?;

        let instance_data = storage.get_instance(url, kind).await.unwrap().unwrap();

        assert_eq!(instance_data.metrics, metrics);
        assert_eq!(instance_data.api_key_ids, api_key_ids);

        Ok(())
    }

    pub(crate) async fn add_roomserver_instance(&self, url: &str) {
        let url = url.parse().unwrap();

        let register_type = RegisterType::Roomserver(RegisterRoomserver {
            rooms: [ROOM_ZERO, ROOM_ONE].into(),
        });

        self.add_instance(&url, register_type).await.unwrap();

        let Some((instance_url, _instance_data)) = self
            .storage
            .get_instances_for_resource(&ROOM_ZERO.into())
            .await
            .unwrap()
        else {
            panic!("expected resource to exist")
        };

        assert_eq!(instance_url, url)
    }

    pub(crate) async fn add_recorder_instance(&self, url: &str) {
        let url = url.parse().unwrap();
        let resource_0 = RecorderResource {
            room_id: ROOM_ZERO,
            breakout_id: None,
        };

        let resource_1 = RecorderResource {
            room_id: ROOM_ONE,
            breakout_id: Some(1),
        };

        let register_type = RegisterType::Recorder(RegisterRecorder {
            rooms: [resource_0, resource_1].into(),
        });

        self.add_instance(&url, register_type).await.unwrap();

        let Some((instance_url, _instance_data)) = self
            .storage
            .get_instances_for_resource(&resource_0.into())
            .await
            .unwrap()
        else {
            panic!("expected resource to exist")
        };

        assert_eq!(instance_url, url)
    }

    pub(crate) async fn add_transcription_instance(&self, url: &str) {
        let url = url.parse().unwrap();

        let resource_0 = TranscriptionResource {
            room_id: ROOM_ZERO,
            breakout_id: None,
        };

        let register_type = RegisterType::Transcription(RegisterTranscription {
            rooms: [
                resource_0,
                TranscriptionResource {
                    room_id: ROOM_ONE,
                    breakout_id: None,
                },
            ]
            .into(),
        });

        self.add_instance(&url, register_type).await.unwrap();

        let Some((instance_url, _instance_data)) = self
            .storage
            .get_instances_for_resource(&resource_0.into())
            .await
            .unwrap()
        else {
            panic!("expected resource to exist")
        };

        assert_eq!(instance_url, url)
    }

    pub(crate) async fn add_multiple_services(&self) {
        self.add_roomserver_instance("http://localhost:8080/roomserver/")
            .await;
        self.add_recorder_instance("http://localhost:8080/recorder/")
            .await;
        self.add_transcription_instance("http://localhost:8080/transcription/")
            .await;
    }

    pub(crate) async fn get_unknown_instance(&self) {
        let url: Url = "http://localhost:8080/unknown/".parse().unwrap();
        let kind = ServiceKind::Roomserver;

        let instance_data = self.storage.get_instance(&url, kind).await.unwrap();

        assert!(instance_data.is_none());
    }

    pub(crate) async fn get_wrong_instance_kind(&self) {
        let url = "http://localhost:8080/recorder/";
        self.add_recorder_instance(url).await;
        let kind = ServiceKind::Roomserver;

        let instance_data = self
            .storage
            .get_instance(&(url.parse().unwrap()), kind)
            .await
            .unwrap();

        assert!(instance_data.is_none());
    }

    pub(crate) async fn duplicate_url(&self) {
        let url_str = "http://localhost:8080";
        let url = url_str.parse().unwrap();

        self.add_roomserver_instance(url_str).await;
        let register_type = RegisterType::Roomserver(RegisterRoomserver { rooms: [].into() });

        let err = self.add_instance(&url, register_type).await.unwrap_err();
        assert_matches!(
            err,
            AddInstanceError::RegistrationError(RegistrationError::AddressAlreadyInUse)
        );
    }

    pub(crate) async fn duplicate_resource(&self) {
        let url_one = "http://localhost:8080/roomserver1/";
        let url_two = "http://localhost:8080/roomserver2/".parse().unwrap();

        self.add_roomserver_instance(url_one).await;

        // roomserver two attempts to register with the same resource as roomserver one
        let register_type = RegisterType::Roomserver(RegisterRoomserver {
            rooms: [RoomId::from_u128(42), ROOM_ONE].into(),
        });

        let err = self
            .add_instance(&url_two, register_type)
            .await
            .unwrap_err();

        let conflicting_rooms = [ServiceResource::Roomserver(ROOM_ONE)];

        assert_matches!(
            err,
            AddInstanceError::RegistrationError(RegistrationError::ResourceConflict(c)) if c == conflicting_rooms
        );

        let resources = self.storage.get_all_resources().await.unwrap();
        // only the two resources from the first roomserver instance should exist
        assert_eq!(resources.len(), 2);
    }

    pub(crate) async fn remove_instance(&self) {
        let url_str = "http://localhost:8080";
        let url = url_str.parse().unwrap();

        self.add_roomserver_instance(url_str).await;

        self.storage
            .remove_instance(&url, ServiceKind::Roomserver)
            .await
            .unwrap();

        let resources = self.storage.get_all_resources().await.unwrap();
        assert!(resources.is_empty());

        let services = self.storage.get_all_services().await.unwrap();
        assert!(services.is_empty());
    }

    pub(crate) async fn set_metrics(&self) {
        let url_str = "http://localhost:8080";

        let metrics = Metrics {
            load: 67,
            accepting_jobs: true,
        };

        self.add_roomserver_instance(url_str).await;

        let url = url_str.parse().unwrap();
        self.storage
            .set_service_metrics(&url, ServiceKind::Roomserver, metrics)
            .await
            .unwrap();

        let instance_data = self
            .storage
            .get_instance(&url, ServiceKind::Roomserver)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(instance_data.metrics, metrics);
    }

    pub(crate) async fn select_instance(&self) {
        let url: Url = "http://localhost:8080/roomserver".parse().unwrap();
        let api_key_ids = vec![ApiKeyId::from("key1"), ApiKeyId::from("key2")];
        let metrics = Metrics {
            load: 5,
            accepting_jobs: true,
        };
        let registration = ServiceRegistration {
            address: url.clone(),
            register_type: RegisterType::Roomserver(RegisterRoomserver { rooms: [].into() }),
            api_key_ids: api_key_ids.clone(),
            metrics,
        };
        self.storage.add_instance(registration).await.unwrap();

        let (selected_url, instance_data) = self
            .storage
            .select_instance_for_resource(RoomId::from_u128(123).into())
            .await
            .unwrap();

        assert_eq!(url, selected_url);
        assert_eq!(api_key_ids, instance_data.api_key_ids);
        assert_eq!(metrics, instance_data.metrics);
    }

    pub(crate) async fn select_lowest_load_instance(&self) {
        let url_one = "http://localhost:8080/roomserver1/".parse().unwrap();
        let url_two = "http://localhost:8080/roomserver2/".parse().unwrap();

        let register_type = RegisterType::Roomserver(RegisterRoomserver { rooms: [].into() });
        self.add_instance(&url_one, register_type.clone())
            .await
            .unwrap();
        self.add_instance(&url_two, register_type).await.unwrap();

        let metrics_one = Metrics {
            load: 10,
            accepting_jobs: true,
        };

        let metrics_two = Metrics {
            load: 5,
            accepting_jobs: true,
        };

        self.storage
            .set_service_metrics(&url_one, ServiceKind::Roomserver, metrics_one)
            .await
            .unwrap();

        self.storage
            .set_service_metrics(&url_two, ServiceKind::Roomserver, metrics_two)
            .await
            .unwrap();

        let (selected_url, instance_data) = self
            .storage
            .select_instance_for_resource(RoomId::from_u128(123).into())
            .await
            .unwrap();

        assert_eq!(selected_url, url_two);
        assert_eq!(instance_data.metrics, metrics_two)
    }

    pub(crate) async fn select_instance_resource_count_tiebreak(&self) {
        let url_one = "http://localhost:8080/roomserver1/".parse().unwrap();
        let url_two = "http://localhost:8080/roomserver2/".parse().unwrap();
        let url_three = "http://localhost:8080/roomserver3/".parse().unwrap();

        // all roomservers will have the same load, so the one with the least resources should be
        // selected
        let metrics = Metrics {
            load: 5,
            accepting_jobs: true,
        };

        // Initialize roomserver one with one resource
        self.add_instance(
            &url_one,
            RegisterType::Roomserver(RegisterRoomserver {
                rooms: [ROOM_ZERO].into(),
            }),
        )
        .await
        .unwrap();

        self.storage
            .set_service_metrics(&url_one, ServiceKind::Roomserver, metrics)
            .await
            .unwrap();

        // Initialize roomserver two with one resource
        self.add_instance(
            &url_two,
            RegisterType::Roomserver(RegisterRoomserver {
                rooms: [ROOM_ONE].into(),
            }),
        )
        .await
        .unwrap();

        self.storage
            .set_service_metrics(&url_two, ServiceKind::Roomserver, metrics)
            .await
            .unwrap();

        // Initialize roomserver three without any resources
        self.add_instance(
            &url_three,
            RegisterType::Roomserver(RegisterRoomserver { rooms: [].into() }),
        )
        .await
        .unwrap();

        self.storage
            .set_service_metrics(&url_three, ServiceKind::Roomserver, metrics)
            .await
            .unwrap();

        let (selected_url, instance_data) = self
            .storage
            .select_instance_for_resource(RoomId::from_u128(123).into())
            .await
            .unwrap();

        assert_eq!(selected_url, url_three);
        assert_eq!(instance_data.metrics, metrics)
    }

    pub(crate) async fn lowest_load_not_accepting_jobs(&self) {
        let url_one = "http://localhost:8080/roomserver1/".parse().unwrap();
        let url_two = "http://localhost:8080/roomserver2/".parse().unwrap();

        let register_type = RegisterType::Roomserver(RegisterRoomserver { rooms: [].into() });
        self.add_instance(&url_one, register_type.clone())
            .await
            .unwrap();
        self.add_instance(&url_two, register_type).await.unwrap();

        let metrics_one = Metrics {
            load: 10,
            accepting_jobs: true,
        };

        let metrics_two = Metrics {
            load: 5,
            accepting_jobs: false, /* This instance is not accepting jobs, so it should not be
                                    * selected */
        };

        self.storage
            .set_service_metrics(&url_one, ServiceKind::Roomserver, metrics_one)
            .await
            .unwrap();

        self.storage
            .set_service_metrics(&url_two, ServiceKind::Roomserver, metrics_two)
            .await
            .unwrap();

        let (selected_url, instance_data) = self
            .storage
            .select_instance_for_resource(RoomId::from_u128(123).into())
            .await
            .unwrap();

        assert_eq!(selected_url, url_one);
        assert_eq!(instance_data.metrics, metrics_one)
    }

    pub(crate) async fn roomserver_token_store(&self) {
        let room1 = RoomId::from_u128(1);
        let token1 = Token::generate();
        self.storage
            .add_roomserver_token(room1, token1)
            .await
            .unwrap();

        let room2 = RoomId::from_u128(2);
        let token2 = Token::generate();
        self.storage
            .add_roomserver_token(room2, token2)
            .await
            .unwrap();

        // get the right room for token1
        assert_eq!(
            self.storage
                .consume_roomserver_token(&token1)
                .await
                .unwrap(),
            Some(room1)
        );

        // ensure the token was consumed and does not exist anymore
        assert_eq!(
            None,
            self.storage
                .consume_roomserver_token(&token1)
                .await
                .unwrap()
        );

        tokio::time::sleep(TEST_ROOMSERVER_TOKEN_EXPIRY + Duration::from_millis(100)).await;

        //ensure the second token expired
        assert_eq!(
            None,
            self.storage
                .consume_roomserver_token(&token2)
                .await
                .unwrap()
        );
    }
}
