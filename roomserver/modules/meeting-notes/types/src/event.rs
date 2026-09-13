// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `meeting-notes` namespace

use opentalk_roomserver_signaling::storage::assets::StorageError;
use opentalk_roomserver_types::{
    connection_id::ConnectionId, signaling::module_error::ModuleError,
};
use opentalk_types_common::assets::AssetId;
use serde::{Deserialize, Serialize};

/// Events sent out by the `meeting-notes` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum MeetingNotesEvent {
    /// The receiving participant was granted write access.
    WriteAccessReceived {
        /// The session URL which should be used to access the meeting notes
        url: String,
    },

    /// The receiving participant was granted readonly access.
    ///
    /// If the participant had write access before, it was revoked.
    ReadAccessReceived {
        /// The session URL which should be used to access the meeting notes
        url: String,
    },

    /// Handle to the PDF asset
    PdfCreated {
        /// The file name of the PDF asset
        filename: String,

        /// The asset id for the PDF asset
        asset_id: AssetId,
    },

    /// Event sent to moderators when the readers or writers of the notes changed
    AccessChanged {
        /// The new readers of the notes
        readers: Vec<ConnectionId>,
        /// The new writers of the notes
        writers: Vec<ConnectionId>,
    },

    /// An error happened when executing a `meeting-notes` command
    Error(MeetingNotesError),
}

impl From<MeetingNotesError> for MeetingNotesEvent {
    fn from(value: MeetingNotesError) -> Self {
        Self::Error(value)
    }
}

/// Errors from the `meeting-notes` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum MeetingNotesError {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The request contains invalid participant ids
    InvalidParticipantSelection,
    /// Is send when another instance just started initializing and etherpad is not available yet
    CurrentlyInitializing,
    /// The etherpad initialization failed
    FailedInitialization,
    /// The etherpad is not yet initialized
    NotInitialized,
    /// The requesting user has exceeded their storage
    StorageExceeded,
    /// Internal error while saving the notes
    InternalStorage,
    /// Generating the etherpad URL failed
    FailedToGenerateUrl {
        /// The connection ids for which the URL generation failed
        connection_ids: Vec<ConnectionId>,
    },
}

impl ModuleError for MeetingNotesError {}

impl From<StorageError> for MeetingNotesError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::QuotaExceeded => Self::StorageExceeded,
            StorageError::Internal(..) | StorageError::ReadAsset(..) => Self::InternalStorage,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::{self, json};

    use super::*;

    #[test]
    fn serialize_write_url() {
        let event = MeetingNotesEvent::WriteAccessReceived {
            url: "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group".into(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "write_access_received",
          "url": "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group"
        }
        "#);
    }

    #[test]
    fn deserialize_write_url() {
        let expected = json!({
            "message": "write_access_received",
            "url": "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group",
        });

        let message = MeetingNotesEvent::WriteAccessReceived {
            url: "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group".into()
        };

        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_read_url() {
        let event = MeetingNotesEvent::ReadAccessReceived {
            url: "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group".into()
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "read_access_received",
          "url": "http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group"
        }
        "#);
    }

    #[test]
    fn deserialize_read_url() {
        let expected = json!({
            "message": "read_access_received",
            "url": "http://localhost:9001/auth_session?sessionID=s.session_id&padName=r.readonly_id",
        });

        let message = MeetingNotesEvent::ReadAccessReceived {
            url: "http://localhost:9001/auth_session?sessionID=s.session_id&padName=r.readonly_id"
                .into(),
        };

        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_insufficient_permissions() {
        let event = MeetingNotesEvent::Error(MeetingNotesError::InsufficientPermissions);

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions() {
        let expected = json!({"message": "error", "error": "insufficient_permissions"});

        let message = MeetingNotesEvent::Error(MeetingNotesError::InsufficientPermissions);
        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_currently_initialization() {
        let event = MeetingNotesEvent::Error(MeetingNotesError::CurrentlyInitializing);

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "error",
          "error": "currently_initializing"
        }
        "#);
    }

    #[test]
    fn deserialize_currently_initializing() {
        let expected = json!({"message": "error", "error": "currently_initializing"});

        let message = MeetingNotesEvent::Error(MeetingNotesError::CurrentlyInitializing);
        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_failed_initialization() {
        let event = MeetingNotesEvent::Error(MeetingNotesError::FailedInitialization);

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "error",
          "error": "failed_initialization"
        }
        "#);
    }

    #[test]
    fn deserialize_failed_initialization() {
        let expected = json!({"message": "error", "error": "failed_initialization"});

        let message = MeetingNotesEvent::Error(MeetingNotesError::FailedInitialization);
        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_invalid_participant_selection() {
        let event = MeetingNotesEvent::Error(MeetingNotesError::InvalidParticipantSelection);

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "error",
          "error": "invalid_participant_selection"
        }
        "#);
    }

    #[test]
    fn invalid_participant_selection() {
        let expected = json!({"message": "error", "error": "invalid_participant_selection"});

        let message = MeetingNotesEvent::Error(MeetingNotesError::InvalidParticipantSelection);
        let produced = serde_json::to_value(message).unwrap();

        assert_eq!(expected, produced);
    }
}
