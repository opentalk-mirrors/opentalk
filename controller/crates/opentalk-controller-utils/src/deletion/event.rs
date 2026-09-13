// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality to delete events including all associated resources

use log::Log;
use opentalk_asset_storage::{ObjectStorage, asset_key};
use opentalk_controller_api_authorization::authorization::{
    AccessMethod, AuthorizationChange, AuthorizationTarget, Authorizer, Resource, Subject,
    SubjectCollection,
};
use opentalk_controller_settings::Settings;
use opentalk_inventory::{EventSharedFolder, Inventory, transaction};
use opentalk_log::debug;
use opentalk_types_common::{assets::AssetId, events::EventId, rooms::RoomId, users::UserId};
use snafu::{ResultExt, ensure};

use super::{Deleter, Error, error::AuthorizationSnafu, shared_folders::delete_shared_folders};
use crate::deletion::{
    StopRoomBackend,
    error::{ObjectDeletionSnafu, RaceConditionSnafu},
    room::delete_rows_associated_with_room,
};

/// Delete an event by id including the corresponding room and resources it
/// references.
#[derive(Debug)]
pub struct EventDeleter {
    event_id: EventId,
    room_id: RoomId,
    fail_on_shared_folder_deletion_error: bool,
}

impl EventDeleter {
    /// Create a new `EventDeleter`
    ///
    /// If `fail_on_shared_folder_deletion` is true, the deletion will fail as soon
    /// as a shared folder can not be deleted from the external storage system.
    ///
    /// Otherwise just warnings will be logged, but the deletion is considered successful.
    pub fn new(
        event_id: EventId,
        room_id: RoomId,
        fail_on_shared_folder_deletion_error: bool,
    ) -> Self {
        Self {
            event_id,
            room_id,
            fail_on_shared_folder_deletion_error,
        }
    }
}

/// A struct holding the information that was collected during the database
/// commit preparation.
#[derive(Debug)]
pub struct EventDeleterPreparedCommit {
    linked_shared_folder: Option<EventSharedFolder>,
}

impl EventDeleterPreparedCommit {
    async fn detect_race_condition(
        &self,
        inventory: &mut dyn Inventory,
        event_id: EventId,
    ) -> Result<(), Error> {
        let current_shared_folder = inventory.get_event_shared_folder(event_id).await?;
        ensure!(
            current_shared_folder == self.linked_shared_folder,
            RaceConditionSnafu
        );

        Ok(())
    }
}

/// A struct holding the information that was collected during database commit.
#[derive(Debug)]
pub struct EventDeleterCommitOutput {
    room_id: RoomId,
    assets: Vec<AssetId>,
}

#[async_trait::async_trait]
impl Deleter for EventDeleter {
    type PreparedCommit = EventDeleterPreparedCommit;
    type CommitOutput = EventDeleterCommitOutput;

    async fn prepare_commit(
        &self,
        _logger: &dyn Log,
        inventory: &mut dyn Inventory,
    ) -> Result<Self::PreparedCommit, Error> {
        let linked_shared_folder = inventory.get_event_shared_folder(self.event_id).await?;

        Ok(EventDeleterPreparedCommit {
            linked_shared_folder,
        })
    }

    async fn check_permissions(
        &self,
        _prepared_commit: &Self::PreparedCommit,
        _logger: &dyn Log,
        authorizer: Authorizer,
        user_id: Option<UserId>,
    ) -> Result<(), Error> {
        let user_id = match user_id {
            Some(user_id) => user_id,
            None => return Ok(()),
        };

        let target = AuthorizationTarget {
            authenticated_subjects: SubjectCollection::from_iter([Subject::User(user_id)]),
            resource: Resource::Event(self.event_id),
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
            prepared_commit.linked_shared_folder.as_slice(),
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

        let event_id = self.event_id;
        let room_id = self.room_id;

        let transaction_result: Result<Vec<AssetId>, Error> =
            transaction(inventory, async |inventory| {
                prepared_commit
                    .detect_race_condition(inventory, event_id)
                    .await?;

                let mut current_assets = inventory.get_all_asset_ids_for_room(room_id).await?;
                current_assets.sort();

                delete_rows_associated_with_room(
                    &logger,
                    inventory,
                    room_id,
                    &current_assets,
                    &[event_id],
                )
                .await?;

                Ok(current_assets)
            })
            .await;

        let assets = transaction_result?;

        Ok(EventDeleterCommitOutput { room_id, assets })
    }

    async fn post_commit(
        &self,
        commit_output: EventDeleterCommitOutput,
        logger: &dyn Log,
        _settings: &Settings,
        authorizer: Authorizer,
        storage: &ObjectStorage,
    ) -> Result<(), Error> {
        debug!(
            log: logger,
            "Deleting {} asset(s) from the storage",
            commit_output.assets.len()
        );
        if let Err(e) = authorizer
            .apply_change(&AuthorizationChange::DeleteEvent {
                event: self.event_id,
            })
            .await
        {
            log::warn!("Couldn't apply event deletion authorization change: {e:?}");
        };
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
        commit_output: &Self::CommitOutput,
    ) -> Vec<AuthorizationChange> {
        vec![
            AuthorizationChange::DeleteEvent {
                event: self.event_id,
            },
            AuthorizationChange::DeleteRoom {
                room: commit_output.room_id,
            },
        ]
    }
}
