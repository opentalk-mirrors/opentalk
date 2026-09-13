// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::{E2eeError, MlsMessages, WelcomeMessage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum E2eeEvent {
    Welcome(WelcomeMessage),
    MlsMessages(MlsMessages),
    Error(E2eeError),
}

impl From<E2eeError> for E2eeEvent {
    fn from(err: E2eeError) -> Self {
        Self::Error(err)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{E2eeError, MlsMessages, WelcomeMessage};

    fn sample_welcome_message() -> WelcomeMessage {
        WelcomeMessage {
            welcome: Bytes::from_static(b"welcome-bytes"),
            ratchet_tree: Bytes::from_static(b"ratchet-tree-bytes"),
        }
    }

    fn sample_mls_messages() -> MlsMessages {
        MlsMessages {
            payload: vec![Bytes::from_static(b"mls1"), Bytes::from_static(b"mls2")],
        }
    }

    #[test]
    fn welcome() {
        let event = E2eeEvent::Welcome(sample_welcome_message());
        let json_value = json!({
            "message": "welcome",
            "welcome": b"welcome-bytes",
            "ratchet_tree": b"ratchet-tree-bytes",
        });
        assert_eq!(serde_json::to_value(&event).unwrap(), json_value);
        assert_eq!(
            serde_json::from_value::<E2eeEvent>(json_value).unwrap(),
            event
        );
    }

    #[test]
    fn mls_messages() {
        let event = E2eeEvent::MlsMessages(sample_mls_messages());
        let json_value = json!({
            "message": "mls_messages",
            "payload": [
                b"mls1",
                b"mls2",
            ]
        });
        assert_eq!(serde_json::to_value(&event).unwrap(), json_value);
        assert_eq!(
            serde_json::from_value::<E2eeEvent>(json_value).unwrap(),
            event
        );
    }

    #[test]
    fn error() {
        let event = E2eeEvent::Error(E2eeError::InvalidInvite);
        let json_value = json!({
            "message": "error",
            "error": "invalid_invite"
        });
        assert_eq!(serde_json::to_value(&event).unwrap(), json_value);
        assert_eq!(
            serde_json::from_value::<E2eeEvent>(json_value).unwrap(),
            event
        );
    }
}
