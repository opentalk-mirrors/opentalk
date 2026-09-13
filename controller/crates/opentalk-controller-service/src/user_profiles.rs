// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides some utilities.

use std::collections::HashMap;

use opentalk_controller_settings::Settings;
use opentalk_database::Result;
use opentalk_inventory::{HasUsers, Inventory};
use opentalk_types_api_v1::users::PublicUserProfile;
use opentalk_types_common::users::UserId;

use crate::ToUserProfile;

/// Utility to fetch user profiles batched
///
/// See [`db_storage::utils::HasUsers`]
#[derive(Debug, Default)]
pub(crate) struct GetUserProfilesBatched {
    users: Vec<UserId>,
}

impl GetUserProfilesBatched {
    pub fn new() -> Self {
        Self { users: vec![] }
    }

    pub fn add(&mut self, has_users: impl HasUsers) -> &mut Self {
        has_users.populate(&mut self.users);
        self
    }

    pub async fn fetch(
        &mut self,
        settings: &Settings,
        inventory: &mut dyn Inventory,
    ) -> Result<UserProfilesBatch, opentalk_inventory::Error> {
        if self.users.is_empty() {
            return Ok(UserProfilesBatch {
                users: HashMap::new(),
            });
        }

        self.users.sort_unstable();
        self.users.dedup();

        inventory
            .get_users_by_ids(&self.users)
            .await
            .map(|users| {
                users
                    .into_iter()
                    .map(|user| (user.id, user.to_public_user_profile(settings)))
                    .collect()
            })
            .map(|users| UserProfilesBatch { users })
    }
}

#[derive(Debug)]
pub(crate) struct UserProfilesBatch {
    users: HashMap<UserId, PublicUserProfile>,
}

impl UserProfilesBatch {
    pub fn get(&self, id: UserId) -> PublicUserProfile {
        self.users
            .get(&id)
            .expect("tried to get user-profile without fetching it first")
            .clone()
    }
}
