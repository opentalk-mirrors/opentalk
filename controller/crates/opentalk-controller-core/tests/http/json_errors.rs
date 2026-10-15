// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Invalid JSON request bodies are answered with an error body containing a machine-readable
//! code. The error messages are produced by the web framework and therefore not checked.

use opentalk_controller_api_authorization::authorization::Admission;
use pretty_assertions::assert_eq;
use reqwest::{StatusCode, header::CONTENT_TYPE};

use crate::harness::{TestApp, TestServer};

fn start_with_public_login() -> TestServer {
    let mut app = TestApp::new();
    app.authorizer
        .expect_authorize()
        .returning(|_| Ok(Admission::Allowed));
    app.start()
}

async fn error_code(response: reqwest::Response) -> String {
    let body: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    body["code"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn malformed_json_body() {
    let server = start_with_public_login();

    let response = server
        .post("/v1/auth/login")
        .header(CONTENT_TYPE, "application/json")
        .body("{")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/json; charset=utf-8");
    assert_eq!(error_code(response).await, "invalid_json");
}

#[tokio::test]
async fn missing_content_type() {
    let server = start_with_public_login();

    let response = server
        .post("/v1/auth/login")
        .body(r#"{"id_token":"token"}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(response).await, "invalid_content_type");
}
