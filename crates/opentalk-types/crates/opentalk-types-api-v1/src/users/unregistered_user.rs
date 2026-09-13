// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Representation of a unregistered user
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UnregisteredUser {
    /// Email of the unregistered user
    pub email: String,

    /// First name of the unregistered user
    pub firstname: String,

    /// Last name of the unregistered user
    pub lastname: String,

    /// Avatar URL for the unregistered user
    pub avatar_url: String,
}
