// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types used by the `recording` service

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

/// Common Request Body used by
///
/// - controller for the `POST /internal/services/recording/start`
/// - recorder for the `POST /v1/init`
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RecordingTarget {
    /// The room id which shall be recorded
    pub room_id: RoomId,

    /// The optional breakout room id in which the recording was requested
    ///
    /// This is the internal equivalent to opentalk-roomserver-types's `BreakoutId`
    pub breakout_room: Option<u32>,
}
