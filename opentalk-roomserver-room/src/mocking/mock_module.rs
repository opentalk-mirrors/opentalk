// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_roomserver_signaling::signaling_module::{
    CreateReplica, ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap,
    SignalingModule, SignalingModuleDescription, SignalingModuleFeatureDescription,
    SignalingModuleInitData,
};
use opentalk_roomserver_types::signaling::module_error::{ModuleError, SignalingModuleError};
use opentalk_types_common::modules::{ModuleId, module_id};
use opentalk_types_signaling::{SignalingModuleFrontendData, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};

pub const MOCK_MODULE_ID: ModuleId = module_id!("mock");

pub struct MockModule {}

#[derive(Serialize, Deserialize)]
pub enum MockCommand {
    Valid,
    Invalid,
    Panic,
}
impl CreateReplica<MockEvent> for MockCommand {
    fn replicate(&self) -> Option<MockEvent> {
        None
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum MockEvent {
    Success,
    Error,
}

impl From<Error> for MockEvent {
    fn from(_: Error) -> Self {
        MockEvent::Error
    }
}

#[derive(Debug)]
pub struct Error;

impl ModuleError for Error {}

impl SignalingModuleDescription for MockModule {
    const MODULE_ID: ModuleId = MOCK_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles whiteboard integration. The whiteboard is a collaborative drawing board that can be used during the meeting.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for MockModule {
    const NAMESPACE: ModuleId = MOCK_MODULE_ID;

    type Incoming = MockCommand;

    type Outgoing = MockEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = MockJoinInfo;

    type PeerJoinInfo = MockPeerData;

    type Error = Error;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {})
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        participant_id: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let mut peer_event_data = PeerDataMap::default();
        let mut participant_data = PeerDataMap::default();
        for p in ctx
            .participants
            .connected()
            .ids()
            .filter(|&p| p != participant_id)
        {
            participant_data.insert(p, MockPeerData(format!("About {p} for {participant_id}")))?;
            peer_event_data.insert(p, MockPeerData(format!("From {participant_id} for {p}")))?;
        }
        Ok(ModuleJoinData {
            join_success: Some(MockJoinInfo(format!("Self: {participant_id}"))),
            peer_events: peer_event_data,
            peer_data: participant_data,
        })
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        participant_id: opentalk_types_signaling::ParticipantId,
        old_room: opentalk_roomserver_types::room_kind::RoomKind,
        new_room: opentalk_roomserver_types::room_kind::RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let mut switch_success = BTreeMap::new();
        for connection_id in ctx
            .participant_state(participant_id)
            .ok_or(Error)?
            .connections()
        {
            switch_success.insert(
                connection_id,
                Some(MockJoinInfo(format!(
                    "Switched room from {old_room:?} to {new_room:?}"
                ))),
            );
        }

        let mut peer = PeerDataMap::default();
        let mut participant_states = PeerDataMap::default();
        for p in ctx
            .participants
            .connected()
            .ids()
            .filter(|&p| p != participant_id)
        {
            peer.insert(p, MockPeerData(format!("From {participant_id} for {p}")))?;
            participant_states
                .insert(p, MockPeerData(format!("About {p} for {participant_id}")))?;
        }
        Ok(ModuleSwitchData {
            switch_success,
            peer_events: peer,
            peer_data: participant_states,
        })
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        participant_id: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_websocket_message(
        &mut self,
        ctx: &mut opentalk_roomserver_signaling::module_context::ModuleContext<'_, Self>,
        sender: opentalk_types_signaling::ParticipantId,
        connection_id: opentalk_roomserver_types::connection_id::ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            MockCommand::Valid => {
                ctx.send_ws_message([sender], MockEvent::Success).unwrap();
                Ok(())
            }
            MockCommand::Invalid => Err(Error.into()),
            MockCommand::Panic => panic!("Don't panic! All is not lost."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockJoinInfo(String);

impl SignalingModuleFrontendData for MockJoinInfo {
    const NAMESPACE: Option<ModuleId> = Some(MockModule::NAMESPACE);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPeerData(String);

impl SignalingModulePeerFrontendData for MockPeerData {
    const NAMESPACE: Option<ModuleId> = Some(MockModule::NAMESPACE);
}
