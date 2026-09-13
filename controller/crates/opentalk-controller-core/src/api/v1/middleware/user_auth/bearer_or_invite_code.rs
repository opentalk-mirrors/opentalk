// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::header::{HeaderValue, InvalidHeaderValue, TryIntoHeaderValue};
use actix_web_httpauth::headers::authorization::{Bearer, ParseError, Scheme};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BearerOrInviteCode {
    Bearer(Bearer),
    InviteCode,
}

impl std::fmt::Display for BearerOrInviteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bearer(bearer) => bearer.fmt(f),
            Self::InviteCode => write!(f, "invite-code"),
        }
    }
}

impl Scheme for BearerOrInviteCode {
    fn parse(header: &HeaderValue) -> Result<Self, ParseError> {
        let header_first_char = header.to_str()?.chars().next();

        match header_first_char {
            Some('B') => Ok(Self::Bearer(Bearer::parse(header)?)),
            Some('I') => Ok(Self::InviteCode),
            _ => Err(ParseError::Invalid),
        }
    }
}

impl TryIntoHeaderValue for BearerOrInviteCode {
    type Error = InvalidHeaderValue;

    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        match self {
            Self::Bearer(bearer) => bearer.try_into_value(),
            Self::InviteCode => HeaderValue::from_str("InviteCode invite-code"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_bearer() {
        let value = HeaderValue::from_static("Bearer abc-def-ghi");
        let scheme = BearerOrInviteCode::parse(&value);

        assert!(scheme.is_ok());
        let scheme = scheme.unwrap();
        assert!(matches!(scheme, BearerOrInviteCode::Bearer(b) if b.token() == "abc-def-ghi"));
    }

    #[test]
    fn test_parse_invite_code() {
        let value =
            HeaderValue::from_str("InviteCode c7fe02dd-ba7b-4fc5-a8ba-a9c778f348dc").unwrap();
        let scheme = BearerOrInviteCode::parse(&value);

        assert!(scheme.is_ok());
        let scheme = scheme.unwrap();

        assert_eq!(scheme, BearerOrInviteCode::InviteCode);
    }

    #[test]
    fn test_empty_header() {
        let value = HeaderValue::from_static("");
        let scheme = BearerOrInviteCode::parse(&value);

        assert!(scheme.is_err());
    }

    #[test]
    fn test_wrong_scheme() {
        let value = HeaderValue::from_static("OAuthToken foo");
        let scheme = BearerOrInviteCode::parse(&value);

        assert!(scheme.is_err());
    }
}
