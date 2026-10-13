// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, HashSet};

use opentalk_report_generation::{ReportDateTime, ToReportDateTime};
use opentalk_roomserver_signaling::module_context::ModuleContext;
use opentalk_roomserver_types_transcription::{
    config::MeetingProtocolConfig, segment::TranscriptionSegment, status::MeetingProtocolMode,
};
use opentalk_types_common::{
    time::{TimeZone, Timestamp},
    users::DisplayName,
};
use opentalk_types_signaling::{ParticipantId, ParticipationVisibility};
use serde::Serialize;

use crate::TranscriptionModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeetingProtocol {
    /// The configuration of the meeting protocol.
    config: MeetingProtocolConfig,
    /// The time zone used when rendering the protocol
    time_zone: TimeZone,
    /// The time the meeting protocol started.
    started_at: ReportDateTime,
    /// Whether the meeting protocol is currently paused.
    pub(super) paused: bool,
    /// The recorded entries of the meeting protocol.
    protocol: Vec<ProtocolEntry>,
    /// The pseudonyms assigned to participants in the order they first spoke.
    ///
    /// Only populated when [`MeetingProtocolConfig::pseudonymous`] is `true`.
    pseudonyms: HashMap<ParticipantId, usize>,
    /// Set of participants which consented to being included in the meeting protocol.
    pub consenting_participants: HashSet<ParticipantId>,
    /// Set of participants that were present in the in the room while the protocol was
    /// transcribed.
    pub seen_participants: HashSet<ParticipantId>,
    /// Set of moderators that were present in the in the room while the protocol was transcribed.
    pub seen_moderators: HashSet<ParticipantId>,
}

impl MeetingProtocol {
    pub fn new(
        ctx: &ModuleContext<'_, TranscriptionModule>,
        config: MeetingProtocolConfig,
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
            protocol: vec![ProtocolEntry {
                timestamp: now,
                event: ProtocolEvent::Start,
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
        if let ProtocolEvent::Text {
            speaker: last_speaker,
            text,
        } = &mut self.protocol.last_mut()?.event
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
            self.push_event(ProtocolEvent::Text {
                speaker,
                text: segment.text.to_owned(),
            });
        }
    }

    /// Pauses the meeting protocol and records a [`ProtocolEvent::Pause`] entry.
    ///
    /// Returns `false` if the meeting protocol was already paused.
    pub(super) fn pause(&mut self) -> bool {
        if self.paused {
            return false;
        }

        self.paused = true;
        self.push_event(ProtocolEvent::Pause);

        true
    }

    /// Resumes the meeting protocol and records a [`ProtocolEvent::Resume`] entry.
    ///
    /// Returns `false` if the meeting protocol was not paused.
    pub(super) fn resume(&mut self) -> bool {
        if !self.paused {
            return false;
        }

        self.paused = false;
        self.push_event(ProtocolEvent::Resume);

        true
    }

    pub fn stop(mut self, ctx: &ModuleContext<'_, TranscriptionModule>) -> FinishedMeetingProtocol {
        let moderators = std::mem::take(&mut self.seen_moderators)
            .into_iter()
            .map(|id| self.speaker(ctx, id))
            .collect();

        let Self {
            config,
            time_zone,
            started_at,
            paused: _,
            mut protocol,
            pseudonyms: _,
            consenting_participants: _,
            seen_participants,
            seen_moderators: _,
        } = self;
        let now = Timestamp::now().to_report_date_time(&time_zone);
        protocol.push(ProtocolEntry {
            timestamp: now,
            event: ProtocolEvent::End,
        });

        FinishedMeetingProtocol {
            started_at,
            ended_at: now,
            present_participant_count: seen_participants.len(),
            moderators,
            pseudonymous: config.pseudonymous,
            protocol,
        }
    }

    fn push_event(&mut self, event: ProtocolEvent) {
        self.protocol.push(ProtocolEntry {
            timestamp: Timestamp::now().to_report_date_time(&self.time_zone),
            event,
        });
    }

    pub fn config(&self) -> MeetingProtocolConfig {
        self.config
    }

    pub fn paused(&self) -> bool {
        self.paused
    }
}

impl From<&MeetingProtocol> for MeetingProtocolMode {
    fn from(value: &MeetingProtocol) -> Self {
        Self {
            config: value.config(),
            paused: value.paused(),
        }
    }
}

#[derive(Debug)]
pub struct FinishedMeetingProtocol {
    pub started_at: ReportDateTime,
    pub ended_at: ReportDateTime,
    pub present_participant_count: usize,
    pub moderators: Vec<Speaker>,
    pub pseudonymous: bool,
    pub protocol: Vec<ProtocolEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolEntry {
    /// The time when the event happened.
    pub timestamp: ReportDateTime,

    /// The event that occurred in the meeting protocol.
    pub event: ProtocolEvent,
}

/// An event that can occur in a meeting protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProtocolEvent {
    /// The protocol has started.
    Start,
    /// The protocol was paused.
    Pause,
    /// The protocol was resumed.
    Resume,
    /// The protocol has ended.
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
        // not included in the rendered protocol.
        participant_id: ParticipantId,
    },
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn serialize_protocol_entry() {
        let entry = ProtocolEntry {
            timestamp: Timestamp::unix_epoch().to_report_date_time(&TimeZone::utc()),
            event: ProtocolEvent::Text {
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
        let event = ProtocolEvent::Start;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "start"
        }
        "#);
    }

    #[test]
    fn serialize_pause() {
        let event = ProtocolEvent::Pause;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "pause"
        }
        "#);
    }

    #[test]
    fn serialize_resume() {
        let event = ProtocolEvent::Resume;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "resume"
        }
        "#);
    }

    #[test]
    fn serialize_end() {
        let event = ProtocolEvent::End;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "end"
        }
        "#);
    }

    #[test]
    fn serialize_text() {
        let event = ProtocolEvent::Text {
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
