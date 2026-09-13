// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::{Settings, UserSearchBackend, UserSearchBackendKeycloak};
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::InventoryProvider;
use opentalk_keycloak_admin::{AuthorizedClient, KeycloakAdminClient};
use opentalk_log::{debug, info};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, ensure};

use crate::{
    Error, Job, JobParameters,
    error::{
        InvalidParameterValueSnafu, KeycloakApiReturnedInvalidUserCountSnafu,
        ParameterLoadingSnafu, ParameterSerializingSnafu, UserSearchBackendIsNotKeycloakSnafu,
    },
    events::update_user_accounts,
};

const DEFAULT_KEYCLOAK_API_PAGE_SIZE: usize = 100;
const DEFAULT_DUMP_FAILED_RESPONSES: bool = false;
const MIN_KEYCLOAK_API_PAGE_SIZE: usize = 1;
const MAX_KEYCLOAK_API_PAGE_SIZE: usize = 10000;

#[derive(Debug)]
pub struct KeycloakAccountSync;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycloakAccountSyncParameters {
    #[serde(default = "default_keycloak_api_page_size")]
    keycloak_api_page_size: usize,
    #[serde(default = "default_dump_failed_responses")]
    dump_failed_responses: bool,
}

impl JobParameters for KeycloakAccountSyncParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

#[async_trait]
impl Job for KeycloakAccountSync {
    type Parameters = KeycloakAccountSyncParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        _authorizer: Authorizer,
        _stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting account cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let KeycloakAccountSyncParameters {
            keycloak_api_page_size,
            dump_failed_responses,
        } = parameters;

        ensure!(
            (MIN_KEYCLOAK_API_PAGE_SIZE..=MAX_KEYCLOAK_API_PAGE_SIZE)
                .contains(&keycloak_api_page_size),
            InvalidParameterValueSnafu {
                parameter_name: "keycloak_api_page_size",
                expected_requirement: format!(
                    "Value between {MIN_KEYCLOAK_API_PAGE_SIZE} and {MAX_KEYCLOAK_API_PAGE_SIZE}"
                ),
            }
        );

        let Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
            api_base_url,
            client_id,
            client_secret,
            external_id_user_attribute_name,
        })) = &settings.user_search_backend
        else {
            return UserSearchBackendIsNotKeycloakSnafu.fail();
        };

        let mut inventory = inventory_provider.get_inventory().await?;

        let users = inventory.get_all_users().await?;

        let authorized_client = AuthorizedClient::new(
            settings.oidc.controller.authority.clone(),
            client_id.clone().into(),
            client_secret.secret().clone(),
        )?;

        let kc_admin_client = KeycloakAdminClient::with_dump_flag(
            api_base_url.clone(),
            authorized_client,
            dump_failed_responses,
        )?;

        let kc_users_count = kc_admin_client.get_users_count().await?;

        let kc_users_count = usize::try_from(kc_users_count).with_context(|_| {
            KeycloakApiReturnedInvalidUserCountSnafu {
                returned_user_count: kc_users_count,
            }
        })?;

        let mut kc_users = vec![];
        let mut offset = 0;

        while offset < kc_users_count {
            let chunk_size = std::cmp::min(keycloak_api_page_size, kc_users_count - offset);
            debug!(log: logger, "Querying {chunk_size} users from Keycloak at offset {offset}.");
            let users_chunk = kc_admin_client
                .get_users_chunk(
                    offset.try_into().expect("Valid i32 number required"),
                    chunk_size.try_into().expect("Valid i32 number required"),
                )
                .await?;
            kc_users.extend(users_chunk);
            offset += chunk_size;
        }

        debug!(log: logger, "Fetched {} users from Keycloak", kc_users.len());

        let user_attribute_name = external_id_user_attribute_name.as_deref();

        update_user_accounts(
            logger,
            inventory.as_mut(),
            users,
            kc_users,
            user_attribute_name,
        )
        .await?;

        Ok(())
    }
}

fn default_keycloak_api_page_size() -> usize {
    DEFAULT_KEYCLOAK_API_PAGE_SIZE
}

fn default_dump_failed_responses() -> bool {
    DEFAULT_DUMP_FAILED_RESPONSES
}
