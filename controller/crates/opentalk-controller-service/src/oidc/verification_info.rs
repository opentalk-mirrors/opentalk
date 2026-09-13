// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};

use super::introspect_info::IntrospectInfo;

/// Info returned from access token verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationInfo {
    /// Expire timestamp of the token
    pub exp: Option<DateTime<Utc>>,
    /// Subject of the access token
    pub sub: Option<String>,
}

impl From<IntrospectInfo> for VerificationInfo {
    fn from(info: IntrospectInfo) -> Self {
        Self {
            exp: info.exp,
            sub: info.sub,
        }
    }
}
