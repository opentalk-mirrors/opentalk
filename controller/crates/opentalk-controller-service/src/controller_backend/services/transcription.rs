// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_internal::recording::RecordingTarget;
use opentalk_types_api_v1::rooms::by_room_id::RoomserverStartResponseBody;
use url::Url;

use crate::{
    ControllerBackend,
    controller_backend::services::{ServiceKind, build_roomserver_params},
};

impl ControllerBackend {
    pub(crate) async fn start_transcription_impl(
        &self,
        body: RecordingTarget,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (room_resource, client_parameters) = build_roomserver_params(
            &settings,
            &mut *inventory,
            body.room_id,
            body.breakout_room,
            ServiceKind::Transcription,
        )
        .await?;

        let access = self
            .roomserver
            .request_access(
                inventory.as_mut(),
                settings,
                self.module_features.clone(),
                room_resource,
                client_parameters,
                host,
            )
            .await?;

        Ok(RoomserverStartResponseBody {
            token: access.token,
            roomserver_address: access.public_url.to_string(),
        })
    }
}
