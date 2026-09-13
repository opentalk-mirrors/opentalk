// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::OptionalExt as _;
use opentalk_db_storage as db;
use opentalk_inventory::{
    ExternalTariffId, ExternalTariffMapping, NewTariff, Tariff, TariffInventory, UpdateTariff,
};
use opentalk_types_common::{tariffs::TariffId, users::UserId};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl TariffInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_tariffs(&mut self) -> Result<Vec<Tariff>> {
        Ok(db::queries::tariffs::get_all_tariffs(&mut self.inner)
            .await
            .context(DatabaseSnafu)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_tariff(&mut self, tariff_id: TariffId) -> Result<Tariff> {
        Ok(db::queries::tariffs::get_tariff(&mut self.inner, tariff_id)
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_tariff_by_name(&mut self, tariff_name: &str) -> Result<Tariff> {
        Ok(
            db::queries::tariffs::get_tariff_by_name(&mut self.inner, tariff_name)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_tariff_for_user(&mut self, user_id: UserId) -> Result<Tariff> {
        Ok(
            db::queries::tariffs::get_tariff_for_user(&mut self.inner, &user_id)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_tariff_by_external_tariff_id(
        &mut self,
        external_tariff_id: ExternalTariffId,
    ) -> Result<Option<Tariff>> {
        Ok(db::queries::tariffs::get_tariff_by_external_tariff_id(
            &mut self.inner,
            &external_tariff_id.into(),
        )
        .await
        .optional()
        .context(DatabaseSnafu)?
        .map(Into::into))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_external_tariff_ids_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<Vec<ExternalTariffId>> {
        Ok(
            db::queries::tariffs::get_all_for_tariff(&mut self.inner, tariff_id)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_tariff(&mut self, tariff: NewTariff) -> Result<Tariff> {
        Ok(
            db::queries::tariffs::create_tariff(&mut self.inner, tariff.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_tariff(&mut self, tariff: Tariff, changeset: UpdateTariff) -> Result<Tariff> {
        Ok(
            db::queries::tariffs::update_tariff(&mut self.inner, changeset.into(), tariff.id)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_tariff(&mut self, tariff_id: TariffId) -> Result<()> {
        Ok(
            db::queries::tariffs::delete_tariff(&mut self.inner, tariff_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    async fn delete_all_external_tariff_mappings_for_tariff(
        &mut self,
        tariff_id: TariffId,
    ) -> Result<()> {
        Ok(
            db::queries::tariffs::delete_all_external_tariff_mappings_for_tariff(
                &mut self.inner,
                tariff_id,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_external_tariff_mappings_for_tariff_by_external_id(
        &mut self,
        tariff_id: TariffId,
        external_tariff_ids: &[ExternalTariffId],
    ) -> Result<()> {
        Ok(
            db::queries::tariffs::delete_external_tariff_mappings_for_tariff_by_external_id(
                &mut self.inner,
                tariff_id,
                &external_tariff_ids
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_external_tariff_mapping(
        &mut self,
        external_tariff_id: ExternalTariffId,
        tariff_id: TariffId,
    ) -> Result<ExternalTariffMapping> {
        Ok(db::queries::tariffs::create_external_tariff_mapping(
            &mut self.inner,
            external_tariff_id.into(),
            tariff_id,
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }
}
