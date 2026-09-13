// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The tenant id communicated as an OIDC claim.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::AsRef,
    derive_more::Display,
    derive_more::From,
    derive_more::FromStr,
    derive_more::Into,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct OidcTenantId(String);

impl OidcTenantId {
    /// Extracts a string slice containing the ´OidcTenantId´.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for OidcTenantId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}
