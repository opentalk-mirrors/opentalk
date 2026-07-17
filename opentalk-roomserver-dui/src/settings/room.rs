// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use icu_locid::langid;
use opentalk_roomserver_client::api::{
    command::{
        AUTOMOD_MODULE_ID, ECHO_MODULE_ID, EXCALIDRAW_MODULE_ID, LEGAL_VOTE_MODULE_ID,
        MEETING_NOTES_MODULE_ID, MEETING_REPORT_MODULE_ID, RAISE_HANDS_MODULE_ID,
        REACTION_MODULE_ID, SUBROOM_AUDIO_MODULE_ID, WHITEBOARD_MODULE_ID,
    },
    event::{E2EE_MODULE_ID, SHARED_FOLDER_MODULE_ID},
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters, Role},
    module_settings::ModuleSettings,
    public_user_profile::PublicUserProfile,
    room_parameters::{EventContext, RoomParameters, WaitingRoom},
    tariff_details::TariffDetails,
};
use opentalk_roomserver_types_chat::CHAT_MODULE_ID;
use opentalk_roomserver_types_moderation::MODERATION_MODULE_ID;
use opentalk_types_common::{
    events::EventId,
    shared_folders::{SharedFolder, SharedFolderAccess},
    tariffs::{QuotaType, TariffId},
    time::TimeZone,
    users::{UserId, UserInfo},
    utils::ExampleData,
};

pub fn alice_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xa11ce),
        email: "alice@example.com".to_string(),
        user_info: UserInfo {
            title: "M.Sc.".parse().expect("Valid title"),
            firstname: "Alice".to_string(),
            lastname: "Aal".to_string(),
            display_name: "Alice the angry".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-alice".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn bob_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xb0b),
        email: "bob@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("Valid title"),
            firstname: "Bob".to_string(),
            lastname: "Barsch".to_string(),
            display_name: "Bob the bold".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-bob".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn default_room_parameters() -> RoomParameters {
    let mut module_settings = ModuleSettings::example_data();
    module_settings.insert_empty(AUTOMOD_MODULE_ID);
    module_settings.insert_empty(ECHO_MODULE_ID);
    module_settings.insert_empty(CHAT_MODULE_ID);
    module_settings.insert_empty(LEGAL_VOTE_MODULE_ID);
    module_settings.insert_empty(E2EE_MODULE_ID);
    module_settings.insert_empty(AUTOMOD_MODULE_ID);
    module_settings.insert_empty(CHAT_MODULE_ID);
    module_settings.insert_empty(SHARED_FOLDER_MODULE_ID);
    module_settings.insert_empty(MEETING_REPORT_MODULE_ID);
    module_settings.insert_empty(MODERATION_MODULE_ID);
    module_settings.insert_empty(RAISE_HANDS_MODULE_ID);
    module_settings.insert_empty(SUBROOM_AUDIO_MODULE_ID);
    module_settings.insert_empty(MEETING_NOTES_MODULE_ID);
    module_settings.insert_empty(WHITEBOARD_MODULE_ID);
    module_settings.insert_empty(EXCALIDRAW_MODULE_ID);
    module_settings.insert_empty(REACTION_MODULE_ID);

    RoomParameters {
        created_by: alice_profile(),
        password: None,
        guest_access: true,
        waiting_room: WaitingRoom::Disabled,
        call_in: None,
        event: Some(EventContext {
            id: EventId::from_u128(0xbdc9186e_ccdd_468a_b83c_35bf62b43a13),
            title: "Dui Test Event".parse().expect("valid title"),
            description: "This is a test event started from the infamous DUI"
                .parse()
                .expect("valid description"),
            is_adhoc: true,
            starts_at: None,
            ends_at: None,
            shared_folder: Some(SharedFolder {
                read: SharedFolderAccess {
                    url: "https://example.com/shared-folder/dui-test-event/read-only".to_string(),
                    password: "shared-folder/dui-test-event/read-only".to_string(),
                },
                read_write: Some(SharedFolderAccess {
                    url: "https://example.com/shared-folder/dui-test-event/write".to_string(),
                    password: "shared-folder/dui-test-event/write".to_string(),
                }),
            }),
        }),
        invite_code: None,
        tariff: TariffDetails {
            id: TariffId::from_u128(0x2da2b825_6db9_4dc4_b9e6_b4fd64e66a16),
            name: "Starter tariff".to_string(),
            quotas: BTreeMap::from([
                (
                    QuotaType::MaxStorage,
                    1024 * 1024 * 1024 * 5, // 5 GiB
                ),
                (QuotaType::RoomParticipantLimit, 25),
                (
                    QuotaType::RoomTimeLimitSecs,
                    60 * 60 * 24, // 24h
                ),
            ]),
            used_quota: BTreeMap::new(),
            disabled_features: BTreeSet::new(),
        },
        streaming_targets: vec![],
        show_meeting_details: true,
        e2e_encryption: false,
        module_settings,
        preferred_language: langid!("en"),
        fallback_language: langid!("en"),
        ws_rate_limit: None,
        allowed_origins: vec!["*".to_string()],
        room_idle_timeout: Duration::from_mins(1),
    }
}

pub fn alice_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-alice-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Registered {
            profile: alice_profile(),
        },
        role: Role::Moderator,
    }
}

pub fn bob_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-bob-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Registered {
            profile: bob_profile(),
        },
        role: Role::User,
    }
}

pub fn gustav_client_parameters(connection: usize) -> ClientParameters {
    ClientParameters {
        device_secret: format!("v3rys3cr3tD3v1ce5tr1ng-gustav-{connection}")
            .parse()
            .expect("secret must be valid"),
        kind: ClientKind::Guest {
            display_name: "Gustav"
                .parse()
                .expect("Gustav must be a valid DisplayName"),
        },
        role: Role::User,
    }
}
