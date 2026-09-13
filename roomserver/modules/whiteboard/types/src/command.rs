// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `whiteboard` namespace.

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::WhiteboardEvent;

/// Commands for the `whiteboard` namespace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum WhiteboardCommand {
    /// Initialize a new space for the room.
    ///
    /// There can only be one space per room.
    Initialize,

    /// Generates a PDF from the current whiteboard.
    GeneratePdf,
}

impl CreateReplica<WhiteboardEvent> for WhiteboardCommand {
    fn replicate(&self) -> Option<WhiteboardEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::WhiteboardCommand;

    #[test]
    fn serialize_initialize() {
        let cmd = WhiteboardCommand::Initialize;
        let produced = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "action": "initialize"
        }
        "#);
    }

    #[test]
    fn deserialize_initialize() {
        let json = json!({
            "action": "initialize"
        });
        let produced: WhiteboardCommand = serde_json::from_value(json).unwrap();

        assert_eq!(produced, WhiteboardCommand::Initialize);
    }

    #[test]
    fn serialize_generate_pdf() {
        let cmd = WhiteboardCommand::GeneratePdf;
        let produced = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "action": "generate_pdf"
        }
        "#);
    }

    #[test]
    fn deserialize_generate_pdf() {
        let json = json!({
            "action": "generate_pdf"
        });
        let produced: WhiteboardCommand = serde_json::from_value(json).unwrap();

        assert_eq!(produced, WhiteboardCommand::GeneratePdf);
    }
}
