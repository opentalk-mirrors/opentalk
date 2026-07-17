// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use anyhow::Context as _;
use icu_locid::langid;
use opentalk_roomserver_common::{
    application_state::ApplicationState,
    settings::{
        Task,
        runtime_settings::{
            reports::Reports,
            reports_typst::{ReportsTypst, reports_typst_packages_test_path},
        },
    },
};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    core::CoreEvent,
    module_settings::{ModuleSettings, SignalingModuleSettings},
    public_user_profile::PublicUserProfile,
    rate_limit::RateLimitSettings,
    room_kind::RoomKind,
    room_parameters::{EventContext, RoomParameters, WaitingRoom},
    tariff_details::TariffDetails,
};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{
    assets::AssetId,
    rooms::{RoomId, RoomPassword},
    streaming::RoomStreamingTarget,
    tariffs::{QuotaType, TariffId},
};
use tokio::{sync::watch, task::JoinHandle};

use super::participant::{MockParticipantJoined, MockParticipantJoining, ReceiveError};
use crate::{
    ModuleRegistry, RoomTaskHandle, RoomTaskHandleError,
    mocking::{
        participant::{
            MockParticipantWaiting, alice_public_profile, create_participant_connection,
        },
        socket::MockSocket,
    },
    storage::{
        memory_asset_storage::MemoryAssetStorage,
        memory_module_storage::MemoryModuleResourceStorage,
    },
    task::{RoomTask, context::RoomTaskContext},
};

#[derive(Debug)]
pub enum Error {
    RoomHandle(RoomTaskHandleError<MockSocket>),
    Participant(ReceiveError),
}

impl From<RoomTaskHandleError<MockSocket>> for Error {
    fn from(error: RoomTaskHandleError<MockSocket>) -> Self {
        Self::RoomHandle(error)
    }
}

impl From<ReceiveError> for Error {
    fn from(error: ReceiveError) -> Self {
        Self::Participant(error)
    }
}

fn settings() -> Task {
    let packages_path = reports_typst_packages_test_path();
    Task {
        conference: Default::default(),
        defaults: Default::default(),
        reports: Reports {
            typst: ReportsTypst { packages_path },
        },
    }
}

pub struct TestRoomBuilder {
    room_id: RoomId,
    settings: Task,
    room_parameters: RoomParameters,
    module_registry: ModuleRegistry,
}

impl TestRoomBuilder {
    pub fn new() -> Self {
        Self {
            room_id: RoomId::from_u128(1),
            settings: settings(),
            room_parameters: RoomParameters {
                created_by: alice_public_profile(),
                password: None,
                guest_access: true,
                waiting_room: WaitingRoom::Disabled,
                call_in: None,
                event: None,
                invite_code: None,
                tariff: TariffDetails {
                    id: TariffId::from_u128(1),
                    name: "Default Tariff".to_string(),
                    quotas: BTreeMap::default(),
                    used_quota: BTreeMap::new(),
                    disabled_features: BTreeSet::default(),
                },
                streaming_targets: Vec::new(),
                show_meeting_details: true,
                e2e_encryption: false,
                module_settings: ModuleSettings::new(),
                preferred_language: langid!("en"),
                fallback_language: langid!("en"),
                ws_rate_limit: None,
                allowed_origins: vec!["*".to_string()],
                room_idle_timeout: Duration::from_secs(10),
            },
            module_registry: ModuleRegistry::new(),
        }
    }

    pub fn event(mut self, event: EventContext) -> Self {
        self.room_parameters.event = Some(event);
        self
    }

    pub fn ws_rate_limit(mut self, rate_limit_settings: RateLimitSettings) -> Self {
        self.room_parameters.ws_rate_limit = Some(rate_limit_settings);
        self
    }

