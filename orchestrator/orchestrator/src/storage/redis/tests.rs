// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::storage::tests::StorageTester;

#[tokio::test]
async fn add_roomserver_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester
        .add_roomserver_instance("http://localhost:8080")
        .await;
}

#[tokio::test]
async fn add_recorder_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester.add_recorder_instance("http://localhost:8080").await;
}

#[tokio::test]
async fn add_transcription_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester
        .add_transcription_instance("http://localhost:8080")
        .await;
}

#[tokio::test]
async fn add_multiple_services() {
    let tester = StorageTester::new_redis_storage().await;
    tester.add_multiple_services().await;
}

#[tokio::test]
async fn get_unknown_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester.get_unknown_instance().await;
}

#[tokio::test]
async fn get_wrong_instance_kind() {
    let tester = StorageTester::new_redis_storage().await;
    tester.get_wrong_instance_kind().await;
}

#[tokio::test]
async fn remove_service() {
    let tester = StorageTester::new_redis_storage().await;
    tester.remove_instance().await;
}

#[tokio::test]
async fn set_metrics() {
    let tester = StorageTester::new_redis_storage().await;
    tester.set_metrics().await;
}

#[tokio::test]
async fn select_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester.select_instance().await;
}

#[tokio::test]
async fn select_lowest_load_instance() {
    let tester = StorageTester::new_redis_storage().await;
    tester.select_lowest_load_instance().await;
}

#[tokio::test]
async fn select_instance_resource_count_tiebreak() {
    let tester = StorageTester::new_redis_storage().await;
    tester.select_instance_resource_count_tiebreak().await;
}

#[tokio::test]
async fn lowest_load_not_accepting_jobs() {
    let tester = StorageTester::new_redis_storage().await;
    tester.lowest_load_not_accepting_jobs().await;
}

#[tokio::test]
async fn duplicate_url() {
    let tester = StorageTester::new_redis_storage().await;
    tester.duplicate_url().await;
}

#[tokio::test]
async fn duplicate_resource() {
    let tester = StorageTester::new_redis_storage().await;
    tester.duplicate_resource().await;
}

#[tokio::test]
async fn roomserver_token_store() {
    let tester = StorageTester::new_redis_storage().await;
    tester.roomserver_token_store().await;
}

// Redis specific tests

#[tokio::test]
async fn cleanup() {
    let tester = StorageTester::new_redis_storage().await;
    tester.cleanup().await;
}
