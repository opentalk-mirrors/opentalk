// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{connection_id::ConnectionId, device_id::DeviceId};

/// A participants connection information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: ConnectionId,
    pub device_id: DeviceId,
}
