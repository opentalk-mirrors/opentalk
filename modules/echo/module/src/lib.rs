// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::convert::Infallible;

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, NoOp, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_echo::{ECHO_MODULE_ID, command::EchoCommand, event::EchoEvent};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub struct EchoModule;

impl SignalingModuleDescription for EchoModule {
    const MODULE_ID: ModuleId = ECHO_MODULE_ID;
    const DESCRIPTION: &'static str = "Used for internal connection checking and development";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];
}

impl SignalingModule for EchoModule {
    const NAMESPACE: ModuleId = ECHO_MODULE_ID;

    type Incoming = EchoCommand;

    type Outgoing = EchoEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = Infallible;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self)
    }

    fn on_participant_joined(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData::default())
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            EchoCommand::Ping => ctx.send_ws_message([participant_id], EchoEvent::Pong)?,
        }
        Ok(())
    }

    fn on_websocket_message_lobby(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            EchoCommand::Ping => ctx.send_ws_message_to_lobby([sender], EchoEvent::Pong)?,
        }
        Ok(())
    }
}
