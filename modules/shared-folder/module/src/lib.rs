// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! # Shared Folder Module
//!
//! ## Functionality
//!
//! Shares URL and password to access a shared folder. Moderators as provided with a
//! read-write URL while other users receive a read-only URL.
//!
//! This module requires that the [`SharedFolder`] is available in the
//! [`RoomParameters::module_settings`]. If these are not present, the module won't initialize and
//! stays disabled.
//!
//! [`RoomParameters::module_settings`]: opentalk_roomserver_types::room_parameters::RoomParameters::module_settings

use std::convert::Infallible;

pub use internal::UpdateSharedFolder;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, NoSettings, PeerDataMap, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    client_parameters::Role, connection_id::ConnectionId,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_shared_folder::{
    SHARED_FOLDER_MODULE_ID, command::SharedFolderCommand, event::SharedFolderEvent, internal,
};
use opentalk_types_common::{modules::ModuleId, shared_folders::SharedFolder};
use opentalk_types_signaling::ParticipantId;

pub struct SharedFolderModule {
    state: SharedFolder,
}

impl SignalingModuleDescription for SharedFolderModule {
    const MODULE_ID: ModuleId = SHARED_FOLDER_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles shared folder integration. This allows automatic \
        creation of shares on a NextCloud instance using the \
        [OCS API](https://docs.nextcloud.com/server/latest/developer_manual/client_apis/OCS/ocs-api-overview.html) \
        or on an OpenCloud instance using its [Graph API](https://owncloud.dev/libre-graph-api/) \
        and [WebDAV](https://docs.opencloud.eu/docs/dev/apis/webdav) endpoints.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for SharedFolderModule {
    const NAMESPACE: ModuleId = SHARED_FOLDER_MODULE_ID;

    type Incoming = SharedFolderCommand;

    type Outgoing = SharedFolderEvent;

    type Internal = internal::UpdateSharedFolder;

    type Loopback = ();

    type JoinInfo = SharedFolder;

    type PeerJoinInfo = ();

    type Error = Infallible;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let shared_folder_state = init_data
            .room_parameters
            .event
            .as_ref()
            .and_then(|e| e.shared_folder.as_ref())
            .cloned();
        match shared_folder_state {
            Some(state) => Some(Self {
                state: state.clone(),
            }),
            None => {
                tracing::debug!("Received no SharedFolder configuration, module disabled");
                None
            }
        }
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        if ctx.participant_role(participant_id) == Some(Role::Moderator) {
            Ok(ModuleJoinData {
                join_success: Some(self.state.clone()),
                peer_events: PeerDataMap::default(),
                peer_data: PeerDataMap::default(),
            })
        } else {
            Ok(ModuleJoinData {
                join_success: Some(self.state.clone().without_write_access()),
                peer_events: PeerDataMap::default(),
                peer_data: PeerDataMap::default(),
            })
        }
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
        _ctx: &mut ModuleContext<'_, Self>,
        _sender: ParticipantId,
        _connection_id: ConnectionId,
        _content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_internal_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        command: Self::Internal,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let Some(participant) = ctx.participants.connected().get(&command.participant_id) else {
            // Participant disconnected, nothing to do
            return Ok(());
        };

        let shared_folder = if participant.is_moderator() {
            self.state.clone()
        } else {
            self.state.clone().without_write_access()
        };

        ctx.send_ws_message(
            [command.participant_id],
            SharedFolderEvent::Updated(shared_folder),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::SharedFolderModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&SharedFolderModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[shared_folder]");
    }
}
