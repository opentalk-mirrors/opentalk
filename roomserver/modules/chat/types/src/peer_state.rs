// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `chat` namespace

use opentalk_types_common::users::GroupName;
use serde::{Deserialize, Serialize};

/// The state of other participants in the `chat` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatPeerState {
    /// A list of group chats
    pub groups: Vec<GroupName>,
}

impl opentalk_types_signaling::SignalingModulePeerFrontendData for ChatPeerState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> = Some(crate::CHAT_MODULE_ID);
}
