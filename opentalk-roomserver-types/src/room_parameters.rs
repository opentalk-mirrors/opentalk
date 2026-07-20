// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Debug, str::FromStr, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use icu_locid::{LanguageIdentifier, langid};
use opentalk_types_common::{
    call_in::CallInInfo,
    events::{EventDescription, EventId, EventTitle},
    rooms::{RoomPassword, invite_codes::InviteCode},
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    tariffs::QuotaType,
    time::Timestamp,
    utils::ExampleData,
};
use serde::{Deserialize, Serialize};

use crate::{
    client_parameters::ClientKind, module_settings::ModuleSettings,
    public_user_profile::PublicUserProfile, rate_limit::RateLimitSettings,
    tariff_details::TariffDetails,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomParameters::example_data())))]
pub struct RoomParameters {
    pub created_by: PublicUserProfile,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// When `true`, guests are allowed to participate in the meeting.
    pub guest_access: bool,

    /// Defines who has to wait in the waiting room until admitted by a moderator.
    pub waiting_room: WaitingRoom,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub event: Option<EventContext>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub invite_code: Option<InviteCode>,

    pub tariff: TariffDetails,

    pub streaming_targets: Vec<RoomStreamingTarget>,

    /// When true the meeting details are visible to all participants. When false, they are visible
    /// to moderators only.
    pub show_meeting_details: bool,

    /// Indicates whether the meeting room should have e2e encryption enabled.
    pub e2e_encryption: bool,

    /// Additional configuration options that are used by modules during initialization.
    pub module_settings: ModuleSettings,

    /// The preferred language of the room.
    #[cfg_attr(feature = "utoipa", schema(value_type = String, example = "de"))]
    pub preferred_language: LanguageIdentifier,

    /// Fallback language to be used for localization purposes when the default language is not
    /// available.
    #[cfg_attr(feature = "utoipa", schema(value_type = String, example = "en"))]
    pub fallback_language: LanguageIdentifier,

    /// WebSocket rate limit settings of the room. No rate limiting is applied if the field is set
    /// to [`None`].
    pub ws_rate_limit: Option<RateLimitSettings>,

    /// Allowed Origins for Cross-Origin Resource Sharing (CORS) when accessing the roomserver's
    /// HTTP API.
    pub allowed_origins: Vec<String>,

    /// The duration in milliseconds after which a room without participants is closed.
    #[serde(with = "crate::duration_ms", default = "default_idle_timeout")]
    #[cfg_attr(feature = "utoipa", schema(value_type = u64, format = "uint64"))]
    pub room_idle_timeout: Duration,
}

/// The default timeout for an empty room
///
/// Should be higher than the lifetime of the signaling token from the token store to ensure that
/// the room doesn't expire before the signaling token does.
const fn default_idle_timeout() -> Duration {
    Duration::from_mins(1)
}

impl RoomParameters {
    pub fn calc_time_limit_quota(&self, start_time: Timestamp) -> Option<Timestamp> {
        let remaining_secs = self
            .tariff
            .quota(&QuotaType::RoomTimeLimitSecs)
            .map(|secs| i64::try_from(secs).unwrap_or(i64::MAX))?;

        start_time
            .checked_add_signed(TimeDelta::seconds(remaining_secs))
            .map(Timestamp::from)
    }
}

impl Debug for RoomParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomParameters")
            .field("created_by", &self.created_by)
            .field("password", &"<REDACTED>")
            .field("waiting_room", &self.waiting_room)
            .field("call_in", &self.call_in)
            .field("event", &self.event)
            .field("invite_code", &self.invite_code)
            .field("tariff", &self.tariff)
            .field("streaming_targets", &self.streaming_targets)
            .field("e2e_encryption", &self.e2e_encryption)
            .field("module_data", &self.module_settings)
            .finish()
    }
}

impl ExampleData for RoomParameters {
    fn example_data() -> Self {
        Self {
            created_by: PublicUserProfile::example_data(),
            password: Some(RoomPassword::from_str("1234").unwrap()),
            guest_access: false,
            waiting_room: WaitingRoom::Disabled,
            call_in: Some(CallInInfo::example_data()),
            event: Some(EventContext::example_data()),
            invite_code: Some(InviteCode::example_data()),
            tariff: TariffDetails::example_data(),
            streaming_targets: vec![RoomStreamingTarget::example_data()],
            show_meeting_details: true,
            e2e_encryption: false,
            module_settings: ModuleSettings::example_data(),
            preferred_language: langid!("de"),
            fallback_language: langid!("en"),
            ws_rate_limit: Some(RateLimitSettings::example_data()),
            allowed_origins: vec!["https://example.com".to_string()],
            room_idle_timeout: Duration::from_mins(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(EventContext::example_data())))]
pub struct EventContext {
    pub id: EventId,

    pub title: EventTitle,

    pub description: EventDescription,

    pub is_adhoc: bool,

