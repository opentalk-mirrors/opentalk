// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event favorites

use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::events::EventId;

use crate::ControllerBackend;

impl ControllerBackend {
    pub(crate) async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let _event = inventory.get_event(event_id).await?;

        let created = inventory
            .create_event_favorite_for_user(event_id, current_user.id)
            .await?;
        Ok(created)
    }

    pub(crate) async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let existed = inventory
            .delete_event_favorite_for_user(event_id, current_user.id)
            .await?;

        if existed {
            Ok(())
        } else {
            Err(ApiError::not_found().into())
        }
    }
}
