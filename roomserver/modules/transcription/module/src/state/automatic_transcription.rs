// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, HashSet};

use opentalk_report_generation::{ReportDateTime, ToReportDateTime};
use opentalk_roomserver_signaling::module_context::ModuleContext;
use opentalk_roomserver_types_transcription::{
    config::AutomaticTranscriptionConfig, segment::TranscriptionSegment,
    status::AutomaticTranscriptionMode,
};
use opentalk_types_common::{
    time::{TimeZone, Timestamp},
    users::DisplayName,
};
use opentalk_types_signaling::{ParticipantId, ParticipationVisibility};
use serde::Serialize;

use crate::TranscriptionModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomaticTranscription {
    /// The configuration of the automatic transcription.
    config: AutomaticTranscriptionConfig,
    /// The time zone used when rendering the transcript.
    time_zone: TimeZone,
    /// The time the automatic transcription started.
    started_at: ReportDateTime,
    /// Whether the automatic transcription is currently paused.
    pub(super) paused: bool,
    /// The recorded entries of the automatic transcription.
    transcript: Vec<TranscriptEntry>,
    /// The pseudonyms assigned to participants in the order they first spoke.
    ///
    /// Only populated when [`AutomaticTranscriptionConfig::pseudonymous`] is `true`.
    pseudonyms: HashMap<ParticipantId, usize>,
    /// Set of participants which consented to being included in the automatic transcription.
    pub consenting_participants: HashSet<ParticipantId>,
    /// Set of participants that were present in the room while the automatic transcription
    /// was running.
    pub seen_participants: HashSet<ParticipantId>,
    /// Set of moderators that were present in the room while the automatic transcription was
    /// running.
    pub seen_moderators: HashSet<ParticipantId>,
}

impl AutomaticTranscription {
    pub fn new(
        ctx: &ModuleContext<'_, TranscriptionModule>,
        config: AutomaticTranscriptionConfig,
        time_zone: TimeZone,
    ) -> Self {
        let now = Timestamp::now().to_report_date_time(&time_zone);
        let seen_participants = ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .visibility(ParticipationVisibility::Visible)
            .ids()
            .collect();
        let seen_moderators = ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .visibility(ParticipationVisibility::Visible)
            .moderators()
            .ids()
            .collect();

        Self {
            config,
            time_zone,
            started_at: now,
            paused: false,
            transcript: vec![TranscriptEntry {
                timestamp: now,
                event: TranscriptEvent::Start,
            }],
            pseudonyms: HashMap::new(),
            consenting_participants: HashSet::new(),
            seen_participants,
            seen_moderators,
        }
    }

    fn pseudonym(&mut self, participant_id: ParticipantId) -> usize {
        // Pseudonyms are user facing so they start at 1.
        let next = self.pseudonyms.len() + 1;

        *self.pseudonyms.entry(participant_id).or_insert(next)
    }

    fn speaker(
        &mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
        participant_id: ParticipantId,
    ) -> Speaker {
        if self.config.pseudonymous {
            let number = self.pseudonym(participant_id);

            Speaker::Pseudonym { number }
        } else {
            ctx.participant_state(participant_id)
                .map(|state| Speaker::Name {
                    name: state.display_name.clone(),
                })
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "Received transcription segment from unknown speaker {participant_id}"
                    );
                    Speaker::Unknown { participant_id }
                })
        }
    }

    fn ongoing_utterance(&mut self, speaker: &Speaker) -> Option<&mut String> {
        if let TranscriptEvent::Text {
            speaker: last_speaker,
            text,
        } = &mut self.transcript.last_mut()?.event
            && last_speaker == speaker
        {
            Some(text)
        } else {
            None
        }
    }

    pub fn push_segment(
        &mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
        segment: &TranscriptionSegment,
    ) {
        let speaker = self.speaker(ctx, segment.participant_id);
        // If the last speaker is the same as the current one, we merge the segments into one entry.
        if let Some(text) = self.ongoing_utterance(&speaker) {
            text.push(' ');
            text.push_str(&segment.text);
        } else {
            self.push_event(TranscriptEvent::Text {
                speaker,
                text: segment.text.to_owned(),
            });
        }
    }

    /// Pauses the automatic transcription and records a [`TranscriptEvent::Pause`] entry.
    ///
    /// Returns `false` if the automatic transcription was already paused.
    pub(super) fn pause(&mut self) -> bool {
        if self.paused {
            return false;
        }

        self.paused = true;
        self.push_event(TranscriptEvent::Pause);

        true
    }

    /// Resumes the automatic transcription and records a [`TranscriptEvent::Resume`] entry.
    ///
    /// Returns `false` if the automatic transcription was not paused.
    pub(super) fn resume(&mut self) -> bool {
        if !self.paused {
            return false;
        }

        self.paused = false;
        self.push_event(TranscriptEvent::Resume);

        true
    }

    pub fn stop(
        mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
    ) -> FinishedAutomaticTranscription {
        let moderators = std::mem::take(&mut self.seen_moderators)
            .into_iter()
            .map(|id| self.speaker(ctx, id))
            .collect();

        let Self {
            config,
            time_zone,
            started_at,
            paused: _,
            mut transcript,
            pseudonyms: _,
            consenting_participants: _,
            seen_participants,
            seen_moderators: _,
        } = self;
        let now = Timestamp::now().to_report_date_time(&time_zone);
        transcript.push(TranscriptEntry {
            timestamp: now,
            event: TranscriptEvent::End,
        });

        FinishedAutomaticTranscription {
            started_at,
            ended_at: now,
            present_participant_count: seen_participants.len(),
            moderators,
            pseudonymous: config.pseudonymous,
            transcript,
        }
    }

    fn push_event(&mut self, event: TranscriptEvent) {
        self.transcript.push(TranscriptEntry {
            timestamp: Timestamp::now().to_report_date_time(&self.time_zone),
            event,
        });
    }

    pub fn config(&self) -> AutomaticTranscriptionConfig {
        self.config
    }

    pub fn paused(&self) -> bool {
        self.paused
    }
}

