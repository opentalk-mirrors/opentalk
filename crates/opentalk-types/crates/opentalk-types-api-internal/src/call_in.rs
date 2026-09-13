// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types used by the `call_in` service

use opentalk_types_common::call_in::{CallInId, CallInPassword};
use serde::{Deserialize, Serialize};

/// Body for the `POST /services/call_in/start` endpoint
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostCallInStartRoomServerRequestBody {
    /// The call-in ID
    pub id: CallInId,
    /// The call-in password
    pub pin: CallInPassword,
}
