// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bytes::Bytes;
use opentalk_types_common::modules::{ModuleId, module_id};

mod command;
mod error;
mod event;

pub use command::E2eeCommand;
pub use error::E2eeError;
pub use event::E2eeEvent;

pub const E2EE_MODULE_ID: ModuleId = module_id!("e2ee");

/// Welcome and ratchet tree sent to the invitee
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WelcomeMessage {
    pub welcome: Bytes,
    pub ratchet_tree: Bytes,
}

impl WelcomeMessage {
    pub fn is_valid(&self) -> bool {
        !self.welcome.is_empty() && !self.ratchet_tree.is_empty()
    }
}

/// Arbitrary encrypted message. This can be any kind of MLS or application message
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MlsMessages {
    pub payload: Vec<Bytes>,
}

impl MlsMessages {
    pub fn is_valid(&self) -> bool {
        !self.payload.is_empty()
    }
}