    pub fn register_module<M: SignalingModule + 'static>(mut self) -> Self {
        self.module_registry.add_module::<M>();
        // Only add an empty setting if none exist yet, so we don't overwrite what was set before.
        if !self.room_parameters.module_settings.contains(M::NAMESPACE) {
            self.room_parameters
                .module_settings
                .insert_empty(M::NAMESPACE);
        }
        self
    }

    pub fn add_init_module_data<T: SignalingModuleSettings>(
        mut self,
        data: &T,
    ) -> Result<Self, serde_json::Error> {
        self.room_parameters.module_settings.insert(data)?;
        Ok(self)
    }

    pub fn waiting_room(mut self, waiting_room: WaitingRoom) -> Self {
        self.room_parameters.waiting_room = waiting_room;
        self
    }

    pub fn room_id(mut self, room_id: RoomId) -> Self {
        self.room_id = room_id;
        self
    }

    pub fn tariff(mut self, tariff: TariffDetails) -> Self {
        self.room_parameters.tariff = tariff;
        self
    }

    pub fn storage_quota(mut self, quota: u64) -> Self {
        self.room_parameters
            .tariff
            .quotas
            .insert(QuotaType::MaxStorage, quota);
        self
    }

    pub fn owner(mut self, created_by: PublicUserProfile) -> Self {
        self.room_parameters.created_by = created_by;
        self
    }

    pub fn show_meeting_details(mut self, show_meeting_details: bool) -> Self {
        self.room_parameters.show_meeting_details = show_meeting_details;
        self
    }

    pub fn settings(mut self, update: impl FnOnce(&mut Task)) -> Self {
        update(&mut self.settings);
        self
    }

    pub fn streaming_target(mut self, streaming_target: RoomStreamingTarget) -> Self {
        self.room_parameters
            .streaming_targets
            .push(streaming_target);
        self
    }

    pub fn password(mut self, password: Option<RoomPassword>) -> Self {
        self.room_parameters.password = password;
        self
    }

    pub fn spawn(self) -> TestRoom {
        TestRoom::spawn(
            self.room_id,
            self.room_parameters,
            self.module_registry,
            self.settings,
        )
    }
}

impl Default for TestRoomBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestRoom {
    room_id: RoomId,
    pub room_handle: RoomTaskHandle<MockSocket>,
    pub join_handle: JoinHandle<()>,
    _settings: Arc<Task>,
    _app_state_tx: watch::Sender<ApplicationState>,
}

impl TestRoom {
    pub fn builder() -> TestRoomBuilder {
        TestRoomBuilder::new()
    }

    fn spawn(
        room_id: RoomId,
        room_parameters: RoomParameters,
        module_registry: ModuleRegistry,
        settings: Task,
    ) -> Self {
        let settings = Arc::new(settings);
        let (app_state_tx, app_state) = watch::channel(ApplicationState::Running);
        let asset_storage = Arc::new(MemoryAssetStorage::new(Quota {
            total: room_parameters.tariff.quota(&QuotaType::MaxStorage),
            used: room_parameters.tariff.used_quota(&QuotaType::MaxStorage),
        }));
        let module_resources = Arc::new(MemoryModuleResourceStorage::new());
        let ctx = RoomTaskContext {
            module_registry: module_registry.into(),
            asset_storage,
            module_resources,
            settings: Arc::clone(&settings),
            app_state,
        };

        let (room_handle, future_room) = RoomTask::setup(ctx, room_id, room_parameters.into());
        let join_handle = tokio::spawn(future_room);
        Self {
            room_id,
            room_handle,
            join_handle,
            _settings: settings,
            _app_state_tx: app_state_tx,
        }
    }

    pub async fn join_participant(
        &mut self,
        client_parameters: ClientParameters,
    ) -> Result<MockParticipantJoined, Error> {
        let (socket, participant) = create_participant_connection();
        self.room_handle
            .accept_signaling_socket(socket, client_parameters)
            .await?;
        let participant = participant.join_success().await?;

        Ok(participant)
    }

