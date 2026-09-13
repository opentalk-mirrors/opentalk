// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use crate::common::HttpCorsAllowedOrigin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpCorsAllowedOrigins(Vec<HttpCorsAllowedOrigin>);

impl HttpCorsAllowedOrigins {
    pub fn into_vec(self) -> Vec<HttpCorsAllowedOrigin> {
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &HttpCorsAllowedOrigin> {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for HttpCorsAllowedOrigins {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = Vec::<HttpCorsAllowedOrigin>::deserialize(deserializer)?;

        if inner.is_empty() {
            return Err(serde::de::Error::custom(
                "List of CORS allowed origins is empty. Either remove the field, or add some entries.",
            ));
        }

        if inner.iter().any(HttpCorsAllowedOrigin::is_wildcard) && inner.len() != 1 {
            return Err(serde::de::Error::custom(
                "Found '*' in a list of multiple CORS allowed origins. If '*' is allowed, it must be the only element in that list.",
            ));
        }

        Ok(Self(inner))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::common::{HttpCorsAllowedOrigin, HttpCorsAllowedOrigins};

    #[test]
    fn deserialize_invalid_empty() {
        assert!(serde_json::from_value::<HttpCorsAllowedOrigins>(json!([])).is_err());
    }

    #[test]
    fn deserialize_wildcard() {
        assert_eq!(
            serde_json::from_value::<HttpCorsAllowedOrigins>(json!(["*"]))
                .unwrap()
                .into_vec(),
            vec![HttpCorsAllowedOrigin::wildcard()]
        );
    }

    #[test]
    fn deserialize_invalid_multiple_wildcard() {
        assert!(serde_json::from_value::<HttpCorsAllowedOrigins>(json!(["*", "*"])).is_err());
    }

    #[test]
    fn deserialize_multiple_urls() {
        assert_eq!(
            serde_json::from_value::<HttpCorsAllowedOrigins>(json!([
                "http://example.com",
                "https://example.com:1338",
                "http://localhost:8000"
            ]))
            .unwrap()
            .into_vec(),
            vec![
                HttpCorsAllowedOrigin::try_from_url_relaxed("http://example.com".parse().unwrap())
                    .unwrap(),
                HttpCorsAllowedOrigin::try_from_url_relaxed(
                    "https://example.com:1338".parse().unwrap()
                )
                .unwrap(),
                HttpCorsAllowedOrigin::try_from_url_relaxed(
                    "http://localhost:8000".parse().unwrap()
                )
                .unwrap(),
            ]
        );
    }

    #[test]
    fn deserialize_invalid_multiple_url_with_wildcard() {
        assert!(
            serde_json::from_value::<HttpCorsAllowedOrigins>(json!([
                "http://example.com",
                "https://example.com:1338",
                "*",
                "http://localhost:8000"
            ]))
            .is_err()
        );
    }
}
