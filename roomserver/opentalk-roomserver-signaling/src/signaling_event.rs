// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingEvent<C> {
    pub namespace: ModuleId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    pub timestamp: Timestamp,
    pub payload: C,
}
