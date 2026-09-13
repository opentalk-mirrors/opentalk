// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone as _, Utc};
use opentalk_types_common::utils::ExampleData;

/// Body for *PUT /rooms/{room_id}/invites/{invite_code}*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PutInviteRequestBody::example_data()))
)]
pub struct PutInviteRequestBody {
    /// Optional expiration date of the invite
    pub expiration: Option<DateTime<Utc>>,
}

impl ExampleData for PutInviteRequestBody {
    fn example_data() -> Self {
        Self {
            expiration: Some(Utc.with_ymd_and_hms(2024, 6, 20, 14, 16, 19).unwrap()),
        }
    }
}
