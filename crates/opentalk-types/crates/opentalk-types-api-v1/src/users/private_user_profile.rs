// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    tariffs::TariffStatus,
    users::{DisplayName, Language, Theme, UserId, UserTitle},
};

/// Private user profile.
///
/// Similar to [`super::PublicUserProfile`], but contains additional "private" information about a user.
/// Is only accessible to the user himself.
/// Is used on */users/me* endpoints.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PrivateUserProfile {
    /// The user id
    pub id: UserId,

    /// The email of the user
    pub email: String,

    /// The title of the user
    pub title: UserTitle,

    /// The user's first name
    pub firstname: String,

    /// The user's last name
    pub lastname: String,

    /// The user's display name
    pub display_name: DisplayName,

    /// The user's avatar URL
    pub avatar_url: String,

    /// The dashboard theme
    pub dashboard_theme: Option<Theme>,

    /// The conference theme
    pub conference_theme: Option<Theme>,

    /// The language for the user
    pub language: Language,

    /// The tariff status of the user
    pub tariff_status: TariffStatus,

    /// The user's used storage
    pub used_storage: u64,
}
