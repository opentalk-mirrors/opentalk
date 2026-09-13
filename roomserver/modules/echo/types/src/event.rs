// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::convert::Infallible;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum EchoEvent {
    /// The response to a [`EchoCommand::Ping`](super::command::EchoCommand::Ping).
    Pong,
}

// This is here to satisfy the `SignalingModule` trait requirements.
impl From<Infallible> for EchoEvent {
    fn from(_: Infallible) -> Self {
        panic!("Infallible cannot be instantiated")
    }
}
