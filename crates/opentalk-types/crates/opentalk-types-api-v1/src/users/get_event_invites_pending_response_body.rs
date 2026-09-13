// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Response body for the `GET /users/me/pending_invites` endpoint
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetEventInvitesPendingResponseBody::example_data()))
)]
pub struct GetEventInvitesPendingResponseBody {
    /// The total number of pending invites for the current user
    pub total_pending_invites: u32,
}

impl ExampleData for GetEventInvitesPendingResponseBody {
    fn example_data() -> Self {
        Self {
            total_pending_invites: 3,
        }
    }
}
