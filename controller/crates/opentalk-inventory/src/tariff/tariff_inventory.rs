// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{tariffs::TariffId, users::UserId};

use super::{ExternalTariffId, ExternalTariffMapping, NewTariff, Tariff, UpdateTariff};
use crate::Result;

/// A trait for retrieving and storing tariff entities.
#[async_trait::async_trait]
pub trait TariffInventory {
    /// Get all tariffs.
    async fn get_all_tariffs(&mut self) -> Result<Vec<Tariff>>;

    /// Get a tariff by its id.
    async fn get_tariff(&mut self, tariff_id: TariffId) -> Result<Tariff>;

    /// Get a tariff by its name.
    async fn get_tariff_by_name(&mut self, name: &str) -> Result<Tariff>;

    /// Get the tariff of a user.
    async fn get_tariff_for_user(&mut self, user_id: UserId) -> Result<Tariff>;

    /// Get a tariff by its external tariff id.
    async fn get_tariff_by_external_tariff_id(
        &mut self,
        external_tariff_id: ExternalTariffId,
    ) -> Result<Option<Tariff>>;

    /// Get all external tariff ids for a tariff.
    async fn get_all_external_tariff_ids_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<Vec<ExternalTariffId>>;

    /// Create a tariff.
    async fn create_tariff(&mut self, tariff: NewTariff) -> Result<Tariff>;

    /// Update a tariff.
    async fn update_tariff(&mut self, tariff: Tariff, changeset: UpdateTariff) -> Result<Tariff>;

    /// Delete a tariff.
    async fn delete_tariff(&mut self, tariff_id: TariffId) -> Result<()>;

    /// Delete external all tariff mappings for a tariff.
    async fn delete_all_external_tariff_mappings_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<()>;

    /// Delete external tariff mappings for a tariff by their external tariff id.
    async fn delete_external_tariff_mappings_for_tariff_by_external_id(
        &mut self,
        tariff_id: TariffId,
        external_tariff_ids: &[ExternalTariffId],
    ) -> Result<()>;

    /// Create a mapping from an external tariff id to the system tariff id.
    async fn create_external_tariff_mapping(
        &mut self,
        external_tariff_id: ExternalTariffId,
        tariff_id: TariffId,
    ) -> Result<ExternalTariffMapping>;
}
