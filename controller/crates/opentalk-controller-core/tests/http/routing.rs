// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::json;

use crate::harness::TestApp;

#[tokio::test]
async fn well_known_api_description() {
    let server = TestApp::new().start();

    let response = server
        .get("/.well-known/opentalk/api")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(
        body,
        json!({ "opentalk_api": { "v1": { "base_url": "v1" } } })
    );
}

/// Unknown paths below `/v1` cannot be mapped to an authorization resource, so access is denied
/// instead of responding with `404 Not Found`.
#[tokio::test]
async fn unknown_v1_path_is_forbidden() {
    let mut app = TestApp::new();
    app.authorizer.expect_authorize().never();
    let server = app.start();

    let response = server.get("/v1/does-not-exist").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unknown_path_outside_v1_is_not_found() {
    let server = TestApp::new().start();

    let response = server.get("/does-not-exist").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
