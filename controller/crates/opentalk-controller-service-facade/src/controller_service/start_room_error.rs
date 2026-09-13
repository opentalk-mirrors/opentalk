// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides error information regarding starting a room session

use opentalk_types_api_v1::error::ApiError;
use strum::AsRefStr;

/// Errors for the /rooms/{room_id}/start* endpoint
#[derive(Clone, Debug, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum StartRoomError {
    /// The provided room password is wrong
    WrongRoomPassword,

    /// The requested room has no breakout rooms enabled
    NoBreakoutRooms,

    /// The provided breakout room ID is invalid
    InvalidBreakoutRoomId,

    /// The user requesting to start the room is banned from the room
    BannedFromRoom,

    /// The guest has not provided a display name
    NoDisplayName,
}

impl From<StartRoomError> for ApiError {
    fn from(start_room_error: StartRoomError) -> Self {
        match start_room_error {
            StartRoomError::WrongRoomPassword => Self::unauthorized()
                .with_code(StartRoomError::WrongRoomPassword.as_ref())
                .with_message("The provided password does not match the room password"),
            StartRoomError::NoBreakoutRooms => Self::bad_request()
                .with_code(StartRoomError::NoBreakoutRooms.as_ref())
                .with_message("The requested room has no breakout rooms"),
            StartRoomError::InvalidBreakoutRoomId => Self::bad_request()
                .with_code(StartRoomError::InvalidBreakoutRoomId.as_ref())
                .with_message("The provided breakout room ID is invalid"),
            StartRoomError::BannedFromRoom => Self::forbidden()
                .with_code(StartRoomError::BannedFromRoom.as_ref())
                .with_message("This user has been banned from entering this room"),
            StartRoomError::NoDisplayName => Self::bad_request()
                .with_code(StartRoomError::NoDisplayName.as_ref())
                .with_message("Display name is not provided"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(
            StartRoomError::WrongRoomPassword.as_ref(),
            "wrong_room_password"
        );
        assert_eq!(
            StartRoomError::NoBreakoutRooms.as_ref(),
            "no_breakout_rooms"
        );
        assert_eq!(
            StartRoomError::InvalidBreakoutRoomId.as_ref(),
            "invalid_breakout_room_id"
        );
        assert_eq!(StartRoomError::BannedFromRoom.as_ref(), "banned_from_room");
        assert_eq!(StartRoomError::NoDisplayName.as_ref(), "no_display_name");
    }
}
