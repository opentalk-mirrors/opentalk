// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::InventoryProvider;
use opentalk_log::{debug, error, info};
use serde::{Deserialize, Serialize};
use snafu::{Report, ResultExt};

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    events::{DeleteSelector, perform_deletion},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdhocEventCleanupParameters {
    #[serde(default = "a_day_in_seconds")]
    seconds_since_creation: u32,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for AdhocEventCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job to cleanup adhoc events a certain duration after they were created
#[derive(Debug)]
pub struct AdhocEventCleanup;

#[async_trait]
impl Job for AdhocEventCleanup {
    type Parameters = AdhocEventCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting ad-hoc event cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let delete_before =
            Utc::now() - Duration::seconds(parameters.seconds_since_creation.into());

        perform_deletion(
            logger,
            inventory_provider.clone(),
            authorizer,
            stop_room_backend,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::AdHocCreatedBefore(delete_before.into()),
        )
        .await
        .map_err(|e| {
            error!(log: logger, "{}", Report::from_error(e));
            Error::JobExecutionFailed
        })?;
        Ok(())
    }
}

const fn a_day_in_seconds() -> u32 {
    24 * 60 * 60
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use log::logger;
    use opentalk_controller_api_authorization::authorization::Authorizer;
    use opentalk_controller_api_authorization_database::OpenTalkAuthorizerBackend;
    use opentalk_controller_settings::SettingsProvider;
    use opentalk_inventory::InventoryProvider as _;
    use opentalk_test_util::{database::DatabaseContext, object_storage::ObjectStorageContext};

    use super::{AdhocEventCleanup, AdhocEventCleanupParameters};
    use crate::{
        Job as _,
        jobs::test_utils::{RecordingStopRoomBackend, create_generic_test_event},
    };

    /// The ad-hoc event cleanup job must notify the room delete backend for the rooms
    /// of the ad-hoc events it deletes, while leaving rooms of non-ad-hoc events untouched.
    #[actix_rt::test]
    async fn adhoc_event_cleanup_only_deletes_adhoc_event_rooms() {
        let settings_provider = SettingsProvider::load_from_path_or_standard_paths(Some(
            Path::new("../../example/controller.toml"),
        ))
        .unwrap();

        let object_storage_ctx = ObjectStorageContext::new().await;
        let mut settings = (*settings_provider.get()).clone();
        settings.minio = object_storage_ctx.minio.clone();

        let db_ctx = DatabaseContext::new().await;
        let mut inventory = db_ctx.inventory_provider.get_inventory().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        // An ad-hoc event is eligible for cleanup, so its room must be closed on the roomserver.
        let adhoc_event = create_generic_test_event(inventory.as_mut(), &user, true).await;
        // A non-ad-hoc event is not selected by this job; its room must be left alone.
        create_generic_test_event(inventory.as_mut(), &user, false).await;

        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider.clone(),
            BTreeMap::new(),
        ));
        let room_delete_backend = RecordingStopRoomBackend::default();

        AdhocEventCleanup::execute(
            logger(),
            db_ctx.inventory_provider.clone(),
            authorizer,
            &room_delete_backend,
            &settings,
            // `seconds_since_creation: 0` makes every already-created ad-hoc event eligible.
            AdhocEventCleanupParameters {
                seconds_since_creation: 0,
                fail_on_shared_folder_deletion_error: false,
            },
        )
        .await
        .unwrap();

        let deleted_rooms = room_delete_backend.deleted_rooms.lock().unwrap();

        assert_eq!(
            *deleted_rooms,
            vec![adhoc_event.room],
            "AdhocEventCleanup should delete the ad-hoc event's room and leave the non-ad-hoc event's room untouched"
        );
    }
}
