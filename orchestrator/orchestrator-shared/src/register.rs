// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_service_auth::ApiKeyId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    Metrics, RegisterRecorder, RegisterRoomserver, RegisterTranscription, error::RegistrationError,
};

/// The server response to the [`Register`] request
#[derive(Debug, Serialize, Deserialize)]
pub enum RegisterResponse {
    Success,
    Error(RegistrationError),
}

impl From<RegistrationError> for RegisterResponse {
    fn from(error: RegistrationError) -> Self {
        Self::Error(error)
    }
}

/// Request to register at the orchestrator
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Register {
    pub register_data: RegisterData,
    /// The type of service
    pub register_type: RegisterType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct RegisterData {
    /// The address of the service
    pub service_address: ServiceAddress,
    /// A list of api key ids to that authorize requests to the service
    pub api_key_ids: Vec<ApiKeyId>,
    /// The initial metrics
    pub metrics: Metrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceAddress {
    /// The service provided a full URL as its address
    Url(Url),
    /// The service only provided a port, so the orchestrator should use the client's IP address
    /// (received on registration) and the provided port to build the URL
    Port(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RegisterType {
    Recorder(RegisterRecorder),
    Roomserver(RegisterRoomserver),
    Transcription(RegisterTranscription),
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::rooms::RoomId;
    use serde_json::json;

    use super::*;
    use crate::RecorderResource;

    #[test]
    fn serialize_register_recorder() {
        let room_id = RoomId::nil();
        let breakout_id = Some(42);

        let resource = RecorderResource {
            room_id,
            breakout_id,
        };

        let register = Register {
            register_data: RegisterData {
                service_address: ServiceAddress::Port(8080),
                api_key_ids: vec![ApiKeyId("recorder".to_string())],
                metrics: Metrics {
                    load: 42,
                    accepting_jobs: true,
                },
            },
            register_type: RegisterType::Recorder(RegisterRecorder {
                rooms: [resource].into_iter().collect(),
            }),
        };

        let serialized = serde_json::to_value(&register).unwrap();

        assert_eq!(
            json!({
                "register_data": {
                    "service_address": {
                        "port": 8080
                    },
                    "api_key_ids": ["recorder"],
                    "metrics": {
                        "load": 42,
                        "accepting_jobs": true
                    }
                },
                "register_type": {
                    "recorder": {
                        "rooms": [
                            {
                                "room_id": room_id,
                                "breakout_id": breakout_id
                            }
                        ]
                    }
                }
            }),
            serialized
        );
    }

    #[test]
    fn deserialize_register_recorder() {
        let room_id = RoomId::nil();
        let breakout_id = Some(42);

        let json = json!({
            "register_data": {
                "service_address": {
                    "port": 8080
                },
                "api_key_ids": ["recorder"],
                "metrics": {
                    "load": 42,
                    "accepting_jobs": true
                }
            },
            "register_type": {
                "recorder": {
                    "rooms": [
                        {
                            "room_id": room_id,
                            "breakout_id": breakout_id
                        }
                    ]
                }
            }
        });

        let deserialized: Register = serde_json::from_value(json).unwrap();

        assert_eq!(
            deserialized,
            Register {
                register_data: RegisterData {
                    service_address: ServiceAddress::Port(8080),
                    api_key_ids: vec![ApiKeyId("recorder".to_string())],
                    metrics: Metrics {
                        load: 42,
                        accepting_jobs: true
                    }
                },
                register_type: RegisterType::Recorder(RegisterRecorder {
                    rooms: [RecorderResource {
                        room_id,
                        breakout_id
                    }]
                    .into_iter()
                    .collect(),
                }),
            }
        );
    }
}
