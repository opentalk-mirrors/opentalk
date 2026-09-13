// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeSet, fmt::Debug};

use opentalk_roomserver_signaling::signaling_module::InternalCommand;
use opentalk_roomserver_types::livekit_proxy::{
    LiveKitProxyRequest, PreparedSocket, websocket::LiveKitSocket,
};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;
use url::Url;

use crate::MicrophoneRestrictionState;

/// Internal LiveKit commands that can be sent by other modules
#[derive(Debug)]
pub enum LiveKitInternal {
    /// Mutes participants
    Mute {
        /// The original sender of the command
        sender: Option<ParticipantId>,
        /// The participants that should get muted
        participants: BTreeSet<ParticipantId>,
        /// The return channel for the result of the operation
        return_channel: oneshot::Sender<ParticipantsMuted>,
    },

    /// Enables or disables microphone restriction state
    UpdateMicrophoneRestrictions {
        /// The original sender of the command
        sender: ParticipantId,
        /// The new microphone restriction state
        new_state: MicrophoneRestrictionState,
        /// The return channel for the result of the operation
        return_channel:
            oneshot::Sender<Result<MicrophoneRestrictionState, MicrophoneRestrictionError>>,
    },

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

    /// Return the configured LiveKit service URL
    GetLivekitServiceUrl {
        return_channel: oneshot::Sender<Url>,
    },
}

/// The type of error that can occur when updating the microphone restriction state
#[derive(Debug)]
pub enum MicrophoneRestrictionErrorKind {
    /// The received command cannot be executed since there is already a conflicting ongoing task.
    ConflictingTask,
    /// Livekit server is not available
    LivekitUnavailable,
}

/// Internal error that can occur when updating the microphone restriction state
#[derive(Debug)]
pub struct MicrophoneRestrictionError {
    /// The original sender of the command
    pub sender: ParticipantId,
    /// The type of error that occurred
    pub error: MicrophoneRestrictionErrorKind,
}

/// Participants were muted by a moderator
#[derive(Debug)]
pub struct ParticipantsMuted {
    /// The moderator that sent the command
    pub sender: Option<ParticipantId>,
    /// The participants that were muted
    pub participants: BTreeSet<ParticipantId>,
}

impl InternalCommand for LiveKitInternal {}
