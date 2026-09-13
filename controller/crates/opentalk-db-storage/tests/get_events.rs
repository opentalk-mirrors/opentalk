// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::TimeZone as _;
use chrono_tz::Tz;
use opentalk_database::DbConnection;
use opentalk_db_storage::{
    self as db,
    tables::{event_invites::NewEventInvite, rooms::NewRoom, users::User},
};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventDescription, EventTitle, invites::InviteRole},
    rooms::GuestAccess,
    time::TimeZone,
    utils::ExampleData,
};

use crate::common::make_user;

mod common;

async fn make_event(
    conn: &mut DbConnection,
    user: &User,
    hour: Option<u32>,
    is_adhoc: bool,
) -> db::queries::events::types::EventRecord {
    let tenant = db::queries::tenants::get_or_create_tenant_by_oidc_id(
        conn,
        &db::tables::tenants::OidcTenantId::from("default".to_string()),
    )
    .await
    .unwrap();

    let room = {
        let room = NewRoom {
            created_by: user.id,
            password: None,
            waiting_room: false,
            guest_access: GuestAccess::default(),
            e2e_encryption: false,
            tenant_id: user.tenant_id,
            name: None,
            suffix: None,
        };

        db::queries::rooms::create_room(conn, room).await.unwrap()
    };

    let date = hour.map(|h| inventory::NewEventDate {
        is_all_day: false,
        starts_at: Tz::UTC.with_ymd_and_hms(2020, 1, 1, h, 0, 0).unwrap(),
        starts_at_tz: TimeZone::from(Tz::UTC),
        ends_at: Tz::UTC.with_ymd_and_hms(2020, 1, 1, h, 0, 0).unwrap(),
        ends_at_tz: TimeZone::from(Tz::UTC),
        duration_secs: 1800,
        recurrence: None,
    });

    let new_event = inventory::NewEvent {
        title: EventTitle::example_data(),
        description: EventDescription::example_data(),
        room: room.id,
        created_by: user.id,
        updated_by: user.id,
        is_adhoc,
        tenant_id: tenant.id,
        show_meeting_details: false,
        date,
    };

    db::queries::events::create_event(conn, new_event.into())
        .await
        .unwrap()
}

#[tokio::test]
async fn get_event_invites() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;

    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let ferdinand = make_user(&mut conn, "Ferdinand", "Jaegermeister", "ferdemeister").await;
    let louise = make_user(&mut conn, "Jeez", "Louise", "Jesus").await;
    let gerhard = make_user(&mut conn, "Gerhard", "Bauer", "Hardi").await;

    // EVENT 1 MIT JEEZ LOUISE AND GERHARD
    let event1 = make_event(&mut conn, &ferdinand, Some(1), true).await;

    let new_event_invite = NewEventInvite {
        event_id: event1.id(),
        invitee: louise.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::User,
    };
    db::queries::events::try_create_event_invite(&mut conn, new_event_invite)
        .await
        .unwrap();

    let new_event_invite = NewEventInvite {
        event_id: event1.id(),
        invitee: gerhard.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::User,
    };
    db::queries::events::try_create_event_invite(&mut conn, new_event_invite)
        .await
        .unwrap();

    // EVENT 2 MIT JEEZ LOUSE UND FERDINAND
    let event2 = make_event(&mut conn, &gerhard, Some(1), true).await;

    let new_event_invite = NewEventInvite {
        event_id: event2.id(),
        invitee: louise.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::User,
    };
    db::queries::events::try_create_event_invite(&mut conn, new_event_invite)
        .await
        .unwrap();

    let new_event_invite = NewEventInvite {
        event_id: event2.id(),
        invitee: ferdinand.id,
        created_by: ferdinand.id,
        created_at: None,
        role: InviteRole::User,
    };
    db::queries::events::try_create_event_invite(&mut conn, new_event_invite)
        .await
        .unwrap();

    let events = &[event1.event(), event2.event()][..];

    let invites_with_invitees =
        db::queries::events::get_event_user_invites_for_events(&mut conn, events)
            .await
            .unwrap();

    for (event, invites_with_users) in events.iter().zip(invites_with_invitees) {
        println!("Event: {event:#?}");
        println!(
            "Invitees: {:#?}",
            invites_with_users
                .into_iter()
                .map(|x| x.1)
                .collect::<Vec<_>>()
        );
        println!("#################################################")
    }
}
