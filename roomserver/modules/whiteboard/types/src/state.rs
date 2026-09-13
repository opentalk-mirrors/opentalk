// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `whiteboard` namespace

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::SignalingModuleFrontendData;
use serde::{Deserialize, Serialize};
use url::Url;

/// Information about a spacedeck space, aka a whiteboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceInfo {
    /// The unique identifier of the space.
    pub id: String,
    /// The URL used to access the space. This is sent to the participants.
    pub url: Url,
}

/// The state of the `whiteboard` module.
///
/// This struct is sent to the participant in the `join_success` message when they join the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status", content = "url")]
pub enum WhiteboardState {
    /// Whiteboard is initializing.
    Initializing,

    /// Whiteboard is initialized.
    Initialized(Url),
}

impl SignalingModuleFrontendData for WhiteboardState {
    const NAMESPACE: Option<ModuleId> = Some(crate::WHITEBOARD_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serde_json::json;
    use url::Url;

    use super::WhiteboardState;

    #[test]
    fn serialize_initializing() {
        let state = WhiteboardState::Initializing;
        let produced = serde_json::to_string_pretty(&state).unwrap();
        assert_snapshot!(produced, @r#"
        {
          "status": "initializing"
        }
        "#);
    }

    #[test]
    fn deserialize_initializing() {
        let json = json!({
            "status": "initializing"
        });
        let parsed: WhiteboardState = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, WhiteboardState::Initializing);
    }

    #[test]
    fn serialize_initialized() {
        let state = WhiteboardState::Initialized(Url::parse("https://example.com").unwrap());
        let produced = serde_json::to_string_pretty(&state).unwrap();
        assert_snapshot!(produced, @r#"
        {
          "status": "initialized",
          "url": "https://example.com/"
        }
        "#);
    }

    #[test]
    fn deserialize_initialized() {
        let json = json!({
            "status": "initialized",
            "url": "https://example.com/"
        });
        let parsed: WhiteboardState = serde_json::from_value(json).unwrap();
        assert_eq!(
            parsed,
            WhiteboardState::Initialized(Url::parse("https://example.com").unwrap())
        );
    }
}
