// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains tariffs database queries

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{tariffs::TariffId, users::UserId};

use crate::{
    schema::{external_tariffs, tariffs, users},
    tables::tariffs::{ExternalTariff, ExternalTariffId, NewTariff, Tariff, UpdateTariff},
};

pub async fn get_tariff(conn: &mut DbConnection, id: TariffId) -> Result<Tariff> {
    tariffs::table
        .filter(tariffs::id.eq(id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_all_tariffs(conn: &mut DbConnection) -> Result<Vec<Tariff>> {
    tariffs::table.load(conn).await.map_err(DatabaseError::from)
}

pub async fn get_tariff_by_name(conn: &mut DbConnection, name: &str) -> Result<Tariff> {
    tariffs::table
        .filter(tariffs::name.eq(name))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn delete_tariff(conn: &mut DbConnection, id: TariffId) -> Result<()> {
    _ = diesel::delete(tariffs::table)
        .filter(tariffs::id.eq(id))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_tariff_by_external_tariff_id(
    conn: &mut DbConnection,
    id: &ExternalTariffId,
) -> Result<Tariff> {
    external_tariffs::table
        .filter(external_tariffs::external_id.eq(id))
        .inner_join(tariffs::table)
        .select(tariffs::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_tariff_for_user(conn: &mut DbConnection, id: &UserId) -> Result<Tariff> {
    users::table
        .filter(users::id.eq(id))
        .inner_join(tariffs::table)
        .select(tariffs::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn create_tariff(conn: &mut DbConnection, new_tariff: NewTariff) -> Result<Tariff> {
    diesel::insert_into(tariffs::table)
        .values(new_tariff)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn update_tariff(
    conn: &mut DbConnection,
    update_tariff: UpdateTariff,
    tariff_id: TariffId,
) -> Result<Tariff> {
    diesel::update(tariffs::table.filter(tariffs::id.eq(tariff_id)))
        .set(update_tariff)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_all_for_tariff(
    conn: &mut DbConnection,
    tariff_id: TariffId,
) -> Result<Vec<ExternalTariffId>> {
    external_tariffs::table
        .filter(external_tariffs::tariff_id.eq(tariff_id))
        .select(external_tariffs::external_id)
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn delete_all_external_tariff_mappings_for_tariff(
    conn: &mut DbConnection,
    tariff_id: TariffId,
) -> Result<()> {
    _ = diesel::delete(external_tariffs::table)
        .filter(external_tariffs::tariff_id.eq(tariff_id))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn delete_external_tariff_mappings_for_tariff_by_external_id(
    conn: &mut DbConnection,
    tariff_id: TariffId,
    external_ids: &[ExternalTariffId],
) -> Result<()> {
    _ = diesel::delete(external_tariffs::table)
        .filter(
            external_tariffs::tariff_id
                .eq(tariff_id)
                .and(external_tariffs::external_id.eq_any(external_ids)),
        )
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn create_external_tariff_mapping(
    conn: &mut DbConnection,
    external_id: ExternalTariffId,
    tariff_id: TariffId,
) -> Result<ExternalTariff> {
    let external_tariff = ExternalTariff {
        external_id,
        tariff_id,
    };

    diesel::insert_into(external_tariffs::table)
        .values(external_tariff)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}
