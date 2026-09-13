// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality to delete rooms including all associated resources

use log::Log;
use opentalk_asset_storage::{ObjectStorage, asset_key};
use opentalk_controller_api_authorization::authorization::{
    AccessMethod, AuthorizationChange, AuthorizationTarget, Authorizer, Resource, Subject,
    SubjectCollection,
};
use opentalk_controller_settings::Settings;
use opentalk_inventory::{EventSharedFolder, Inventory, transaction};
use opentalk_log::debug;
use opentalk_types_common::{
    assets::AssetId, events::EventId, module_resources::ModuleResourceId, rooms::RoomId,
    users::UserId,
};
use snafu::{ResultExt, ensure};

use super::{Deleter, Error, error::AuthorizationSnafu};
use crate::deletion::{
    StopRoomBackend,
    error::{ObjectDeletionSnafu, RaceConditionSnafu},
    shared_folders::delete_shared_folders,
};

/// Delete a room by id including the resources it references.
#[derive(Debug)]
pub struct RoomDeleter {
    room_id: RoomId,
    fail_on_shared_folder_deletion_error: bool,
}

impl RoomDeleter {
    /// Create a new `RoomDeleter`
    ///
    /// If `fail_on_shared_folder_deletion` is true, the deletion will fail as soon
    /// as a shared folder can not be deleted from the external storage system.
    ///
    /// Otherwise just warnings will be logged, but the deletion is considered successful.
    pub fn new(room_id: RoomId, fail_on_shared_folder_deletion_error: bool) -> Self {
        Self {
            room_id,
            fail_on_shared_folder_deletion_error,
        }
    }
}

/// A struct holding the information that was collected during the database
/// commit preparation.
#[derive(Debug)]
pub struct RoomDeleterPreparedCommit {
    linked_module_resources: Vec<ModuleResourceId>,
    linked_shared_folders: Vec<EventSharedFolder>,
}

impl RoomDeleterPreparedCommit {
    async fn detect_race_condition(
        &self,
        inventory: &mut dyn Inventory,
        room_id: RoomId,
    ) -> Result<(), Error> {
        let mut current_module_resources = inventory.get_all_module_ids_for_room(room_id).await?;
        current_module_resources.sort();

        ensure!(
            current_module_resources == self.linked_module_resources,
            RaceConditionSnafu
        );

        let mut current_shared_folders =
            inventory.get_event_shared_folders_for_room(room_id).await?;
        current_shared_folders.sort_by_key(|a| a.event_id);
        ensure!(
            current_module_resources == self.linked_module_resources,
            RaceConditionSnafu
        );

        Ok(())
    }
}

/// A struct holding the information that was collected during database commit.
#[derive(Debug)]
pub struct RoomDeleterCommitOutput {
    assets: Vec<AssetId>,
}

#[async_trait::async_trait]
impl Deleter for RoomDeleter {
    type PreparedCommit = RoomDeleterPreparedCommit;
    type CommitOutput = RoomDeleterCommitOutput;

