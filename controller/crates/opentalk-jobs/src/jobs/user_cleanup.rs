// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Days, Utc};
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
    users::{DeleteSelector, perform_deletion},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserCleanupParameters {
    #[serde(default = "default_days_since_user_has_been_disabled")]
    days_since_user_has_been_disabled: u64,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for UserCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job for cleaning up users that were disabled at minimum a defined duration ago
#[derive(Debug)]
pub struct UserCleanup;

#[async_trait]
impl Job for UserCleanup {
    type Parameters = UserCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting disabled user cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        info!(log: logger, "");

        let now = Utc::now();
        let delete_before = now
            .checked_sub_days(Days::new(parameters.days_since_user_has_been_disabled))
            .ok_or_else(|| {
                error!(log: logger, "Couldn't subtract number of retention days");
                Error::JobExecutionFailed
            })?;

        perform_deletion(
            logger,
            inventory_provider,
            authorizer,
            stop_room_backend,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::DisabledBefore(delete_before.into()),
        )
        .await
        .map_err(|err| {
            error!(log: logger, "{}", Report::from_error(err));
            Error::JobExecutionFailed
        })?;

        Ok(())
    }
}

fn default_days_since_user_has_been_disabled() -> u64 {
    30
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use chrono::{DateTime, Days, Utc};
    use log::logger;
    use opentalk_controller_api_authorization::authorization::Authorizer;
    use opentalk_controller_api_authorization_database::OpenTalkAuthorizerBackend;
    use opentalk_controller_settings::SettingsProvider;
    use opentalk_controller_utils::deletion::NoOpStopRoomBackend;
    use opentalk_inventory::{
        Event, Inventory, InventoryProvider as _, UpdateEvent, UpdateUser, User,
    };
    use opentalk_test_util::{database::DatabaseContext, object_storage::ObjectStorageContext};
    use opentalk_types_common::{events::EventId, time::Timestamp, users::UserId};

    use super::{UserCleanup, default_days_since_user_has_been_disabled};
    use crate::{
        Job as _,
        jobs::{
            test_utils::{RecordingStopRoomBackend, create_generic_test_event},
            user_cleanup::UserCleanupParameters,
        },
    };

    fn init_logger() {
        let mut builder = env_logger::Builder::new();
        builder
            .filter_level(log::LevelFilter::Info)
            .format_timestamp(None)
            .parse_default_env();
        let _ = builder.try_init();
    }

    async fn set_disabled_since(
        inventory: &mut dyn Inventory,
        user_id: UserId,
        since: DateTime<Utc>,
    ) -> User {
        inventory
            .update_user(
                user_id,
                UpdateUser {
                    title: None,
                    email: None,
                    firstname: None,
                    lastname: None,
                    avatar_url: None,
                    timezone: None,
                    phone: None,
                    display_name: None,
                    language: None,
                    dashboard_theme: None,
                    conference_theme: None,
                    tariff_id: None,
                    tariff_status: None,
                    disabled_since: Some(Some(since.into())),
                    updated_at: Timestamp::now(),
                },
            )
            .await
            .unwrap()
    }

    async fn update_event(inventory: &mut dyn Inventory, user: UserId, event: EventId) -> Event {
        inventory
            .update_event(
                event,
                UpdateEvent {
                    title: None,
                    description: None,
                    updated_by: user,
                    updated_at: Timestamp::now(),
                    is_adhoc: None,
                    show_meeting_details: None,
                    date: None,
                },
            )
            .await
            .unwrap()
    }

    #[actix_rt::test]
    async fn cleanup_user_with_event() {
        init_logger();
        let settings_provider = SettingsProvider::load_from_path_or_standard_paths(Some(
            Path::new("../../example/controller.toml"),
        ))
        .unwrap();

        let object_storage_ctx = ObjectStorageContext::new().await;
        let mut settings = settings_provider.get().as_ref().clone();
        settings.minio = object_storage_ctx.minio.clone();

        let db_ctx = DatabaseContext::new().await;
        let mut inventory = db_ctx.inventory_provider.get_inventory().await.unwrap();

        let inviter = db_ctx.create_test_user(0, vec![]).await.unwrap();
        let updated_by = db_ctx.create_test_user(2, vec![]).await.unwrap();

        let event = create_generic_test_event(inventory.as_mut(), &inviter, true).await;
        update_event(inventory.as_mut(), updated_by.id, event.id).await;

        let disabled_since = Utc::now()
            .checked_sub_days(Days::new(default_days_since_user_has_been_disabled() + 1))
            .unwrap();
        let updated_by =
            set_disabled_since(inventory.as_mut(), updated_by.id, disabled_since).await;

        // User::get filters disabled users
        let user_exists = inventory
            .get_all_users()
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == updated_by.id);
        assert!(user_exists);

        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider.clone(),
            BTreeMap::new(),
        ));

        UserCleanup::execute(
            logger(),
            db_ctx.inventory_provider.clone(),
            authorizer,
            &NoOpStopRoomBackend,
            &settings,
            serde_json::from_str("{}").unwrap(),
        )
        .await
        .unwrap();

        let user_exists = inventory
            .get_all_users()
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == updated_by.id);
        assert!(!user_exists, "User was not successfully cleaned up");
    }

    #[actix_rt::test]
    async fn cleanup_user() {
        init_logger();
        let settings_provider = SettingsProvider::load_from_path_or_standard_paths(Some(
            Path::new("../../example/controller.toml"),
        ))
        .unwrap();

        let object_storage_ctx = ObjectStorageContext::new().await;
        let mut settings = settings_provider.get().as_ref().clone();
        settings.minio = object_storage_ctx.minio.clone();

        let db_ctx = DatabaseContext::new().await;
        let mut inventory = db_ctx.inventory_provider.get_inventory().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        let disabled_since = Utc::now()
            .checked_sub_days(Days::new(default_days_since_user_has_been_disabled() + 1))
            .unwrap();
        let inviter = set_disabled_since(inventory.as_mut(), user.id, disabled_since).await;

        // User::get filters disabled users
        let user_exists = inventory
            .get_all_users()
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == inviter.id);
        assert!(user_exists);

        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider.clone(),
            BTreeMap::new(),
        ));

        UserCleanup::execute(
            logger(),
            db_ctx.inventory_provider.clone(),
            authorizer,
            &NoOpStopRoomBackend,
            &settings,
            serde_json::from_str("{}").unwrap(),
        )
        .await
        .unwrap();

        let user_exists = inventory
            .get_all_users()
            .await
            .unwrap()
            .iter()
            .any(|u| u.id == inviter.id);
        assert!(!user_exists, "User was not successfully cleaned up");
    }

    #[actix_rt::test]
    async fn user_cleanup_deletes_events_and_rooms() {
        let settings_provider = SettingsProvider::load_from_path_or_standard_paths(Some(
            Path::new("../../example/controller.toml"),
        ))
        .unwrap();

        let object_storage_ctx = ObjectStorageContext::new().await;
        let mut settings = settings_provider.get().as_ref().clone();
        settings.minio = object_storage_ctx.minio.clone();

        let db_ctx = DatabaseContext::new().await;
        let mut inventory = db_ctx.inventory_provider.get_inventory().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();
        let event = create_generic_test_event(inventory.as_mut(), &user, false).await;

        let disabled_since = Utc::now()
            .checked_sub_days(Days::new(default_days_since_user_has_been_disabled() + 1))
            .unwrap();
        let user = set_disabled_since(inventory.as_mut(), user.id, disabled_since).await;

        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider.clone(),
            BTreeMap::new(),
        ));
        let room_delete_backend = RecordingStopRoomBackend::default();

        UserCleanup::execute(
            logger(),
            db_ctx.inventory_provider.clone(),
            authorizer,
            &room_delete_backend,
            &settings,
            UserCleanupParameters {
                days_since_user_has_been_disabled: 0,
                fail_on_shared_folder_deletion_error: false,
            },
        )
        .await
        .unwrap();

        let events = inventory
            .get_all_event_and_room_ids_created_by_user_including_disabled(user.id)
            .await
            .unwrap();
        assert_eq!(events, vec![]);

        let deleted_rooms = room_delete_backend.deleted_rooms.lock().unwrap();
        assert_eq!(*deleted_rooms, vec![event.room]);
    }
}
