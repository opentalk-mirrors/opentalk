// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use super::{EmailOnlyUser, PublicInviteUserProfile};
use crate::users::UnregisteredUser;

/// Profile of an event invitee
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventInviteeProfile::example_data()))
)]
pub enum EventInviteeProfile {
    /// Registered user profile
    Registered(PublicInviteUserProfile),
    /// Unregistered user profile
    Unregistered(UnregisteredUser),
    /// Email only user profile
    Email(EmailOnlyUser),
}

impl ExampleData for EventInviteeProfile {
    fn example_data() -> Self {
        Self::Registered(PublicInviteUserProfile::example_data())
    }
}
