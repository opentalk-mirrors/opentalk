// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_service_auth::service::ApiKeys;
use serde::Deserialize;

/// Service configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Services {
    pub keys: ApiKeys,
}
