// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ExcalidrawError {
    InsufficientPermissions,
    AlreadyStarted,
    NotStarted,
    UnknownParticipant,
}

impl ModuleError for ExcalidrawError {}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ExcalidrawError;

    #[test]
    fn serialize_insufficient_permissions() {
        let error = ExcalidrawError::InsufficientPermissions;
        let raw = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions() {
        let raw = json!({
            "error": "insufficient_permissions",
        });

        let error: ExcalidrawError = serde_json::from_str(&raw.to_string()).unwrap();
        assert_eq!(error, ExcalidrawError::InsufficientPermissions);
    }

    #[test]
    fn serialize_already_started() {
        let error = ExcalidrawError::AlreadyStarted;
        let raw = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "already_started"
        }
        "#);
    }

    #[test]
    fn deserialize_already_started() {
        let raw = json!({
            "error": "already_started",
        });

        let error: ExcalidrawError = serde_json::from_str(&raw.to_string()).unwrap();
        assert_eq!(error, ExcalidrawError::AlreadyStarted);
    }

    #[test]
    fn serialize_not_started() {
        let error = ExcalidrawError::NotStarted;
        let raw = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "not_started"
        }
        "#);
    }

    #[test]
    fn deserialize_not_started() {
        let raw = json!({
            "error": "not_started",
        });

        let error: ExcalidrawError = serde_json::from_str(&raw.to_string()).unwrap();
        assert_eq!(error, ExcalidrawError::NotStarted);
    }

    #[test]
    fn serialize_unknown_participant() {
        let error = ExcalidrawError::UnknownParticipant;
        let raw = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "unknown_participant"
        }
        "#);
    }

    #[test]
    fn deserialize_unknown_participant() {
        let raw = json!({
            "error": "unknown_participant",
        });

        let error: ExcalidrawError = serde_json::from_str(&raw.to_string()).unwrap();
        assert_eq!(error, ExcalidrawError::UnknownParticipant);
    }
}
