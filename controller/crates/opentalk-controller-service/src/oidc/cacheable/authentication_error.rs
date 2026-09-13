// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationError {
    InvalidIdToken,
    InvalidAccessToken,
    AccessTokenInactive,
    SessionExpired,
}

impl From<opentalk_types_api_v1::error::AuthenticationError> for AuthenticationError {
    fn from(value: opentalk_types_api_v1::error::AuthenticationError) -> Self {
        use opentalk_types_api_v1::error::AuthenticationError as Other;
        match value {
            Other::InvalidIdToken => Self::InvalidIdToken,
            Other::InvalidAccessToken => Self::InvalidAccessToken,
            Other::AccessTokenInactive => Self::AccessTokenInactive,
            Other::SessionExpired => Self::SessionExpired,
        }
    }
}

impl From<AuthenticationError> for opentalk_types_api_v1::error::AuthenticationError {
    fn from(value: AuthenticationError) -> Self {
        use AuthenticationError as Other;
        match value {
            Other::InvalidIdToken => Self::InvalidIdToken,
            Other::InvalidAccessToken => Self::InvalidAccessToken,
            Other::AccessTokenInactive => Self::AccessTokenInactive,
            Other::SessionExpired => Self::SessionExpired,
        }
    }
}
