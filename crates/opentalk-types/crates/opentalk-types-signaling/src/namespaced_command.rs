// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::modules::ModuleId;

/// An envelope of a command annotated with their respective module id.
///
/// This is used for WebSocket messages sent to the backend.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NamespacedCommand<O> {
    /// The module to which the message is targeted
    #[cfg_attr(feature = "serde", serde(rename = "namespace"))]
    pub module: ModuleId,
    /// The payload of the message
    pub payload: O,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use opentalk_types_common::modules::ModuleId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::NamespacedCommand;

    #[test]
    fn test_namespaced_command_serialization() {
        let module = ModuleId::example_data();
        let payload = "test_payload".to_string();

        let command = NamespacedCommand { module, payload };

        let expected = json!({
            "namespace": "mymodule",
            "payload": "test_payload"
        });

        let serialized = serde_json::to_value(&command).unwrap();
        assert_eq!(serialized, expected);

        let deserialized: NamespacedCommand<String> =
            serde_json::from_value(expected.clone()).unwrap();

        assert_eq!(deserialized, command);
    }
}
