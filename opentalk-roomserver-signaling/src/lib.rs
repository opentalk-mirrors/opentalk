// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer Signaling
//!
//! Types required to develop signaling modules. Every module is required to
//! implement the [`SignalingModule`](signaling_module::SignalingModule) trait.

pub mod banned_participant;
pub mod event_origin;
pub mod instruction;
/// The [`internal_module_message`] module is intended for internal use only
#[doc(hidden)]
pub mod internal_module_message;
pub mod lobby_participant;
pub mod localization;
/// The [`loopback`] module is intended for internal use only
#[doc(hidden)]
pub mod loopback;
pub mod module_context;
pub mod participant_filter;
pub mod participant_state;
pub mod room_info;
pub mod signaling_event;
pub mod signaling_module;
pub mod storage;