    pub async fn enter_waiting_room(
        &mut self,
        client_parameters: ClientParameters,
    ) -> Result<MockParticipantWaiting, Error> {
        let (socket, participant) = create_participant_connection();
        self.room_handle
            .accept_signaling_socket(socket, client_parameters)
            .await?;
        let participant = participant.join_waiting_room().await?;

        Ok(participant)
    }

    pub async fn join_alice_moderator(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::alice(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_bob(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::bob(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_charlie(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::charlie(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_dave(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::dave(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_frank_moderator(&mut self, device_number: usize) -> MockParticipantJoined {
        MockParticipantJoining::frank(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_gustav_guest(&mut self) -> MockParticipantJoined {
        MockParticipantJoining::gustav().join(self).await.unwrap()
    }

    pub async fn join_richard_registered_callin(
        &mut self,
        device_number: usize,
    ) -> MockParticipantJoined {
        MockParticipantJoining::richard_registered_callin(device_number)
            .join(self)
            .await
            .unwrap()
    }

    pub async fn join_recorder(
        &mut self,
        room_kind: RoomKind,
        device_number: usize,
    ) -> MockParticipantJoined {
        MockParticipantJoined::recorder(device_number)
            .join(self, room_kind)
            .await
            .unwrap()
    }

    pub async fn join_transcription(
        &mut self,
        room_kind: RoomKind,
        device_number: usize,
    ) -> MockParticipantJoined {
        MockParticipantJoined::transcription(device_number)
            .join(self, room_kind)
            .await
            .unwrap()
    }

    pub async fn waiting_room_bob(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::bob(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_charlie(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::charlie(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_dave(&mut self, device_number: usize) -> MockParticipantWaiting {
        MockParticipantJoining::dave(device_number)
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub async fn waiting_room_gustav_guest(&mut self) -> MockParticipantWaiting {
        MockParticipantJoining::gustav()
            .enter_waiting_room(self)
            .await
            .unwrap()
    }

    pub fn id(&self) -> RoomId {
        self.room_id
    }

    pub async fn stored_asset(&self, id: AssetId) -> Option<Vec<u8>> {
        let assets = self.downcast_asset_storage();
        assets.asset(id).await
    }

    pub async fn stored_assets(&self) -> Vec<Vec<u8>> {
        let assets = self.downcast_asset_storage();
        assets.all_assets().await
    }

    pub async fn file_count(&self) -> usize {
        let assets = self.downcast_asset_storage();
        assets.asset_count().await
    }

    fn downcast_asset_storage(&self) -> Arc<MemoryAssetStorage> {
        let assets = self.room_handle.assets();
        Arc::downcast(assets).expect("The RoomTask must be configured with MemoryStorage.")
    }

    pub fn downcast_module_resource_storage(&self) -> Arc<MemoryModuleResourceStorage> {
        let module_resources = self.room_handle.module_resources();
        Arc::downcast(module_resources)
            .expect("The RoomTask must be configured with MemoryStorage.")
    }
}

pub async fn flush_connected_events(others: &mut [&mut MockParticipantJoined]) {
    for p in others {
        let event = p
            .receive::<CoreEvent>()
            .await
            .with_context(|| {
                format!(
                    "`{}` didn't receive an event",
                    p.join_success().display_name
                )
            })
            .unwrap();
        assert!(
            matches!(event.payload, CoreEvent::ParticipantConnected { .. },),
            "Participant `{}` didn't receive CoreEvent::ParticipantConnected, got {:?}",
            p.join_success().display_name.as_str(),
            event
        );
    }
}

pub async fn flush_disconnected_events(others: &mut [&mut MockParticipantJoined]) {
    for p in others {
        let event = p
            .receive::<CoreEvent>()
            .await
            .with_context(|| {
                format!(
                    "`{}` didn't receive an event",
                    p.join_success().display_name
                )
            })
            .unwrap();

        assert!(
            matches!(event.payload, CoreEvent::ParticipantDisconnected { .. },),
            "Participant `{}` didn't receive CoreEvent::ParticipantDisconnected, got {:?}",
            p.join_success().display_name.as_str(),
            event,
        );
    }
}
