// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality to delete users including all associated resources

use log::Log;
use opentalk_asset_storage::ObjectStorage;
use opentalk_controller_api_authorization::authorization::{AuthorizationChange, Authorizer};
use opentalk_controller_settings::Settings;
use opentalk_inventory::{Inventory, transaction};
use opentalk_log::debug;
use opentalk_types_common::users::UserId;

use super::{Deleter, Error};
use crate::deletion::StopRoomBackend;
/// Delete a user by id including the corresponding room and resources it
/// references.
#[derive(Debug)]
pub struct UserDeleter {
    user_id: UserId,
}

impl UserDeleter {
    /// Create a new `UserDeleter`.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

#[async_trait::async_trait]
impl Deleter for UserDeleter {
    type PreparedCommit = ();
    type CommitOutput = ();

    async fn prepare_commit(
        &self,
        _logger: &dyn Log,
        _inventory: &mut dyn Inventory,
    ) -> Result<Self::PreparedCommit, Error> {
        Ok(())
    }

    async fn check_permissions(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        _authorizer: Authorizer,
        _user_id: Option<UserId>,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn pre_commit(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        _inventory: &mut dyn Inventory,
        _stop_room_backend: &dyn StopRoomBackend,
        _settings: &Settings,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn commit_to_inventory(
        &self,
        _prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::CommitOutput, Error> {
        let user_id = self.user_id;

        debug!(log: logger, "Deleting all database resources of user {user_id}");
        let _transaction_result: Result<(), opentalk_inventory::Error> =
            transaction(inventory, async |inventory| {
                inventory.remove_user_from_all_groups(user_id).await?;
                inventory.delete_user(user_id).await?;

                Ok(())
            })
            .await;

        Ok(())
    }

    async fn post_commit(
        &self,
        _commit_output: (),
        _logger: &dyn Log,
        _settings: &Settings,
        _authorizer: Authorizer,
        _storage: &ObjectStorage,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn authorization_changes(
        &self,
        _commit_output: &Self::CommitOutput,
    ) -> Vec<AuthorizationChange> {
        vec![AuthorizationChange::DeleteUser { user: self.user_id }]
    }
}