    async fn prepare_commit(
        &self,
        _logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::PreparedCommit, Error> {
        if inventory
            .get_event_id_for_room(self.room_id)
            .await?
            .is_some()
        {
            return Err(Error::Conflict {
                message: format!(
                    "Unable to delete room with id {} due to conflicting event",
                    self.room_id
                ),
            });
        }

        let mut linked_module_resources =
            inventory.get_all_module_ids_for_room(self.room_id).await?;
        let mut linked_shared_folders = inventory
            .get_event_shared_folders_for_room(self.room_id)
            .await?;

        // Sort for improved equality comparison later on, inside the transaction.
        linked_module_resources.sort();
        linked_shared_folders.sort_by_key(|a| a.event_id);

        Ok(RoomDeleterPreparedCommit {
            linked_module_resources,
            linked_shared_folders,
        })
    }

    async fn check_permissions(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        authorizer: Authorizer,
        user_id: Option<UserId>,
    ) -> Result<(), Error> {
        let Some(user_id) = user_id else {
            return Ok(());
        };

        let target = AuthorizationTarget {
            authenticated_subjects: SubjectCollection::from_iter([Subject::User(user_id)]),
            resource: Resource::Room(self.room_id.into()),
            access_method: AccessMethod::Delete,
        };
        if !authorizer
            .authorize(target)
            .await
            .with_context(|_| AuthorizationSnafu)?
            .is_allowed()
        {
            return Err(Error::Forbidden);
        }

        Ok(())
    }

    async fn pre_commit(
        &self,
        prepared_commit: &Self::PreparedCommit,
        logger: &dyn Log,
        _inventory: &mut dyn Inventory,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
    ) -> Result<(), Error> {
        stop_room_backend.stop_room(self.room_id).await?;

        delete_shared_folders(
            logger,
            settings,
            &prepared_commit.linked_shared_folders,
            self.fail_on_shared_folder_deletion_error,
        )
        .await?;
        Ok(())
    }

    async fn commit_to_inventory(
        &self,
        prepared_commit: Self::PreparedCommit,
        logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::CommitOutput, Error> {
        debug!(log: logger, "Deleting all database resources");

        let room_id = self.room_id;

        let transaction_result: Result<Vec<AssetId>, Error> =
            transaction(inventory, async |inventory| {
                prepared_commit
                    .detect_race_condition(inventory, room_id)
                    .await?;

                let shared_folder_event_ids = prepared_commit
                    .linked_shared_folders
                    .iter()
                    .map(|e| e.event_id)
                    .collect::<Vec<EventId>>();

                let mut current_assets = inventory.get_all_asset_ids_for_room(room_id).await?;
                current_assets.sort();

                delete_rows_associated_with_room(
                    logger,
                    inventory,
                    room_id,
                    &current_assets,
                    &shared_folder_event_ids,
                )
                .await?;

                Ok(current_assets)
            })
            .await;

        let assets = transaction_result?;

        Ok(RoomDeleterCommitOutput { assets })
    }

    async fn post_commit(
        &self,
        commit_output: RoomDeleterCommitOutput,
        logger: &dyn Log,
        _settings: &Settings,
        _authorizer: Authorizer,
        storage: &ObjectStorage,
    ) -> Result<(), Error> {
        debug!(
            log: logger,
            "Deleting {} asset(s) from the storage",
            commit_output.assets.len()
        );
        for asset_id in commit_output.assets {
            debug!(log: logger, "Deleting asset {asset_id} from the storage");
            storage
                .delete(asset_key(&asset_id))
                .await
                .context(ObjectDeletionSnafu)?;
        }

        Ok(())
    }

    fn authorization_changes(
        &self,
        _commit_output: &Self::CommitOutput,
    ) -> Vec<AuthorizationChange> {
        vec![AuthorizationChange::DeleteRoom { room: self.room_id }]
    }
}

pub(crate) async fn delete_rows_associated_with_room(
    logger: &dyn Log,
    inventory: &mut dyn Inventory,
    room_id: RoomId,
    asset_ids: &[AssetId],
    shared_folder_event_ids: &[EventId],
) -> Result<(), opentalk_inventory::Error> {
    debug!(log: logger, "Deleting shared folders from database");
    inventory
        .delete_shared_folders_by_event_ids(shared_folder_event_ids)
        .await?;

    debug!(log: logger, "Deleting module resources from database");
    inventory
        .delete_all_module_resources_for_room(room_id)
        .await?;

    debug!(log: logger, "Deleting event from database");
    inventory.delete_event_for_room(room_id).await?;

    debug!(log: logger, "Deleting sip config from database");
    inventory.delete_room_sip_config(&room_id.into()).await?;

    debug!(log: logger, "Deleting asset information from database");
    inventory.delete_assets_by_ids(asset_ids).await?;

    debug!(log: logger, "Deleting room");
    inventory.delete_room(room_id).await?;

    Ok(())
}
