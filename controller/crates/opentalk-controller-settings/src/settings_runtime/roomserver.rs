// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_roomserver_modules::{ECHO_MODULE_ID, LIVEKIT_MODULE_ID};
use opentalk_roomserver_room::settings::{Internal, Task};
use opentalk_roomserver_types::{
    module_settings::ModuleSettings,
    rate_limit::{self, RateLimitSettings},
};
use opentalk_service_auth::ApiKey;
use opentalk_types_common::modules::ModuleId;
use url::Url;

use crate::{
    SettingsError,
    settings_file::{self, WebSocketRateLimit},
};

const MANDATORY_MODULES: [ModuleId; 2] = [ECHO_MODULE_ID, LIVEKIT_MODULE_ID];

/// The timeout for an empty room
///
/// Should be higher than the lifetime of the signaling token from the token store to ensure that
/// the room doesn't expire before the signaling token does.
pub(crate) const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_mins(1);

/// RoomServer settings
#[derive(Debug, Clone, PartialEq)]
pub struct RoomServer {
    /// The kind of RoomServer to use
    pub kind: RoomServerKind,

    /// Settings regarding the RoomServer modules
    pub modules: ModuleSettings,

    /// Rate limit settings for RoomServer WebSocket connections. If `None`, the WebSocket rate limit is disabled.
    pub websocket_rate_limit: Option<RateLimitSettings>,

    /// The duration after which a room without participants is closed.
    pub room_idle_timeout: Duration,
}

impl TryFrom<settings_file::RoomServer> for RoomServer {
    type Error = SettingsError;

    fn try_from(
        settings_file::RoomServer {
            kind,
            modules,
            websocket_rate_limit,
            room_idle_timeout,
        }: settings_file::RoomServer,
    ) -> Result<Self, Self::Error> {
        let mut missing_modules = Vec::new();
        for module_id in MANDATORY_MODULES {
            if !modules.contains(module_id.clone()) {
                missing_modules.push(module_id.to_string());
            }
        }

        let room_idle_timeout = room_idle_timeout
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_IDLE_TIMEOUT);

        if missing_modules.is_empty() {
            Ok(Self {
                kind: kind.into(),
                modules,
                websocket_rate_limit: rate_limit_from_settings_file(websocket_rate_limit),
                room_idle_timeout,
            })
        } else {
            Err(SettingsError::MandatoryModulesMissing {
                modules: missing_modules,
            })
        }
    }
}

/// RoomServer settings
#[derive(Debug, Clone, PartialEq)]
pub enum RoomServerKind {
    /// A roomserver that is started and managed by the controller
    Internal {
        /// Settings for the room task.
        settings: Task,

        /// Internal configuration for the roomserver.
        server: Internal,
    },
    /// A standalone roomserver that is accessed via its API.
    External {
        /// The URL the controller uses for requests to the roomserver.
        service_url: Url,

        /// The API key to access the roomserver
        api_key: ApiKey,
    },
}

impl From<settings_file::RoomServerKind> for RoomServerKind {
    fn from(value: settings_file::RoomServerKind) -> Self {
        match value {
            settings_file::RoomServerKind::Internal { settings, server } => Self::Internal {
                settings: settings.into(),
                server: server.map(Into::into).unwrap_or_default(),
            },
            settings_file::RoomServerKind::External {
                service_url,
                api_key,
            } => Self::External {
                service_url,
                api_key,
            },
        }
    }
}

fn rate_limit_from_settings_file(config: Option<WebSocketRateLimit>) -> Option<RateLimitSettings> {
    let Some(config) = config else {
        return Some(RateLimitSettings::default());
    };
    let WebSocketRateLimit {
        disabled,
        tokens_per_second,
        token_bucket_size,
        // Do not use rest pattern (`..`) here. We ensure that all fields are used by destructuring `WebSocketRateLimit`
    } = config;

    if disabled {
        return None;
    }

    Some(RateLimitSettings {
        tokens_per_second: tokens_per_second.unwrap_or(rate_limit::DEFAULT_TOKENS_PER_SECOND),
        token_bucket_size: token_bucket_size.unwrap_or(rate_limit::DEFAULT_TOKEN_BUCKET_SIZE),
    })
}