impl From<&AutomaticTranscription> for AutomaticTranscriptionMode {
    fn from(value: &AutomaticTranscription) -> Self {
        Self {
            config: value.config(),
            paused: value.paused(),
        }
    }
}

#[derive(Debug)]
pub struct FinishedAutomaticTranscription {
    pub started_at: ReportDateTime,
    pub ended_at: ReportDateTime,
    pub present_participant_count: usize,
    pub moderators: Vec<Speaker>,
    pub pseudonymous: bool,
    pub transcript: Vec<TranscriptEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptEntry {
    /// The time when the event happened.
    pub timestamp: ReportDateTime,

    /// The event that occurred in the automatic transcription.
    pub event: TranscriptEvent,
}

/// An event that can occur in an automatic transcription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptEvent {
    /// The automatic transcription has started.
    Start,
    /// The automatic transcription was paused.
    Pause,
    /// The automatic transcription was resumed.
    Resume,
    /// The automatic transcription has ended.
    End,
    /// A participant spoke.
    Text {
        /// The speaker of the text.
        speaker: Speaker,
        /// The text that was spoken.
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Speaker {
    Name {
        name: DisplayName,
    },
    Pseudonym {
        number: usize,
    },
    Unknown {
        // This field ensures that two different unknown speakers are not considered equal and is
        // not included in the rendered transcript.
        participant_id: ParticipantId,
    },
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn serialize_transcript_entry() {
        let entry = TranscriptEntry {
            timestamp: Timestamp::unix_epoch().to_report_date_time(&TimeZone::utc()),
            event: TranscriptEvent::Text {
                speaker: Speaker::Name {
                    name: DisplayName::from_str_lossy("Alice"),
                },
                text: "Hello world".to_string(),
            },
        };
        let raw = serde_json::to_string_pretty(&entry).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "timestamp": "1970-01-01T00:00:00",
          "event": {
            "kind": "text",
            "speaker": {
              "kind": "name",
              "name": "Alice"
            },
            "text": "Hello world"
          }
        }
        "#);
    }

    #[test]
    fn serialize_start() {
        let event = TranscriptEvent::Start;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "start"
        }
        "#);
    }

    #[test]
    fn serialize_pause() {
        let event = TranscriptEvent::Pause;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "pause"
        }
        "#);
    }

    #[test]
    fn serialize_resume() {
        let event = TranscriptEvent::Resume;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "resume"
        }
        "#);
    }

    #[test]
    fn serialize_end() {
        let event = TranscriptEvent::End;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "end"
        }
        "#);
    }

    #[test]
    fn serialize_text() {
        let event = TranscriptEvent::Text {
            speaker: Speaker::Name {
                name: DisplayName::from_str_lossy("Alice"),
            },
            text: "Hello world".to_string(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "text",
          "speaker": {
            "kind": "name",
            "name": "Alice"
          },
          "text": "Hello world"
        }
        "#);
    }

    #[test]
    fn serialize_speaker_name() {
        let speaker = Speaker::Name {
            name: DisplayName::from_str_lossy("Alice"),
        };
        let raw = serde_json::to_string_pretty(&speaker).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "name",
          "name": "Alice"
        }
        "#);
    }

    #[test]
    fn serialize_speaker_pseudonym() {
        let speaker = Speaker::Pseudonym { number: 1 };
        let raw = serde_json::to_string_pretty(&speaker).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "pseudonym",
          "number": 1
        }
        "#);
    }

    #[test]
    fn serialize_speaker_unknown() {
        let speaker = Speaker::Unknown {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&speaker).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "unknown",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }
}
