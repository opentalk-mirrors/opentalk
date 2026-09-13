// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::MeetingNotesEvent;

/// Commands for the `meeting_notes` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MeetingNotesCommand {
    /// Select a participant as writer
    GrantWriteAccess {
        /// The targeted participants
        participant_ids: BTreeSet<ParticipantId>,
    },

    /// Deselect a participant as writer
    RevokeWriteAccess {
        /// The targeted participants
        participant_ids: BTreeSet<ParticipantId>,
    },

    /// Generates a pdf of the current meeting-notes
    GeneratePdf,
}

impl CreateReplica<MeetingNotesEvent> for MeetingNotesCommand {
    fn replicate(&self) -> Option<MeetingNotesEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_select_writer() {
        let command = MeetingNotesCommand::GrantWriteAccess {
            participant_ids: BTreeSet::from_iter([
                ParticipantId::nil(),
                ParticipantId::from_u128(0x1),
            ]),
        };

        assert_snapshot!(
            serde_json::to_string_pretty(&command).unwrap(), @r#"
        {
          "action": "grant_write_access",
          "participant_ids": [
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_select_writer() {
        let json = json!({
            "action": "grant_write_access",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        });

        let produced: MeetingNotesCommand = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([
                    ParticipantId::nil(),
                    ParticipantId::from_u128(0x1)
                ]),
            }
        );
    }

    #[test]
    fn serialize_deselect_writer() {
        let command = MeetingNotesCommand::RevokeWriteAccess {
            participant_ids: BTreeSet::from_iter([
                ParticipantId::nil(),
                ParticipantId::from_u128(0x1),
            ]),
        };

        assert_snapshot!(
            serde_json::to_string_pretty(&command).unwrap(), @r#"
        {
          "action": "revoke_write_access",
          "participant_ids": [
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_deselect_writer() {
        let json = json!({
            "action": "revoke_write_access",
            "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
        });

        let produced: MeetingNotesCommand = serde_json::from_value(json).unwrap();
        assert_eq!(
            produced,
            MeetingNotesCommand::RevokeWriteAccess {
                participant_ids: BTreeSet::from_iter([
                    ParticipantId::nil(),
                    ParticipantId::from_u128(0x1)
                ]),
            }
        );
    }

    #[test]
    fn serialize_generate_pdf() {
        let command = MeetingNotesCommand::GeneratePdf;

        assert_snapshot!(
            serde_json::to_string_pretty(&command).unwrap(), @r#"
        {
          "action": "generate_pdf"
        }
        "#);
    }

    #[test]
    fn deserialize_generate_pdf() {
        let json = serde_json::json!({
            "action": "generate_pdf"
        });

        let produced: MeetingNotesCommand = serde_json::from_value(json).unwrap();
        assert_eq!(produced, MeetingNotesCommand::GeneratePdf);
    }
}
