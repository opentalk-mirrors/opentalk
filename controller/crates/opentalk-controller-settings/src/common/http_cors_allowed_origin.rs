// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{ResultExt as _, Snafu, ensure};
use url::Url;

const WILDCARD: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq, serde_with::DeserializeFromStr)]
pub struct HttpCorsAllowedOrigin(Option<Url>);

impl HttpCorsAllowedOrigin {
    pub fn header_value(&self) -> String {
        if let Some(url) = &self.0 {
            url.to_string().trim_end_matches('/').to_string()
        } else {
            WILDCARD.to_string()
        }
    }

    pub fn url(&self) -> Option<&Url> {
        self.0.as_ref()
    }

    pub fn is_wildcard(&self) -> bool {
        self.0.is_none()
    }

    pub fn try_from_url_relaxed(url: Url) -> Result<Self, TryFromHttpCorsAllowedOriginError> {
        ensure!(url.has_host(), MustHaveHostSnafu { url });
        let url = Self::sanitize_url(url.clone());
        Ok(Self(Some(url)))
    }

    fn sanitize_url(mut url: Url) -> Url {
        url.set_path("");
        _ = url.set_username("");
        _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
        url
    }

    pub fn wildcard() -> Self {
        Self(None)
    }
}

#[derive(Debug, Snafu)]
pub enum TryFromHttpCorsAllowedOriginError {
    #[snafu(display("CORS must be either \"*\" or a valid URL."))]
    Parse { source: url::ParseError },

    #[snafu(display("CORS origin \"{url}\" must be a valid URL with a host part."))]
    MustHaveHost { url: Url },

    #[snafu(display(
        "CORS origin \"{url}\" must not contain a path, query, fragment, username or password part, should this be \"{sanitized}\" instead?"
    ))]
    MustBeSanitized { url: Url, sanitized: String },
}

impl FromStr for HttpCorsAllowedOrigin {
    type Err = TryFromHttpCorsAllowedOriginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == WILDCARD {
            return Ok(Self(None));
        }

        let url: Url = s.parse().context(ParseSnafu)?;
        ensure!(url.has_host(), MustHaveHostSnafu { url });
        let sanitized_url = Self::sanitize_url(url.clone());

        ensure!(
            url == sanitized_url,
            MustBeSanitizedSnafu {
                url,
                sanitized: sanitized_url.to_string().trim_end_matches('/')
            }
        );

        Ok(Self(Some(sanitized_url)))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::HttpCorsAllowedOrigin;

    #[test]
    fn parse_valid() {
        assert_eq!(
            "http://example.com"
                .parse::<HttpCorsAllowedOrigin>()
                .unwrap()
                .url(),
            Some(&"http://example.com".parse().unwrap())
        );
        assert_eq!(
            "https://example.com:1337"
                .parse::<HttpCorsAllowedOrigin>()
                .unwrap()
                .url(),
            Some(&"https://example.com:1337".parse().unwrap())
        );
        assert_eq!("*".parse::<HttpCorsAllowedOrigin>().unwrap().url(), None);
    }

    #[test]
    fn parse_invalid() {
        assert!("abcdef".parse::<HttpCorsAllowedOrigin>().is_err());
        assert!(
            "https://user@example.com"
                .parse::<HttpCorsAllowedOrigin>()
                .is_err()
        );
        assert!(
            "https://example.com/abc"
                .parse::<HttpCorsAllowedOrigin>()
                .is_err()
        );
        assert!(
            "https://example.com/?abc=def"
                .parse::<HttpCorsAllowedOrigin>()
                .is_err()
        );
        assert!(
            "https://example.com/#abc"
                .parse::<HttpCorsAllowedOrigin>()
                .is_err()
        );
    }
}
