// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::SharedFolderEvent;

/// Incoming websocket messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SharedFolderCommand {}

impl CreateReplica<SharedFolderEvent> for SharedFolderCommand {
    fn replicate(&self) -> Option<SharedFolderEvent> {
        None
    }
}
