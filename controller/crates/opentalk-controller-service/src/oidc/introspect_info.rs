// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};

/// Info returned from the access token introspection
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct IntrospectInfo {
    /// Access token is still active
    pub active: bool,
    /// Expire timestamp of the token
    pub exp: Option<DateTime<Utc>>,
    /// Subject of the access token
    pub sub: Option<String>,
}
