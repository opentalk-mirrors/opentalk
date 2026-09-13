// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::call_in::{CallInId, CallInPassword};

/// Body for the `POST /services/call_in/start` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostCallInStartRequestBody {
    /// The call-in ID
    pub id: CallInId,
    /// The call-in password
    pub pin: CallInPassword,
}
