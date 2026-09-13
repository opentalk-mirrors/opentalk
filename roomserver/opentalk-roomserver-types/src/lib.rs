// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Types that are used in the _OpenTalk RoomServer Web API_ and are shared between the RoomServer
//! crates.

pub mod api;
pub mod breakout;
pub mod client_parameters;
pub mod connection_id;
pub mod core;
pub mod device_id;
pub mod disconnect_reason;
pub mod duration_ms;
pub mod error;
pub mod join;
pub mod kick_reason;
pub mod livekit_proxy;
pub mod module_settings;
pub mod public_user_profile;
pub mod rate_limit;
pub mod room_action;
pub mod room_info;
pub mod room_kind;
pub mod room_parameters;
pub mod room_parameters_patch;
pub mod shared_json;
pub mod shared_raw_json;
pub mod signaling;
pub mod tariff_details;

/// Delimiter used for subroom-audio LiveKit room identifiers (`room_id#whisper_id`).
pub const LIVEKIT_SUBROOM_AUDIO_ROOM_DELIMITER: char = '#';