    pub starts_at: Option<DateTime<Utc>>,

    pub ends_at: Option<DateTime<Utc>>,

    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,
}

impl ExampleData for EventContext {
    fn example_data() -> Self {
        Self {
            id: EventId::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            is_adhoc: false,
            starts_at: Some(DateTime::UNIX_EPOCH),
            ends_at: Some(DateTime::UNIX_EPOCH + chrono::Duration::hours(1)),
            shared_folder: Some(SharedFolder::example_data()),
        }
    }
}

// Ordering is from least to most strict, which the functions of this enum depends on.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum WaitingRoom {
    /// No users have to go to the waiting room
    Disabled = 0,
    /// Guests have to go through the waiting room, registered users are allowed to skip
    ForGuests = 1,
    /// Everyone has to go through the waiting room (except moderators).
    ForEveryone = 2,
}

impl WaitingRoom {
    /// Does a participant of `kind` have to be placed in the waiting room?
    ///
    /// Does not take moderator rights into account.
    pub fn applies_to(self, kind: &ClientKind) -> bool {
        Self::min_strictness(kind).is_some_and(|min| self >= min)
    }

    /// Returns this setting raised just enough to place `kind` in the waiting room.
    ///
    /// Unchanged if `self` already catches `kind`, or if `kind` never enters the waiting room.
    pub fn raise_to_include(self, kind: &ClientKind) -> Self {
        match Self::min_strictness(kind) {
            Some(min_required) => self.max(min_required),
            None => self,
        }
    }

