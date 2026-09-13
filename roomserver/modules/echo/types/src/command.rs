// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::EchoEvent;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EchoCommand {
    /// A ping command to which the module responds with a [`Pong`](crate::event::EchoEvent::Pong)
    /// event.
    Ping,
}

impl CreateReplica<EchoEvent> for EchoCommand {
    fn replicate(&self) -> Option<EchoEvent> {
        None
    }
}
