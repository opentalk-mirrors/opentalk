// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use opentalk_asset_storage::verify_storage_usage;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::Inventory;
use opentalk_roomserver_types::{
    breakout::breakout_id::BreakoutId,
    client_parameters::{ClientKind, ClientParameters, Role},
    room_kind::RoomKind,
};
use opentalk_types_api_v1::{error::ApiError, rooms::RoomResource};
use opentalk_types_common::{
    rooms::RoomId,
    roomserver::{DEVICE_SECRET_MIN_LENGTH, DeviceSecret},
};
use rand::RngExt;

use crate::ToUserProfile;

mod call_in;
mod recording;
mod transcription;

pub(crate) enum ServiceKind {
    Recording,
    Transcription,
}

pub(crate) async fn build_roomserver_params(
    settings: &Settings,
    inventory: &mut dyn Inventory,
    room_id: RoomId,
    breakout_room: Option<u32>,
    service_kind: ServiceKind,
) -> Result<(RoomResource, ClientParameters), CaptureApiError> {
    let (room, creator) = inventory.get_room_with_creator(&room_id.into()).await?;

    let _ = verify_storage_usage(inventory, room.created_by).await?;

    let room_resource = RoomResource {
        id: room.id,
        alias: room.alias,
        created_by: creator.to_public_user_profile(settings),
        created_at: room.created_at,
        password: room.password,
        guest_access: room.guest_access,
        waiting_room: room.waiting_room,
    };

    let device_secret = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(DEVICE_SECRET_MIN_LENGTH)
        .map(|c| c as char)
        .collect::<String>();
    let device_secret = DeviceSecret::from_str(&device_secret).map_err(|e| {
        log::error!("Failed to generate device secret: {e}");
        ApiError::internal().with_message("Failed to generate device secret")
    })?;

    let room = if let Some(breakout_id) = breakout_room {
        RoomKind::Breakout(BreakoutId::from(breakout_id))
    } else {
        RoomKind::Main
    };

    let client_parameters = ClientParameters {
        device_secret,
        kind: match service_kind {
            ServiceKind::Recording => ClientKind::Recorder { room },
            ServiceKind::Transcription => ClientKind::Transcription { room },
        },
        role: Role::User,
    };

    Ok((room_resource, client_parameters))
}