    /// The minimum required waiting room strictness for a client to be placed in the waiting room.
    fn min_strictness(kind: &ClientKind) -> Option<WaitingRoom> {
        match kind {
            ClientKind::Registered { .. } => Some(WaitingRoom::ForEveryone),
            ClientKind::Guest { .. }
            | ClientKind::CallIn { .. }
            | ClientKind::RegisteredCallIn { .. } => Some(WaitingRoom::ForGuests),
            ClientKind::Recorder { .. } | ClientKind::Transcription { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::{users::DisplayName, utils::ExampleData};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{public_user_profile::PublicUserProfile, room_kind::RoomKind};

    #[test]
    fn room_parameters() {
        let params = RoomParameters::example_data();
        let json = json!({
            "created_by": {
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                "display_name": "Alice Adams",
                "email": "alice@example.com",
                "firstname": "Alice",
                "id": "00000000-0000-0000-0000-0000000a11c3",
                "lastname": "Adams",
                "title": "",
                "timezone": "Europe/Berlin"
            },
            "password": "1234",
            "call_in": {
                "tel": "+555-123-456-789",
                "id": "1234567890",
                "password": "0987654321"
            },
            "guest_access": false,
            "waiting_room": "disabled",
            "event": {
                "description": "The Weekly Team Event",
                "id": "00000000-0000-0000-0000-004433221100",
                "is_adhoc": false,
                "starts_at": "1970-01-01T00:00:00Z",
                "ends_at": "1970-01-01T01:00:00Z",
                "shared_folder": {
                    "read": {
                        "password": "v3rys3cr3t",
                        "url": "https://cloud.example.com/shares/abc123"
                    }
                },
                "title": "Team Event"
            },
            "invite_code": "00000000-0000-0000-0000-0000deadbeef",
            "tariff": {
                "id": "00000000-0000-0000-0000-000000000000",
                "name": "Starter tariff",
                "quotas": {
                    "max_storage": 50000
                },
                "used_quota": {
                    "max_storage": 20000
                },
                "disabled_features": ["recording::record"],
            },
            "show_meeting_details": true,
            "streaming_targets": [
                {
                    "id": "00000000-0000-0000-0000-000043434343",
                    "kind": "custom",
                    "name": "Example Stream",
                    "public_url": "https://streaming.example.com/livestream123",
                    "streaming_endpoint": "https://ingress.streaming.example.com/",
                    "streaming_key": "aabbccddeeff",
                }
            ],
            "e2e_encryption": false,
            "module_settings": {
                "livekit":  {
                    "api_key": "devkey",
                    "api_secret": "secret",
                    "public_url": "http://localhost:7880",
                    "service_url": "http://localhost:7880"
                }
            },
            "preferred_language": "de",
            "fallback_language": "en",
            "ws_rate_limit": {
                "tokens_per_second": 10,
                "token_bucket_size": 30,
            },
            "allowed_origins": ["https://example.com"],
            "room_idle_timeout": 60 * 1000,
        });

        // serialization
        assert_eq!(json.clone(), serde_json::to_value(params.clone()).unwrap());

        // deserialization
        assert_eq!(params, serde_json::from_value(json).unwrap(),);
    }

    #[test]
    fn serialize_waiting_room_disabled() {
        let waiting_room = WaitingRoom::Disabled;
        let produced = serde_json::to_string_pretty(&waiting_room).unwrap();

        assert_snapshot!(produced, @r#""disabled""#);
    }

    #[test]
    fn deserialize_waiting_room_disabled() {
        let json = json!("disabled");
        let produced: WaitingRoom = serde_json::from_value(json).unwrap();

        assert_eq!(WaitingRoom::Disabled, produced);
    }

    #[test]
    fn serialize_waiting_room_for_guests() {
        let waiting_room = WaitingRoom::ForGuests;
        let produced = serde_json::to_string_pretty(&waiting_room).unwrap();

        assert_snapshot!(produced, @r#""for_guests""#);
    }

    #[test]
    fn deserialize_waiting_room_for_guests() {
        let json = json!("for_guests");
        let produced: WaitingRoom = serde_json::from_value(json).unwrap();

        assert_eq!(WaitingRoom::ForGuests, produced);
    }

    #[test]
    fn serialize_waiting_room_for_everyone() {
        let waiting_room = WaitingRoom::ForEveryone;
        let produced = serde_json::to_string_pretty(&waiting_room).unwrap();

        assert_snapshot!(produced, @r#""for_everyone""#);
    }

    #[test]
    fn deserialize_waiting_room_for_everyone() {
        let json = json!("for_everyone");
        let produced: WaitingRoom = serde_json::from_value(json).unwrap();

        assert_eq!(WaitingRoom::ForEveryone, produced);
    }

    #[test]
    fn event_info() {
        let info = EventContext::example_data();
        let json = json!({
            "id": "00000000-0000-0000-0000-004433221100",
            "title": "Team Event",
            "description": "The Weekly Team Event",
            "is_adhoc": false,
            "starts_at": "1970-01-01T00:00:00Z",
            "ends_at": "1970-01-01T01:00:00Z",
            "shared_folder": {
                "read": {
                    "password": "v3rys3cr3t",
                    "url": "https://cloud.example.com/shares/abc123",
                }
            },
        });

        // serialization
        assert_eq!(json.clone(), serde_json::to_value(info.clone()).unwrap());

        // deserialization
        assert_eq!(info, serde_json::from_value(json).unwrap());
    }

    fn registered() -> ClientKind {
        ClientKind::Registered {
            profile: PublicUserProfile::example_data(),
        }
    }

    fn guest() -> ClientKind {
        ClientKind::Guest {
            display_name: DisplayName::from_str_lossy("Guest"),
        }
    }

    fn call_in() -> ClientKind {
        ClientKind::CallIn {
            display_name: DisplayName::from_str_lossy("1001"),
        }
    }

    fn registered_call_in() -> ClientKind {
        ClientKind::RegisteredCallIn {
            profile: PublicUserProfile::example_data(),
        }
    }

    fn recorder() -> ClientKind {
        ClientKind::Recorder {
            room: RoomKind::Main,
        }
    }

    fn transcription() -> ClientKind {
        ClientKind::Transcription {
            room: RoomKind::Main,
        }
    }

    #[test]
    fn applies_to_disabled() {
        let waiting_room = WaitingRoom::Disabled;

        assert!(!waiting_room.applies_to(&registered()));
        assert!(!waiting_room.applies_to(&guest()));
        assert!(!waiting_room.applies_to(&call_in()));
        assert!(!waiting_room.applies_to(&registered_call_in()));
        assert!(!waiting_room.applies_to(&recorder()));
        assert!(!waiting_room.applies_to(&transcription()));
    }

    #[test]
    fn applies_to_for_guests() {
        let waiting_room = WaitingRoom::ForGuests;

        assert!(!waiting_room.applies_to(&registered()));
        assert!(waiting_room.applies_to(&guest()));
        assert!(waiting_room.applies_to(&call_in()));
        assert!(waiting_room.applies_to(&registered_call_in()));
        assert!(!waiting_room.applies_to(&recorder()));
        assert!(!waiting_room.applies_to(&transcription()));
    }

    #[test]
    fn applies_to_for_everyone() {
        let waiting_room = WaitingRoom::ForEveryone;

        assert!(waiting_room.applies_to(&registered()));
        assert!(waiting_room.applies_to(&guest()));
        assert!(waiting_room.applies_to(&call_in()));
        assert!(waiting_room.applies_to(&registered_call_in()));
        assert!(!waiting_room.applies_to(&recorder()));
        assert!(!waiting_room.applies_to(&transcription()));
    }

    #[test]
    fn raise_to() {
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&registered()),
            WaitingRoom::ForEveryone
        );
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&guest()),
            WaitingRoom::ForGuests
        );
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&call_in()),
            WaitingRoom::ForGuests
        );
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&registered_call_in()),
            WaitingRoom::ForGuests
        );
        // Services don't raise the waiting room, as they are never subject to it
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&recorder()),
            WaitingRoom::Disabled
        );
        assert_eq!(
            WaitingRoom::Disabled.raise_to_include(&transcription()),
            WaitingRoom::Disabled
        );
    }
}
