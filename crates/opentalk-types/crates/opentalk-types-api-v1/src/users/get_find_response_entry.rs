// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{PublicUserProfile, UnregisteredUser};

/// The response for users found
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "lowercase")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum GetFindResponseEntry {
    /// Registered user
    Registered(PublicUserProfile),

    /// Unregistered user
    Unregistered(UnregisteredUser),
}
