// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::InternalCommand;
use opentalk_roomserver_types::livekit_proxy::{
    LiveKitProxyRequest, PreparedSocket, websocket::LiveKitSocket,
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum SubroomAudioInternal {
    /// Prepare a livekit proxy socket by verifying access and connecting to the upstream LiveKit
    /// server.
    ConnectUpstreamSocket {
        websocket_request: Box<LiveKitProxyRequest>,
        return_channel: oneshot::Sender<Option<PreparedSocket>>,
    },

    /// Connect the client socket and finalize the proxy setup.
    ConnectDownstreamSocket {
        websocket_request: Box<LiveKitProxyRequest>,
        upstream_socket: Box<PreparedSocket>,
        downstream_socket: Box<dyn LiveKitSocket>,
        return_channel: oneshot::Sender<()>,
    },
}

impl InternalCommand for SubroomAudioInternal {}
