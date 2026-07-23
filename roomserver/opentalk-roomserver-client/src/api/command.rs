// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types_recording::{RECORDING_MODULE_ID, command::RecordingCommand};
use opentalk_roomserver_types_transcription::{
    TRANSCRIPTION_MODULE_ID, command::TranscriptionCommand,
};
use opentalk_types_common::modules::ModuleId;
use serde::{Deserialize, Serialize};
// reexport commands for easier usage
pub use {
    opentalk_roomserver_types::{
        breakout::{BREAKOUT_MODULE_ID, command::BreakoutCommand},
        core::{CORE_MODULE_ID, CoreCommand},
    },
    opentalk_roomserver_types_automod::{AUTOMOD_MODULE_ID, command::AutomodCommand},
    opentalk_roomserver_types_chat::{CHAT_MODULE_ID, command::ChatCommand},
    opentalk_roomserver_types_e2ee::{E2EE_MODULE_ID, E2eeCommand},
    opentalk_roomserver_types_echo::{ECHO_MODULE_ID, command::EchoCommand},
    opentalk_roomserver_types_excalidraw::{EXCALIDRAW_MODULE_ID, ExcalidrawCommand},
    opentalk_roomserver_types_legal_vote::{LEGAL_VOTE_MODULE_ID, LegalVoteCommand},
    opentalk_roomserver_types_livekit::{
        LIVEKIT_MODULE_ID, LiveKitCommand, MicrophoneRestrictionState,
    },
    opentalk_roomserver_types_meeting_notes::{MEETING_NOTES_MODULE_ID, MeetingNotesCommand},
    opentalk_roomserver_types_meeting_report::{
        MEETING_REPORT_MODULE_ID, command::MeetingReportCommand,
    },
    opentalk_roomserver_types_moderation::{MODERATION_MODULE_ID, command::ModerationCommand},
    opentalk_roomserver_types_polls::{POLLS_MODULE_ID, command::PollsCommand},
    opentalk_roomserver_types_raise_hands::{RAISE_HANDS_MODULE_ID, command::RaiseHandsCommand},
    opentalk_roomserver_types_reaction::{REACTION_MODULE_ID, ReactionCommand},
    opentalk_roomserver_types_shared_folder::{
        SHARED_FOLDER_MODULE_ID, command::SharedFolderCommand,
    },
    opentalk_roomserver_types_subroom_audio::{
        SUBROOM_AUDIO_MODULE_ID, command::SubroomAudioCommand,
    },
    opentalk_roomserver_types_timer::{TIMER_MODULE_ID, TimerCommand},
    opentalk_roomserver_types_training_participation_report::{
        TRAINING_PARTICIPATION_REPORT_MODULE_ID, command::TrainingParticipationReportCommand,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalingCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    #[serde(flatten)]
    pub payload: SignalingModuleCommand,
}

impl SignalingCommand {
    pub fn namespace(&self) -> ModuleId {
        self.payload.namespace()
    }
}

impl From<SignalingModuleCommand> for SignalingCommand {
    fn from(payload: SignalingModuleCommand) -> Self {
        Self {
            transaction_id: None,
            payload,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
pub enum SignalingModuleCommand {
    Core(CoreCommand),
    Automod(AutomodCommand),
    Breakout(BreakoutCommand),
    Echo(EchoCommand),
    Chat(ChatCommand),

    #[serde(rename = "livekit")]
    LiveKit(LiveKitCommand),

    E2ee(E2eeCommand),

    Timer(TimerCommand),
    Polls(PollsCommand),
    SharedFolder(SharedFolderCommand),
    MeetingReport(MeetingReportCommand),
    Moderation(ModerationCommand),
    RaiseHands(RaiseHandsCommand),
    SubroomAudio(SubroomAudioCommand),
    MeetingNotes(MeetingNotesCommand),
    Excalidraw(ExcalidrawCommand),
    LegalVote(LegalVoteCommand),
    TrainingParticipationReport(TrainingParticipationReportCommand),
    Recording(RecordingCommand),
    Transcription(TranscriptionCommand),
    Reaction(ReactionCommand),
}

impl SignalingModuleCommand {
    pub fn namespace(&self) -> ModuleId {
        match self {
            Self::Core(..) => CORE_MODULE_ID,
            Self::Automod(..) => AUTOMOD_MODULE_ID,
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
    use std::collections::BTreeSet;

    use insta::assert_snapshot;
    use opentalk_roomserver_types::{breakout::command::BreakoutCommand, core::CoreCommand};
    use opentalk_roomserver_types_automod::command::AutomodCommand;
    use opentalk_roomserver_types_chat::command::ChatCommand;
    use opentalk_roomserver_types_echo::command::EchoCommand;
    use opentalk_roomserver_types_excalidraw::ExcalidrawCommand;
    use opentalk_roomserver_types_legal_vote::{
        LegalVoteCommand, cancel::CustomCancelReason, vote::LegalVoteId,
    };
    use opentalk_roomserver_types_livekit::LiveKitCommand;
    use opentalk_roomserver_types_meeting_notes::MeetingNotesCommand;
    use opentalk_roomserver_types_meeting_report::command::MeetingReportCommand;
    use opentalk_roomserver_types_moderation::command::ModerationCommand;
    use opentalk_roomserver_types_polls::{
        ChoiceId, PollId,
        command::{Choices, PollsCommand, Vote},
    };
    use opentalk_roomserver_types_raise_hands::command::RaiseHandsCommand;
    use opentalk_roomserver_types_reaction::ReactionCommand;
    use opentalk_roomserver_types_recording::command::RecordingCommand;
    use opentalk_roomserver_types_timer::{TimerCommand, command::Kind};
    use opentalk_roomserver_types_training_participation_report::TrainingParticipationReportCommand;
    use opentalk_roomserver_types_transcription::command::TranscriptionCommand;
    use opentalk_types_common::modules::ModuleId;
    use opentalk_types_signaling::ParticipantId;
    use serde::Deserialize;

    use super::SignalingModuleCommand;
    use crate::api::command::SignalingCommand;

    #[derive(Debug, Clone, Deserialize)]
    pub struct NamespaceOnly {
        pub namespace: ModuleId,
    }

    #[test]
    fn serialize_command_core() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Core(CoreCommand::EnterRoom { display_name: None }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();
        assert_snapshot!(raw, @r#"
        {
          "namespace": "core",
          "payload": {
            "action": "enter_room"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_automod() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Automod(AutomodCommand::Stop),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "automod",
          "payload": {
            "action": "stop"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_breakout() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Breakout(BreakoutCommand::Stop { delay: None }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "breakout",
          "payload": {
            "action": "stop"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_chat() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Chat(ChatCommand::DisableChat),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "chat",
          "payload": {
            "action": "disable_chat"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_echo() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Echo(EchoCommand::Ping),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "echo",
          "payload": {
            "action": "ping"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_livekit() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::LiveKit(LiveKitCommand::CreateNewAccessToken),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "livekit",
          "payload": {
            "action": "create_new_access_token"
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_timer() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Timer(TimerCommand::Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "timer",
          "payload": {
            "action": "start",
            "kind": "stopwatch",
            "style": null,
            "title": null,
            "enable_ready_check": false
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_polls() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Polls(PollsCommand::Vote(Vote {
                poll_id: PollId::nil(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            })),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "polls",
          "payload": {
            "action": "vote",
            "poll_id": "00000000-0000-0000-0000-000000000000",
            "choice_id": 0
          }
        }
        "#);

        // Check that the ModuleId from the actual SignalingModule matches what we serialize.
        let namespace_only: NamespaceOnly = serde_json::from_str(&raw).unwrap();
        assert_eq!(namespace_only.namespace, command.namespace());
    }

    #[test]
    fn serialize_command_meeting_report() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::MeetingReport(
                MeetingReportCommand::GenerateAttendanceReport {
                    include_email_addresses: false,
                },
            ),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "meeting_report",
          "payload": {
            "action": "generate_attendance_report",
            "include_email_addresses": false
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_moderation() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Moderation(ModerationCommand::Accept {
                target: ParticipantId::nil(),
            }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "moderation",
          "payload": {
            "action": "accept",
            "target": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_raise_hands() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::RaiseHands(RaiseHandsCommand::RaiseHand),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "raise_hands",
          "payload": {
            "action": "raise_hand"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_meeting_notes() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::MeetingNotes(MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::new(),
            }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "meeting_notes",
          "payload": {
            "action": "grant_write_access",
            "participant_ids": []
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_excalidraw() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Excalidraw(ExcalidrawCommand::Follow {
                participant_id: ParticipantId::nil(),
            }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "excalidraw",
          "payload": {
            "action": "follow",
            "participant_id": "00000000-0000-0000-0000-000000000000"
          }
        }
        "#);
    }

    #[test]
    fn serialize_command_legal_vote() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::LegalVote(LegalVoteCommand::Cancel {
                legal_vote_id: LegalVoteId::from_u128(1),
                reason: CustomCancelReason::try_from("Test Reason").unwrap(),
            }),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "legal_vote",
          "payload": {
            "action": "cancel",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "reason": "Test Reason"
          }
        }
        "#);
    }

    #[test]
    fn serialize_training_participation_report() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::TrainingParticipationReport(
                TrainingParticipationReportCommand::ConfirmPresence,
            ),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "training_participation_report",
          "payload": {
            "action": "confirm_presence"
          }
        }
        "#);
    }

    #[test]
    fn serialize_recording() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Recording(RecordingCommand::StartRecording),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "recording",
          "payload": {
            "action": "start_recording"
          }
        }
        "#);
    }

    #[test]
    fn serialize_transcription() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Transcription(TranscriptionCommand::Start {
                language: None,
            }),
        };

        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "transcription",
          "payload": {
            "action": "start"
          }
        }
        "#);
    }

    #[test]
    fn serialize_reaction() {
        let command = SignalingCommand {
            transaction_id: None,
            payload: SignalingModuleCommand::Reaction(ReactionCommand::React {
                reaction: opentalk_roomserver_types_reaction::Reaction::ThumbsUp,
            }),
        };

        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "namespace": "reaction",
          "payload": {
            "action": "react",
            "reaction": "thumbs_up"
          }
        }
        "#);
    }
}
