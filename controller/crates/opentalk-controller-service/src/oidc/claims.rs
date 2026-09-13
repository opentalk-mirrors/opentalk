// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use openidconnect::AdditionalClaims;
use serde::{Deserialize, Serialize};

use super::jwt;

/// Additional OIDC Claims defined by the controller, which aren't provided inside [`openidconnect::StandardClaims`]
///
// A note to devs:
// Please also update fields in `docs/admin/keycloak.md`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct OpenTalkAdditionalClaims {
    /// The tenant id
    pub tenant_id: Option<String>,
    /// The tariff id
    pub tariff_id: Option<String>,
    /// The tariff status
    pub tariff_status: Option<String>,
    /// Groups the user belongs to.
    #[serde(default)]
    pub x_grp: Vec<String>,
}

impl AdditionalClaims for OpenTalkAdditionalClaims {}

/// Contains the `exp` claim
#[derive(Deserialize, Debug)]
pub struct OnlyExpiryClaim {
    /// Expires at
    #[serde(with = "time")]
    pub exp: DateTime<Utc>,
}

impl jwt::VerifyClaims for OnlyExpiryClaim {
    fn exp(&self) -> DateTime<Utc> {
        self.exp
    }
}

/// Mandatory claims for JWT access token as specified in
/// [rfc9068](https://datatracker.ietf.org/doc/html/rfc9068#section-2.2)
#[derive(Deserialize, Debug)]
pub struct JWTAccessTokenClaims {
    /// Issuer (URL to the OIDC Provider)
    #[expect(unused)]
    pub iss: String,
    /// Expires at
    #[serde(with = "time")]
    pub exp: DateTime<Utc>,
    /// Audience claim
    #[expect(unused)]
    pub aud: String,
    /// Subject
    pub sub: String,
    // Client identifier
    // For some reason Keycloak does not include this claim in the JWT access token
    // pub client_id: String,
    /// Issued at
    #[expect(unused)]
    #[serde(with = "time")]
    pub iat: DateTime<Utc>,
    // JWT ID
    #[expect(unused)]
    pub jti: String,
}

impl jwt::VerifyClaims for JWTAccessTokenClaims {
    fn exp(&self) -> DateTime<Utc> {
        self.exp
    }
}

/// Mandatory claims for JWT logout token as specified in
/// [Logout Token Validation](https://openid.net/specs/openid-connect-backchannel-1_0.html#Validation)
///
/// Note: currenlty we implement only stateless authentication,
///       therefore we completely depend on `sub` and do not need `sid`
#[derive(Deserialize, Debug)]
pub struct JWTLogoutTokenClaims {
    /// Issuer (URL to the OIDC Provider)
    #[expect(unused)]
    pub iss: String,
    /// Expires at
    #[serde(with = "time")]
    pub exp: DateTime<Utc>,
    /// Audience claim
    #[expect(unused)]
    pub aud: String,
    /// Subject
    pub sub: String,
    /// Issued at
    #[expect(unused)]
    #[serde(with = "time")]
    pub iat: DateTime<Utc>,
    // JWT ID
    #[expect(unused)]
    pub jti: String,
    // Events claim
    pub events: HashMap<String, serde_json::Value>,
}

impl jwt::VerifyClaims for JWTLogoutTokenClaims {
    fn exp(&self) -> DateTime<Utc> {
        self.exp
    }
}

/// Service claims
#[derive(Deserialize, Debug)]
pub struct ServiceClaims {
    /// Expires at
    #[serde(with = "time")]
    pub exp: DateTime<Utc>,
    /// Issued at
    #[expect(unused)]
    #[serde(with = "time")]
    pub iat: DateTime<Utc>,
    /// Issuer (URL to the OIDC Provider)
    #[expect(unused)]
    pub iss: String,
    /// Keycloak realm management
    pub realm_access: RealmAccess,
}

impl jwt::VerifyClaims for ServiceClaims {
    fn exp(&self) -> DateTime<Utc> {
        self.exp
    }
}

/// Keycloak realm-management claim which includes the realm specific roles of the client
/// Only included in
#[derive(Deserialize, Debug)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[cfg(test)]
#[derive(Deserialize, Debug, Clone)]
pub(super) struct TestClaims {
    pub(super) sub: String,
    #[serde(with = "time")]
    pub(super) exp: DateTime<Utc>,
    #[serde(flatten)]
    pub(super) opentalk: OpenTalkAdditionalClaims,
}

#[cfg(test)]
impl jwt::VerifyClaims for TestClaims {
    fn exp(&self) -> DateTime<Utc> {
        self.exp
    }
}

mod time {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds: i64 = Deserialize::deserialize(deserializer)?;

        Utc.timestamp_opt(seconds, 0).single().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Failed to convert {seconds} seconds to DateTime<Utc>",
            ))
        })
    }
}
