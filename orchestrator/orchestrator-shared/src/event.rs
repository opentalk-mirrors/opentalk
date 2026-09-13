// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{Metrics, RecorderEvent, RoomserverEvent, TranscriptionEvent};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Event {
    Metrics(Metrics),
    Recorder(RecorderEvent),
    Roomserver(RoomserverEvent),
    Transcription(TranscriptionEvent),
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::rooms::RoomId;
    use serde_json::json;

    use super::*;
    use crate::{RecorderResource, TranscriptionResource};

    #[test]
    fn serialize_event_metrics() {
        let event = Event::Metrics(Metrics {
            load: 42,
            accepting_jobs: true,
        });

        let serialized = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json!({
                "type": "metrics",
                "load": 42,
                "accepting_jobs": true
            }),
            serialized
        );
    }

    #[test]
    fn deserialize_event_metrics() {
        let json = json!({
            "type": "metrics",
            "load": 42,
            "accepting_jobs": true
        });

        let deserialized: Event = serde_json::from_value(json).unwrap();

        assert_eq!(
            Event::Metrics(Metrics {
                load: 42,
                accepting_jobs: true
            }),
            deserialized
        );
    }

    #[test]
    fn serialize_event_recorder() {
        let event = Event::Recorder(RecorderEvent::RemoveRecording(RecorderResource {
            room_id: RoomId::nil(),
            breakout_id: Some(42),
        }));

        let serialized = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json!({
                "type": "recorder",
                "remove_recording": {
                    "room_id": RoomId::nil(),
                    "breakout_id": Some(42)
                }
            }),
            serialized
        );
    }

    #[test]
    fn deserialize_event_recorder() {
        let json = json!({
            "type": "recorder",
            "remove_recording": {
                "room_id": RoomId::nil(),
                "breakout_id": Some(42)
            }
        });

        let deserialized: Event = serde_json::from_value(json).unwrap();

        assert_eq!(
            Event::Recorder(RecorderEvent::RemoveRecording(RecorderResource {
                room_id: RoomId::nil(),
                breakout_id: Some(42)
            })),
            deserialized
        );
    }

    #[test]
    fn serialize_event_roomserver() {
        let event = Event::Roomserver(RoomserverEvent::RemoveRoom(RoomId::nil()));

        let serialized = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json!({
                "type": "roomserver",
                "remove_room": RoomId::nil()
            }),
            serialized
        );
    }

    #[test]
    fn deserialize_event_roomserver() {
        let json = json!({
            "type": "roomserver",
            "remove_room": RoomId::nil()
        });

        let deserialized: Event = serde_json::from_value(json).unwrap();

        assert_eq!(
            Event::Roomserver(RoomserverEvent::RemoveRoom(RoomId::nil())),
            deserialized
        );
    }

    #[test]
    fn serialize_event_transcription() {
        let event = Event::Transcription(TranscriptionEvent::RemoveTranscription(
            TranscriptionResource {
                room_id: RoomId::nil(),
                breakout_id: Some(1),
            },
        ));

        let serialized = serde_json::to_value(&event).unwrap();

        assert_eq!(
            json!({
                "type": "transcription",
                "remove_transcription": {
                    "room_id": RoomId::nil(),
                    "breakout_id": 1
                }
            }),
            serialized
        );
    }

    #[test]
    fn deserialize_event_transcription() {
        let json = json!({
            "type": "transcription",
            "remove_transcription": {
                "room_id": RoomId::nil(),
                "breakout_id": null
            }
        });

        let deserialized: Event = serde_json::from_value(json).unwrap();

        assert_eq!(
            Event::Transcription(TranscriptionEvent::RemoveTranscription(
                TranscriptionResource {
                    room_id: RoomId::nil(),
                    breakout_id: None,
                },
            )),
            deserialized
        );
    }
}
