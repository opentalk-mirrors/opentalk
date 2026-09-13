// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{
    breakout::{BREAKOUT_MODULE_ID, event::BreakoutEvent},
    core::{CORE_MODULE_ID, CoreEvent},
    error::{ERROR_MODULE_ID, SignalingError},
};
use opentalk_roomserver_types_automod::{AUTOMOD_MODULE_ID, event::AutomodEvent};
use opentalk_roomserver_types_chat::{CHAT_MODULE_ID, event::ChatEvent};
use opentalk_roomserver_types_echo::{ECHO_MODULE_ID, event::EchoEvent};
use opentalk_roomserver_types_legal_vote::{LEGAL_VOTE_MODULE_ID, event::LegalVoteEvent};
use opentalk_roomserver_types_meeting_notes::{MEETING_NOTES_MODULE_ID, MeetingNotesEvent};
use opentalk_roomserver_types_meeting_report::{
    MEETING_REPORT_MODULE_ID, event::MeetingReportEvent,
};
use opentalk_roomserver_types_moderation::{MODERATION_MODULE_ID, event::ModerationEvent};
use opentalk_roomserver_types_polls::{POLLS_MODULE_ID, event::PollsEvent};
use opentalk_roomserver_types_raise_hands::{RAISE_HANDS_MODULE_ID, event::RaiseHandsEvent};
use opentalk_roomserver_types_recording::{RECORDING_MODULE_ID, event::RecordingEvent};
use opentalk_roomserver_types_subroom_audio::{SUBROOM_AUDIO_MODULE_ID, event::SubroomAudioEvent};
use opentalk_roomserver_types_timer::{TIMER_MODULE_ID, TimerEvent};
use opentalk_roomserver_types_training_participation_report::{
    TRAINING_PARTICIPATION_REPORT_MODULE_ID, TrainingParticipationReportEvent,
};
use opentalk_roomserver_types_transcription::{TRANSCRIPTION_MODULE_ID, event::TranscriptionEvent};
use opentalk_roomserver_types_whiteboard::{WHITEBOARD_MODULE_ID, WhiteboardEvent};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use serde::{Deserialize, Serialize};
// reexport events for easier usage
pub use {
    opentalk_roomserver_types_e2ee::{E2EE_MODULE_ID, E2eeEvent},
    opentalk_roomserver_types_excalidraw::{EXCALIDRAW_MODULE_ID, ExcalidrawEvent},
    opentalk_roomserver_types_livekit::{
        Credentials, LIVEKIT_MODULE_ID, LiveKitError, LiveKitEvent, LiveKitState,
    },
    opentalk_roomserver_types_reaction::{REACTION_MODULE_ID, ReactionEvent},
    opentalk_roomserver_types_shared_folder::{
        event::SharedFolder,
        {SHARED_FOLDER_MODULE_ID, event::SharedFolderEvent},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    timestamp: Timestamp,
    #[serde(flatten)]
    pub payload: SignalingModuleEvent,
}

impl SignalingEvent {
    pub fn namespace(&self) -> ModuleId {
        self.payload.namespace()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
pub enum SignalingModuleEvent {
    Error(SignalingError),
    Automod(AutomodEvent),
    Core(CoreEvent),
    Breakout(BreakoutEvent),
    Echo(EchoEvent),
    Chat(ChatEvent),

    #[serde(rename = "livekit")]
    LiveKit(LiveKitEvent),

    E2ee(E2eeEvent),

    Timer(TimerEvent),
    Polls(PollsEvent),
    SharedFolder(SharedFolderEvent),
    MeetingReport(MeetingReportEvent),
    Moderation(ModerationEvent),
    RaiseHands(RaiseHandsEvent),
    SubroomAudio(SubroomAudioEvent),
    MeetingNotes(MeetingNotesEvent),
    Whiteboard(WhiteboardEvent),
    Excalidraw(ExcalidrawEvent),
    LegalVote(LegalVoteEvent),
    TrainingParticipationReport(TrainingParticipationReportEvent),
    Recording(RecordingEvent),
    Transcription(TranscriptionEvent),
    Reaction(ReactionEvent),
}

impl SignalingModuleEvent {
    pub fn namespace(&self) -> ModuleId {
        match self {
            Self::Error(..) => ERROR_MODULE_ID,
            Self::Automod(..) => AUTOMOD_MODULE_ID,
            Self::Core(..) => CORE_MODULE_ID,
            Self::Breakout(..) => BREAKOUT_MODULE_ID,
            Self::Echo(..) => ECHO_MODULE_ID,
            Self::Chat(..) => CHAT_MODULE_ID,
            Self::LiveKit(..) => LIVEKIT_MODULE_ID,
            Self::E2ee(..) => E2EE_MODULE_ID,
            Self::Timer(..) => TIMER_MODULE_ID,
            Self::Polls(..) => POLLS_MODULE_ID,
            Self::SharedFolder(..) => SHARED_FOLDER_MODULE_ID,
            Self::MeetingReport(..) => MEETING_REPORT_MODULE_ID,
            Self::Moderation(..) => MODERATION_MODULE_ID,
            Self::RaiseHands(..) => RAISE_HANDS_MODULE_ID,
            Self::SubroomAudio(..) => SUBROOM_AUDIO_MODULE_ID,
            Self::MeetingNotes(..) => MEETING_NOTES_MODULE_ID,
            Self::Whiteboard(..) => WHITEBOARD_MODULE_ID,
            Self::Excalidraw(..) => EXCALIDRAW_MODULE_ID,
            Self::LegalVote(..) => LEGAL_VOTE_MODULE_ID,
            Self::TrainingParticipationReport(..) => TRAINING_PARTICIPATION_REPORT_MODULE_ID,
            Self::Recording(..) => RECORDING_MODULE_ID,
            Self::Transcription(..) => TRANSCRIPTION_MODULE_ID,
            Self::Reaction(..) => REACTION_MODULE_ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use insta::assert_snapshot;
    use opentalk_roomserver_types::{
        breakout::event::BreakoutEvent, connection_id::ConnectionId, core::CoreEvent,
        error::SignalingError,
    };
    use opentalk_roomserver_types_automod::event::{AutomodEvent, StoppedReason};
    use opentalk_roomserver_types_chat::event::ChatEvent;
    use opentalk_roomserver_types_echo::event::EchoEvent;
    use opentalk_roomserver_types_excalidraw::ExcalidrawEvent;
    use opentalk_roomserver_types_legal_vote::{
        LegalVoteEvent,
        token::Token,
        vote::{LegalVoteId, VoteOption},
    };
    use opentalk_roomserver_types_livekit::LiveKitEvent;
    use opentalk_roomserver_types_meeting_notes::MeetingNotesEvent;
    use opentalk_roomserver_types_meeting_report::event::MeetingReportEvent;
    use opentalk_roomserver_types_moderation::event::ModerationEvent;
    use opentalk_roomserver_types_polls::{
        ChoiceId, PollId,
        command::{Choices, Vote},
        event::PollsEvent,
    };
    use opentalk_roomserver_types_raise_hands::event::RaiseHandsEvent;
    use opentalk_roomserver_types_reaction::ReactionEvent;
    use opentalk_roomserver_types_subroom_audio::{
        WhisperId,
        event::SubroomAudioEvent,
        state::{WhisperGroup, WhisperState},
    };
    use opentalk_roomserver_types_timer::TimerEvent;
    use opentalk_roomserver_types_training_participation_report::TrainingParticipationReportEvent;
    use opentalk_roomserver_types_transcription::{
        event::TranscriptionEvent, state::TranscriptionStatus,
    };
    use opentalk_roomserver_types_whiteboard::WhiteboardEvent;
    use opentalk_types_common::{assets::AssetId, modules::ModuleId, time::Timestamp};
    use opentalk_types_signaling::ParticipantId;
    use serde::Deserialize;
    use url::Url;

    use super::SignalingModuleEvent;
    use crate::api::event::SignalingEvent;

    #[derive(Debug, Clone, Deserialize)]
    pub struct NamespaceOnly {
        pub namespace: ModuleId,
    }

    #[test]
    fn serialize_event_error_internal() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Error(SignalingError::Internal),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "error",
          "payload": {
            "error": "internal"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_error_unknown_namespace() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Error(SignalingError::UnknownNamespace {
                invalid_namespace: "foo".into(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "error",
          "payload": {
            "error": "unknown_namespace",
            "invalid_namespace": "foo"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_automod() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Automod(AutomodEvent::Stopped(
                StoppedReason::SessionFinished,
            )),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "automod",
          "payload": {
            "message": "stopped",
            "reason": "session_finished"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_core() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Core(CoreEvent::ParticipantConnected {
                participant_id: ParticipantId::from_u128(0x01),
                connection_id: ConnectionId::from_u128(0x02),
                peer_data: Default::default(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "core",
          "payload": {
            "message": "participant_connected",
            "participant_id": "00000000-0000-0000-0000-000000000001",
            "connection_id": "00000000-0000-0000-0000-000000000002"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_breakout() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Breakout(BreakoutEvent::Closed),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "breakout",
          "payload": {
            "message": "closed"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_chat() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Chat(ChatEvent::ChatDisabled {
                issued_by: ParticipantId::from_u128(0x01),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "chat",
          "payload": {
            "message": "chat_disabled",
            "issued_by": "00000000-0000-0000-0000-000000000001"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_echo() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Echo(EchoEvent::Pong),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "echo",
          "payload": {
            "message": "pong"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_livekit() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::LiveKit(LiveKitEvent::ScreenSharePermissionsUpdated {
                grant: false,
                participants: BTreeSet::new(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "livekit",
          "payload": {
            "message": "screen_share_permissions_updated",
            "grant": false,
            "participants": []
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_timer() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Timer(TimerEvent::UpdatedReadyStatus {
                participant_id: ParticipantId::nil(),
                status: true,
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "timer",
          "payload": {
            "message": "updated_ready_status",
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "status": true
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_polls() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Polls(PollsEvent::Voted(Vote {
                poll_id: PollId::nil(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            })),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "polls",
          "payload": {
            "message": "voted",
            "poll_id": "00000000-0000-0000-0000-000000000000",
            "choice_id": 0
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, event.namespace());
    }

    #[test]
    fn serialize_event_meeting_report() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::MeetingReport(MeetingReportEvent::PdfAsset {
                filename: "name".into(),
                asset_id: AssetId::nil(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "meeting_report",
          "payload": {
            "message": "pdf_asset",
            "filename": "name",
            "asset_id": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_moderation() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Moderation(ModerationEvent::Accepted),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "moderation",
          "payload": {
            "message": "accepted"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_raise_hands() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::RaiseHands(RaiseHandsEvent::HandRaised {
                participant: ParticipantId::nil(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "raise_hands",
          "payload": {
            "message": "hand_raised",
            "participant": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_subroom_audio() {
        let group = WhisperGroup {
            whisper_id: WhisperId::nil(),
            participants: BTreeMap::from([
                (ParticipantId::from_u128(0), WhisperState::Creator),
                (ParticipantId::from_u128(1), WhisperState::Invited),
            ]),
        };

        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::SubroomAudio(SubroomAudioEvent::WhisperGroupCreated {
                token: "<jwt-token>".into(),
                group: group.into(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "subroom_audio",
          "payload": {
            "message": "whisper_group_created",
            "token": "<jwt-token>",
            "whisper_id": "00000000-0000-0000-0000-000000000000",
            "participants": [
              {
                "participant_id": "00000000-0000-0000-0000-000000000000",
                "state": "creator"
              },
              {
                "participant_id": "00000000-0000-0000-0000-000000000001",
                "state": "invited"
              }
            ]
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_meeting_notes() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::MeetingNotes(MeetingNotesEvent::AccessChanged {
                readers: Vec::new(),
                writers: Vec::new(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "meeting_notes",
          "payload": {
            "message": "access_changed",
            "readers": [],
            "writers": []
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_whiteboard() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Whiteboard(WhiteboardEvent::Initialized {
                url: Url::parse("https://example.com").unwrap(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "whiteboard",
          "payload": {
            "message": "initialized",
            "url": "https://example.com/"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_excalidraw() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Excalidraw(ExcalidrawEvent::Followed {
                participant_id: ParticipantId::nil(),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "excalidraw",
          "payload": {
            "message": "followed",
            "participant_id": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_legal_vote() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::LegalVote(LegalVoteEvent::Voted {
                legal_vote_id: LegalVoteId::from_u128(1),
                vote_option: VoteOption::Yes,
                issuer: ParticipantId::from_u128(2),
                consumed_token: Token::new(3),
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "legal_vote",
          "payload": {
            "message": "voted",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "vote_option": "yes",
            "issuer": "00000000-0000-0000-0000-000000000002",
            "consumed_token": "11111111114"
          }
        }
        "#);
    }

    #[test]
    fn serialize_training_participation_report() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::TrainingParticipationReport(
                TrainingParticipationReportEvent::PresenceConfirmationLogged,
            ),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "training_participation_report",
          "payload": {
            "message": "presence_confirmation_logged"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_transcription() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Transcription(TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Running {
                    language: Some("en".into()),
                },
            }),
        };

        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "transcription",
          "payload": {
            "message": "state_updated",
            "status": "running",
            "language": "en"
          }
        }
        "#);
    }

    #[test]
    fn serialize_event_reaction() {
        let event = SignalingEvent {
            transaction_id: None,
            timestamp: Timestamp::unix_epoch(),
            payload: SignalingModuleEvent::Reaction(ReactionEvent::Reacted {
                participant_id: ParticipantId::nil(),
                reaction: opentalk_roomserver_types_reaction::Reaction::ThumbsUp,
            }),
        };

        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00Z",
          "namespace": "reaction",
          "payload": {
            "message": "reacted",
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "reaction": "thumbs_up"
          }
        }
        "#);
    }
}
