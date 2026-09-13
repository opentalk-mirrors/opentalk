// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Error variants for the WWW Authenticate header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuthenticationError {
    /// The provided id token is invalid
    InvalidIdToken,

    /// The provided access token is invalid
    InvalidAccessToken,

    /// The provided access token expired
    AccessTokenInactive,

    /// The user session expired"
    SessionExpired,
}

impl AuthenticationError {
    /// Get the error message for the error
    pub const fn message(&self) -> &'static str {
        match self {
            Self::InvalidIdToken => "The provided id token is invalid",
            Self::InvalidAccessToken => "The provided access token is invalid",
            Self::AccessTokenInactive => "The provided access token expired",
            Self::SessionExpired => "The user session expired",
        }
    }

    /// Get the error code for the variant
    pub const fn error_code(&self) -> &'static str {
        match self {
            AuthenticationError::InvalidIdToken
            | AuthenticationError::InvalidAccessToken
            | AuthenticationError::AccessTokenInactive
            | AuthenticationError::SessionExpired => "invalid_token",
        }
    }

    /// Build the header value string
    pub fn header_value(&self) -> String {
        format!(
            "Bearer error=\"{}\", error_description=\"{}\"",
            self.error_code(),
            self.message()
        )
    }
}
