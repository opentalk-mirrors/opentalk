// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, NoOp, NoSettings, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_e2ee::{
    E2EE_MODULE_ID, E2eeCommand, E2eeError, E2eeEvent, MlsMessages, WelcomeMessage,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

#[derive(Debug)]
pub struct E2eeModule;

impl SignalingModuleDescription for E2eeModule {
    const MODULE_ID: ModuleId = E2EE_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles end-to-end encryption functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for E2eeModule {
    const NAMESPACE: ModuleId = E2EE_MODULE_ID;

    type Incoming = E2eeCommand;

    type Outgoing = E2eeEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = E2eeError;

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
        _sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            E2eeCommand::Invite {
                invitee,
                welcome_message,
                mls_messages,
            } => self.publish_invite(ctx, connection_id, invitee, welcome_message, mls_messages),
            E2eeCommand::Message(message) => Self::forward_message(ctx, connection_id, message),
        }
    }
}

impl E2eeModule {
    fn publish_invite(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ConnectionId,
        invitee: ConnectionId,
        welcome_message: WelcomeMessage,
        mls_messages: MlsMessages,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        Self::ensure_valid_invite(ctx, invitee, &welcome_message, &mls_messages)?;

        // send MLS message to all but the invitee and sender
        ctx.send_ws_message_to_connections(
            ctx.participants
                .connected()
                .room(ctx.room)
                .connection_ids()
                .filter(|&p| p != sender && p != invitee),
            E2eeEvent::MlsMessages(mls_messages),
        )?;

        ctx.send_ws_message_to_connections([invitee], E2eeEvent::Welcome(welcome_message))?;

        Ok(())
    }

    fn ensure_valid_invite(
        ctx: &mut ModuleContext<'_, Self>,
        invitee: ConnectionId,
        welcome_message: &WelcomeMessage,
        mls_messages: &MlsMessages,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        if !welcome_message.is_valid() || !mls_messages.is_valid() {
            return Err(E2eeError::InvalidInvite.into());
        }
        Self::ensure_valid_connection(ctx, invitee)
    }

    fn ensure_valid_connection(
        ctx: &mut ModuleContext<'_, Self>,
        connection: ConnectionId,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        if ctx
            .participants
            .connected()
            .room(ctx.room)
            .connection_ids()
            .any(|c| c == connection)
        {
            Ok(())
        } else {
            Err(E2eeError::InvalidParticipantTarget.into())
        }
    }

    fn forward_message(
        ctx: &mut ModuleContext<'_, E2eeModule>,
        connection_id: ConnectionId,
        message: MlsMessages,
    ) -> Result<(), SignalingModuleError<<Self as SignalingModule>::Error>> {
        ctx.send_ws_message_to_connections(
            ctx.participants
                .connected()
                .room(ctx.room)
                .connection_ids()
                .filter(|&c| connection_id != c),
            E2eeEvent::MlsMessages(message),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::E2eeModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&E2eeModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[e2ee]");
    }
}
