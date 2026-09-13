// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

// Each `tests/*.rs` file is compiled as its own crate and pulls in this module via `mod common;`. No single test
// binary uses every helper, so items unused by a given binary would otherwise trip `dead_code`.
#![allow(dead_code)]

use opentalk_database::DbConnection;
use opentalk_db_storage::{
    self as db,
    tables::{rooms::NewRoom, users::User},
};
use opentalk_types_common::{
    rooms::{GuestAccess, RoomId, RoomName, RoomSuffix},
    streaming::{StreamingTarget, StreamingTargetId, StreamingTargetKind},
    tariffs::TariffStatus,
    users::UserTitle,
    utils::ExampleData as _,
};

pub async fn make_user(
    conn: &mut DbConnection,
    firstname: &str,
    lastname: &str,
    display_name: &str,
) -> db::tables::users::User {
    let tenant = db::queries::tenants::get_or_create_tenant_by_oidc_id(
        conn,
        &db::tables::tenants::OidcTenantId::from("default".to_owned()),
    )
    .await
    .unwrap();

    let tariff = db::queries::tariffs::get_tariff_by_name(conn, "OpenTalkDefaultTariff")
        .await
        .unwrap();

    let new_user = db::tables::users::NewUser {
        email: format!(
            "{}.{}@example.org",
            firstname.to_lowercase(),
            lastname.to_lowercase()
        ),
        title: UserTitle::new(),
        firstname: firstname.into(),
        lastname: lastname.into(),
        avatar_url: None,
        display_name: display_name.parse().expect("valid display name"),
        language: Some("de".parse().expect("valid language")),
        oidc_sub: format!("{firstname}{lastname}"),
        phone: None,
        tenant_id: tenant.id,
        tariff_id: tariff.id,
        tariff_status: TariffStatus::Default,
        timezone: None,
    };

    db::queries::users::create_user(conn, new_user)
        .await
        .unwrap()
}

pub fn new_room(user: &User, name: Option<RoomName>, suffix: Option<RoomSuffix>) -> NewRoom {
    NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        guest_access: GuestAccess::default(),
        e2e_encryption: false,
        tenant_id: user.tenant_id,
        name,
        suffix,
    }
}

/// Default name used when creating a streaming target in tests.
pub const STREAMING_TARGET_NAME: &str = "Stream";

pub async fn create_streaming_target(
    conn: &mut DbConnection,
    room_id: RoomId,
    name: &str,
) -> StreamingTargetId {
    let created = db::queries::streaming_targets::create_room_streaming_target(
        conn,
        room_id,
        StreamingTarget {
            name: name.to_owned(),
            kind: StreamingTargetKind::example_data(),
        },
    )
    .await
    .unwrap();

    created.id
}
