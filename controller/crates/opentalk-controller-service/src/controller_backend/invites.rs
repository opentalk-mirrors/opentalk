// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::utils::is_room_guest_access_allowed;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::invites::{PostInviteVerifyRequestBody, PostInviteVerifyResponseBody},
};

use crate::ControllerBackend;

impl ControllerBackend {
    pub(crate) async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let invite = inventory.get_room_invite(data.invite_code).await?;
        let room = inventory.get_room(&invite.room.into()).await?;
        let tariff = self.get_room_tariff(room.id.into()).await?;

        if !is_room_guest_access_allowed(&room, &tariff) {
            // Do not leak the existence of the invite
            return Err(ApiError::not_found().into());
        }

        Ok(PostInviteVerifyResponseBody {
            room_id: invite.room,
            password_required: room.password.is_some(),
        })
    }
}
