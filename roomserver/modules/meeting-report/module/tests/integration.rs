// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use insta::assert_snapshot;
use opentalk_roomserver_module_meeting_report::MeetingReportModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_meeting_report::{
    command::MeetingReportCommand,
    event::{MeetingReportError, MeetingReportEvent},
};

#[test_log::test(tokio::test)]
async fn generate_meeting_report() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingReportModule>()
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let _gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_command::<MeetingReportModule>(
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: true,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingReportModule>()
        .await
        .unwrap()
        .payload;
    let MeetingReportEvent::PdfAsset { asset_id, .. } = event else {
        panic!("Expected PdfAsset event, got {event:#?}");
    };

    let file = room.stored_asset(asset_id).await.unwrap();
    // Title, details, start & end are missing because they do not exist in the
    // TestRoom
    let content = pdf_extract::extract_text_from_mem(&file).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
    ]}, {
        assert_snapshot!(content, @r"
        Attendance Report
         Meeting :

        Report created at : [timestamp]

        Report timezone : Europe/Berlin

        Participants
         Nr Name Role

        1 Alice the angry Moderator

        2 Bob the bold User

        3 Gustav the great Guest
        ")
    });
}

#[test_log::test(tokio::test)]
async fn quota_exceeded() {
    let mut room = TestRoom::builder()
        .storage_quota(0)
        .register_module::<MeetingReportModule>()
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MeetingReportModule>(
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: false,
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice
            .receive_event::<MeetingReportModule>()
            .await
            .unwrap()
            .payload,
        MeetingReportEvent::Error(MeetingReportError::StorageExceeded)
    );
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingReportModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<MeetingReportModule>(
        MeetingReportCommand::GenerateAttendanceReport {
            include_email_addresses: true,
        },
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        bob.receive_event::<MeetingReportModule>()
            .await
            .unwrap()
            .payload,
        MeetingReportEvent::Error(MeetingReportError::InsufficientPermissions)
    );
}
