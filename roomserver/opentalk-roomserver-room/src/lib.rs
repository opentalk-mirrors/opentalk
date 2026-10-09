// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Contains code for room management and signaling.
//!
//! The room state is managed by the [`task::RoomTask`], where each room has its own [`tokio::task`]
//! with an instance of a [`RoomTask`](task::RoomTask). The [`RoomTasks`](task::RoomTask) have a
//! channel interface that is exposed via the [`RoomTaskHandle`] through which the web api can send
//! requests to each individual room.
//!
//! The active rooms are created and tracked with the [`RoomTaskRegistry`]. When a
//! [`task::RoomTask`] gets destroyed, it removes itself from the [`RoomTaskRegistry`].

pub mod message_router;
pub mod metrics;
#[cfg(any(test, feature = "mock"))]
pub mod mocking;
pub mod orchestrator;
pub mod registry;
pub(crate) mod room_map;
pub mod signaling;
pub mod storage;
pub mod task;

pub use opentalk_roomserver_common::{
    application_state::ApplicationState, token_store::TokenStore,
};
pub use opentalk_roomserver_types::signaling::signaling_context::SignalingClientContext;

pub mod settings {
    pub use opentalk_roomserver_common::settings::{Internal, Task};
    pub mod settings_file {
        pub use opentalk_roomserver_common::settings::settings_file::{
            internal::Internal, task::Task,
        };
    }
}

pub use crate::{
    registry::RoomTaskRegistry,
    signaling::module_initializer::ModuleRegistry,
    task::{
        RoomTaskApiError,
        context::RoomTaskContext,
        handle::{Request, RoomTaskHandle, RoomTaskHandleError},
    },
};

#[cfg(test)]
mod tests {
    use std::{
        assert_matches,
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
        time::Duration,
    };

    use futures::StreamExt as _;
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_types::{
        client_parameters::{self, ClientParameters},
        core::{CORE_MODULE_ID, CoreEvent, JoinBlockedReason},
        room_parameters::RoomParameters,
        signaling::websocket::{CloseFrame, SignalingSocketMessage},
        tariff_details::TariffDetails,
    };
    use opentalk_types_api_internal::module_assets::Quota;
    use opentalk_types_common::{
        rooms::RoomId,
        roomserver::DeviceSecret,
        tariffs::{QuotaType, TariffId},
        utils::ExampleData,
    };
    use opentalk_types_signaling::Role;
    use tokio::{sync::watch, time::sleep};

    use super::{signaling::module_initializer::ModuleRegistry, task::handle::RoomTaskHandle};
    use crate::{
        mocking::{participant::create_participant_connection, room::TestRoom, socket::MockSocket},
        storage::{
            memory_asset_storage::MemoryAssetStorage,
            memory_module_storage::MemoryModuleResourceStorage,
        },
        task::{RoomTask, context::RoomTaskContext},
    };

    const TIMEOUT: Duration = Duration::from_millis(500);

    fn create_room_task() -> (RoomTaskHandle<MockSocket>, watch::Sender<ApplicationState>) {
        let id = RoomId::from_u128(0xc270ab35_5cdb_4614_b872_8dd66ceefc70);
        let mut params = RoomParameters::example_data();
        params.room_idle_timeout = TIMEOUT;
        let params = Arc::new(params);
        let module_registry = Arc::new(ModuleRegistry::new());
        let (sender, app_state) = watch::channel(ApplicationState::Running);
        let asset_storage = Arc::new(MemoryAssetStorage::new(Quota {
            total: None,
            used: 0,
        }));
        let module_resources = Arc::new(MemoryModuleResourceStorage::new());
        let ctx = RoomTaskContext {
            module_registry,
            asset_storage,
            module_resources,
            settings: Arc::default(),
            app_state,
        };

        let (task_handle, future_room) = RoomTask::setup(ctx, id, params);
        tokio::spawn(future_room);
        (task_handle, sender)
    }

    #[test_log::test(tokio::test)]
    async fn timeout() {
        let (handle, _sender) = create_room_task();
        sleep(TIMEOUT - Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap();
        sleep(TIMEOUT + Duration::from_millis(100)).await;
        handle.refresh_idle_timeout().await.unwrap_err();
    }

    #[test_log::test(tokio::test)]
    async fn accept_signaling_socket() {
        let (handle, _sender) = create_room_task();
        let (socket, _) = create_participant_connection();
        let client_parameters = ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: client_parameters::ClientKind::Unregistered,
            role: Role::Moderator,
        };

        handle
            .accept_signaling_socket(socket, client_parameters)
            .await
            .unwrap();
    }

    #[test_log::test(tokio::test)]
    async fn set_parameters_updates_room_state() {
        let (handle, _sender) = create_room_task();

        let initial_allowed_origins = handle.allowed_origins().await.unwrap();
        assert_ne!(
            initial_allowed_origins,
            vec!["https://example.invalid".to_owned()]
        );

        let mut updated_parameters = RoomParameters::example_data();
        updated_parameters.allowed_origins = vec!["https://example.invalid".to_owned()];

        handle.set_parameters(updated_parameters).await.unwrap();

        let allowed_origins = handle.allowed_origins().await.unwrap();
        assert_eq!(allowed_origins, vec!["https://example.invalid".to_owned()]);
    }

    #[test_log::test(tokio::test)]
    async fn room_participant_limit() {
        let mut room = TestRoom::builder()
            .tariff(TariffDetails {
                id: TariffId::generate(),
                name: "Room Participant Limit".into(),
                quotas: BTreeMap::from_iter([(QuotaType::RoomParticipantLimit, 2)]),
                used_quota: BTreeMap::new(),
                disabled_features: BTreeSet::new(),
            })
            .spawn();

        // Alice and Bob join the room, the room participant limit is now reached
        let _alice = room.join_alice_moderator(0).await;
        let _charlie = room.join_charlie(0).await;

        // Charlie tries to join the room, but is rejected because the room participant limit is
        // reached
        let (socket, mut participant) = create_participant_connection();
        room.room_handle
            .accept_signaling_socket(
                socket,
                ClientParameters {
                    device_secret: "Device Secret Charlie".parse().unwrap(),
                    kind: client_parameters::ClientKind::Unregistered,
                    role: Role::User,
                },
            )
            .await
            .unwrap();

        // Charlie first receives a `JoinBlocked` event
        let msg = participant.receiver.next().await.unwrap();
        let SignalingSocketMessage::Text(msg) = msg else {
            panic!("Expected text message, received {msg:?}");
        };
        let event: SignalingEvent<CoreEvent> = serde_json::from_str(&msg).unwrap();
        assert_eq!(event.namespace, CORE_MODULE_ID);
        assert_matches!(
            event.payload,
            CoreEvent::JoinBlocked {
                reason: JoinBlockedReason::ParticipantLimitReached
            }
        );

        // Then Charlie receives a close frame with code 1013 (Try Again Later)
        let msg = participant.receiver.next().await.unwrap();
        assert!(
            matches!(
                msg,
                SignalingSocketMessage::Close(Some(CloseFrame { code: 1013, .. }))
            ),
            "Expected close message, received {msg:?}"
        );
    }
}
