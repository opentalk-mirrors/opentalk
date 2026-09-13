// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{assets::FileExtension, rooms::RoomId, time::Timestamp};

/// Query for the `GET /services/recording/upload` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetRecordingUploadQuery {
    /// The room id
    pub room_id: RoomId,

    /// The file extension
    pub file_extension: FileExtension,

    /// The recording creation timestamp
    pub timestamp: Timestamp,
}
