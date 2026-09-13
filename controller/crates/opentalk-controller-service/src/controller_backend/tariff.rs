// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::Tariff;
use opentalk_types_common::{
    rooms::RoomIdOrAlias,
    tariffs::{TariffId, TariffResource},
    users::UserId,
};

use crate::ControllerBackend;

impl ControllerBackend {
    pub(super) async fn get_tariff(
        &self,
        tariff_id: TariffId,
    ) -> Result<TariffResource, CaptureApiError> {
        let tariff = {
            let mut inventory = self.inventory_provider.get_inventory().await?;
            inventory.get_tariff(tariff_id).await
        }?;

        self.build_tariff_resource(&tariff)
    }

    pub(super) async fn get_room_tariff(
        &self,
        room: RoomIdOrAlias,
    ) -> Result<TariffResource, CaptureApiError> {
        let tariff = {
            let mut inventory = self.inventory_provider.get_inventory().await?;
            let room = inventory.get_room(&room).await?;
            inventory.get_tariff_for_user(room.created_by).await
        }?;

        self.build_tariff_resource(&tariff)
    }

    pub(super) async fn get_tariff_for_user(
        &self,
        user_id: UserId,
    ) -> Result<TariffResource, CaptureApiError> {
        let tariff = {
            let mut inventory = self.inventory_provider.get_inventory().await?;
            inventory.get_tariff_for_user(user_id).await
        }?;

        self.build_tariff_resource(&tariff)
    }

    pub(super) fn build_tariff_resource(
        &self,
        tariff: &Tariff,
    ) -> Result<TariffResource, CaptureApiError> {
        let settings = self.settings_provider.get();
        Ok(tariff.to_tariff_resource(
            settings.defaults.disabled_features.clone(),
            self.module_features.clone(),
        ))
    }
}
