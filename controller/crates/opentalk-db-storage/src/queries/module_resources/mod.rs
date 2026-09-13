// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains module resource database queries

use diesel::{prelude::*, sql_types::Jsonb};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_inventory as inventory;
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId};

use crate::{
    schema::module_resources,
    tables::module_resources::{ModuleResource, NewModuleResource},
};

pub mod types;

use types::{Filter, JsonOperationError};

define_sql_function! {
    fn ot_patch_json(target: Jsonb, changeset: Jsonb) -> Jsonb;
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_module_resource(
    conn: &mut DbConnection,
    filter: Filter,
) -> Result<Vec<ModuleResource>> {
    let filter = filter
        .into_diesel_filter()
        .ok_or_else(|| DatabaseError::Custom {
            message: "Missing filter for get module_resource query".to_owned(),
        })?;

    module_resources::table
        .filter(filter)
        .get_results(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_module_ids_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<ModuleResourceId>> {
    module_resources::table
        .select(module_resources::id)
        .filter(module_resources::room_id.eq(room_id))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_module_resources(
    conn: &mut DbConnection,
    filter: Filter,
) -> Result<Vec<ModuleResource>> {
    let filter = filter
        .into_diesel_filter()
        .ok_or_else(|| DatabaseError::Custom {
            message: "Missing filter for delete query".to_owned(),
        })?;

    diesel::delete(module_resources::table)
        .filter(filter)
        .get_results(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_all_module_resources_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<()> {
    let _ = diesel::delete(module_resources::table)
        .filter(module_resources::room_id.eq(room_id))
        .execute(conn)
        .await?;

    Ok(())
}

/// Update the targeted json data by a set of json patch
/// [ModuleResourceOperations](inventory::ModuleResourceOperation).
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn patch_module_resources(
    conn: &mut DbConnection,
    filter: Filter,
    operations: Vec<inventory::ModuleResourceOperation>,
) -> Result<Vec<ModuleResource>, JsonOperationError> {
    let filter = filter
        .into_diesel_filter()
        .ok_or_else(|| JsonOperationError::Database {
            source: DatabaseError::Custom {
                message: "Missing filter for patch query".to_owned(),
            },
        })?;

    diesel::update(module_resources::table)
        .filter(filter)
        .set(module_resources::data.eq(ot_patch_json(
            module_resources::data,
            serde_json::to_value(operations).unwrap(),
        )))
        .get_results(conn)
        .await
        .map_err(JsonOperationError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_module_resource(
    conn: &mut DbConnection,
    new_module_resource: NewModuleResource,
) -> Result<ModuleResource> {
    diesel::insert_into(module_resources::table)
        .values(new_module_resource)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}
