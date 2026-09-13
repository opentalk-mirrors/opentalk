// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::{ModuleId, module_id};
use serde::{Deserialize, Serialize};

pub const ERROR_MODULE_ID: ModuleId = module_id!("error");

/// Errors that are reported back to the client via the signaling socket.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum SignalingError {
    /// The requested namespace is unknown to the room server
    UnknownNamespace { invalid_namespace: String },

    /// The received message was not valid JSON.
    InvalidJson { message: String },

    /// A non-specific internal error
    Internal,

    /// A fatal error in a module occurred. The module was terminated.
    ///
    /// Further messages to this namespace will be ignored.
    FatalModuleError { namespace: ModuleId },

    /// The requested operation is not supported in the current context
    NotSupported,

    /// The client is sending messages too quickly. Slow down.
    SlowDown,
}

impl From<serde_json::Error> for SignalingError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson {
            message: format!("Failed to deserialize message: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::SignalingError;

    #[test]
    fn unknown_namespace() {
        let val = SignalingError::UnknownNamespace {
            invalid_namespace: "test".into(),
        };

        let val = serde_json::to_value(val).unwrap();

        assert_eq!(
            val,
            json!( {
                "error": "unknown_namespace",
                "invalid_namespace": "test"
            }),
        );
    }

    #[test]
    fn serialize_slow_down() {
        let raw = serde_json::to_string_pretty(&SignalingError::SlowDown).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "slow_down"
        }
        "#);
    }

    #[test]
    fn deserialize_slow_down() {
        let produced: SignalingError = serde_json::from_value(json!({
            "error": "slow_down"
        }))
        .unwrap();

        assert_eq!(produced, SignalingError::SlowDown);
    }
}
