// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides some helper functions and the like.

use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_inventory::{Asset, Inventory};
use opentalk_types_api_internal::module_assets::AssetResource;
use opentalk_types_api_v1::users::{PrivateUserProfile, PublicUserProfile};
use opentalk_types_common::{
    time::TimeZone,
    users::{Language, UserId, UserInfo},
};

/// A trait providing conversion of database users to public and private user profiles
pub trait ToUserProfile {
    /// Convert to the public user profile
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile;

    /// Convert to the private user profile
    fn to_private_user_profile(&self, settings: &Settings, used_storage: u64)
    -> PrivateUserProfile;
}

impl ToUserProfile for opentalk_inventory::User {
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PublicUserProfile {
            id: self.id,
            email: self.email.clone(),
            user_info: UserInfo {
                title: self.title.clone(),
                firstname: self.firstname.clone(),
                lastname: self.lastname.clone(),
                display_name: self.display_name.clone(),
                avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            },
        }
    }

    fn to_private_user_profile(
        &self,
        settings: &Settings,
        used_storage: u64,
    ) -> PrivateUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);
        let default_user_language = &settings.defaults.user_language;

        PrivateUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            dashboard_theme: self.dashboard_theme,
            conference_theme: self.conference_theme,
            avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            language: self
                .language
                .clone()
                .unwrap_or(Language(default_user_language.clone())),
            tariff_status: self.tariff_status,
            used_storage,
        }
    }
}

impl ToUserProfile for RequestUser {
    fn to_public_user_profile(&self, settings: &Settings) -> PublicUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);

        PublicUserProfile {
            id: self.id,
            email: self.email.clone(),
            user_info: UserInfo {
                title: self.title.clone(),
                firstname: self.firstname.clone(),
                lastname: self.lastname.clone(),
                display_name: self.display_name.clone(),
                avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            },
        }
    }

    fn to_private_user_profile(
        &self,
        settings: &Settings,
        used_storage: u64,
    ) -> PrivateUserProfile {
        let default_avatar = email_to_libravatar_url(&settings.avatar.libravatar_url, &self.email);
        let default_user_language = Language(settings.defaults.user_language.clone());

        PrivateUserProfile {
            id: self.id,
            email: self.email.clone(),
            title: self.title.clone(),
            firstname: self.firstname.clone(),
            lastname: self.lastname.clone(),
            display_name: self.display_name.clone(),
            dashboard_theme: self.dashboard_theme,
            conference_theme: self.conference_theme,
            avatar_url: self.avatar_url.clone().unwrap_or(default_avatar),
            language: self.language.clone().unwrap_or(default_user_language),
            tariff_status: self.tariff_status,
            used_storage,
        }
    }
}

/// Helper function to turn an email address into libravatar URL.
pub fn email_to_libravatar_url(libravatar_url: &str, email: &str) -> String {
    format!("{}{:x}", libravatar_url, md5::compute(email))
}

/// Converts an Asset from the database to an asset resource
pub fn asset_to_asset_resource(asset: Asset) -> AssetResource {
    let Asset {
        id,
        created_at,
        updated_at: _,
        namespace,
        kind,
        filename,
        tenant_id: _,
        size,
    } = asset;
    AssetResource {
        id,
        filename,
        namespace,
        created_at: created_at.into(),
        kind,
        size,
    }
}

/// Returns a user's effective timezone
pub async fn get_user_timezone(
    user_id: UserId,
    inventory: &mut dyn Inventory,
    settings: &Settings,
) -> TimeZone {
    let user_timezone = inventory
        .get_user(user_id)
        .await
        .ok()
        .and_then(|user| user.timezone);

    user_timezone.unwrap_or(settings.defaults.timezone)
}
