// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoomAction {
    Created,
    Updated,
}

impl RoomAction {
    /// Returns `true` if the room action is [`Created`].
    ///
    /// [`Created`]: RoomAction::Created
    #[must_use]
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created)
    }

    #[cfg(feature = "axum")]
    pub fn from_status_code(status_code: axum::http::StatusCode) -> Option<Self> {
        match status_code {
            axum::http::StatusCode::CREATED => Some(Self::Created),
            axum::http::StatusCode::NO_CONTENT => Some(Self::Updated),
            _ => None,
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for RoomAction {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Created => axum::http::StatusCode::CREATED.into_response(),
            Self::Updated => axum::http::StatusCode::NO_CONTENT.into_response(),
        }
    }
}
