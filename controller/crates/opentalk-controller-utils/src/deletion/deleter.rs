// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use log::Log;
use opentalk_asset_storage::ObjectStorage;
use opentalk_controller_api_authorization::authorization::{AuthorizationChange, Authorizer};
use opentalk_controller_settings::Settings;
use opentalk_inventory::Inventory;
use opentalk_types_common::users::UserId;

use super::Error;
use crate::deletion::StopRoomBackend;

/// A trait for performing multi-stage deletion of database elements according
/// to this sequence:
/// - Collect referenced datasets that might be changed by others during the process
/// - Check permissions
/// - Perform pre-commit actions, such as removing referenced data from external systems
/// - Commit the changes to the database in a transaction, with a race condition protection
/// - Perform post-commit actions, similar to the pre-commit actions
#[async_trait::async_trait]
pub trait Deleter: Sync {
    /// The outcome of the database commit preparation.
    type PreparedCommit: Sync + Send;

    /// The outcome of the database commit.
    type CommitOutput: Sync + Send;

    /// Perform a full sequence of the steps implemented by trait.
    #[expect(clippy::too_many_arguments)]
    async fn perform(
        &self,
        logger: &dyn Log,
        inventory: &mut dyn Inventory,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        user_id: Option<UserId>,
        settings: &Settings,
        object_storage: &ObjectStorage,
    ) -> Result<(), Error> {
        let prepared_commit = self.prepare_commit(logger, inventory).await?;
        self.check_permissions(&prepared_commit, logger, authorizer.clone(), user_id)
            .await?;
        self.pre_commit(
            &prepared_commit,
            logger,
            inventory,
            stop_room_backend,
            settings,
        )
        .await?;
        let commit_output = self
            .commit_to_inventory(prepared_commit, logger, inventory)
            .await?;

        let authorization_changes = self.authorization_changes(&commit_output);
        if let Err(e) = authorizer.apply_changes(&authorization_changes).await {
            log::warn!(
                "Could not apply authorization changes, {e:?}. Attempted to perform these changes: {authorization_changes:?}"
            );
        }

        self.post_commit(commit_output, logger, settings, authorizer, object_storage)
            .await?;
        Ok(())
    }

    /// Prepare the database commit.
    ///
    /// This should be used to collect data that can be used in the subsequent
    /// steps of the deletion sequence. That can be a list of referenced items,
    /// a list of permissions to be checked in the `check_permissions` step or
    /// other relevant information.
    async fn prepare_commit(
        &self,
        logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::PreparedCommit, Error>;

    /// Check the permissions that are required to perform the deletion.
    ///
    /// This function should return an `Err` if the permissions are not sufficient.
    /// The `user_id` parameter either contains the user who wants to perform the
    /// action (e.g. when called through an API endpoint), or `None` if this
    /// action runs outside the permission system (e.g. by command-line tooling
    /// executed by an administrator).
    async fn check_permissions(
        &self,
        prepared_commit: &Self::PreparedCommit,
        logger: &dyn Log,
        authorizer: Authorizer,
        user_id: Option<UserId>,
    ) -> Result<(), Error>;

    /// Execute actions before the database commit is performed.
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

    /// Commit the changes to the inventory.
    async fn commit_to_inventory(
        &self,
        prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::CommitOutput, Error>;

    /// Execute actions after the database commit was performed.
    async fn post_commit(
        &self,
        _commit_output: Self::CommitOutput,
        _logger: &dyn Log,
        _settings: &Settings,
        _authorizer: Authorizer,
        _storage: &ObjectStorage,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Get the changes that must be applied the authorization system once the deletion succeeds.
    fn authorization_changes(&self, commit_output: &Self::CommitOutput)
    -> Vec<AuthorizationChange>;
}
