// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # RoomServer Common
//!
//! Types that are shared between the `opentalk-roomserver` crate and signaling
//! module crates are placed here. They can't be part of the `opentalk-roomserver`
//! since this would create a dependency cycle.

pub mod application_state;
pub mod settings;
pub mod token_store;
