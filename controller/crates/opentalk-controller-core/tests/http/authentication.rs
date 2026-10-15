// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{
    AccessMethod, Admission, Resource, Subject,
};
use opentalk_inventory::User;
use opentalk_types_api_v1::users::PrivateUserProfile;
use opentalk_types_common::{
    tariffs::TariffStatus,
    users::{DisplayName, Language, UserId, UserTitle},
    utils::ExampleData as _,
};
use pretty_assertions::assert_eq;
use reqwest::StatusCode;

use crate::harness::{ACCESS_TOKEN, TestApp};

const USER_ID: UserId = UserId::from_u128(0x0001);

#[tokio::test]
async fn unauthenticated_request_to_protected_resource_is_unauthorized() {
    let mut app = TestApp::new();
    app.authorizer
        .expect_authorize()
        .withf(|target| {
            target.resource == Resource::UserMe
                && !target.authenticated_subjects.contains_any_user()
        })
        .returning(|_| Ok(Admission::AuthenticationRequired));
    let server = app.start();

    let response = server.get("/v1/users/me").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.text().await.unwrap(), "");
}

#[tokio::test]
async fn denied_request_is_forbidden() {
    let mut app = TestApp::new();
    app.authorizer
        .expect_authorize()
        .returning(|_| Ok(Admission::Denied));
    let server = app.start();

    let response = server.get("/v1/users/me").send().await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.text().await.unwrap(), "");
}

#[tokio::test]
async fn authenticated_user_reaches_authorization_and_handler() {
    let mut app = TestApp::new();
    app.authenticate(User {
        id: USER_ID,
        ..User::example_data()
    })
    .await;
    app.authorizer
        .expect_authorize()
        .withf(|target| {
            target.resource == Resource::UserMe
                && target.access_method == AccessMethod::Get
                && target
                    .authenticated_subjects
                    .contains(&Subject::User(USER_ID))
        })
        .returning(|_| Ok(Admission::Allowed));
    app.service
        .expect_get_me()
        .withf(|current_user| current_user.id == USER_ID)
        .returning(|current_user| Box::pin(std::future::ready(Ok(profile(current_user.id)))));
    let server = app.start();

    let response = server
        .get("/v1/users/me")
        .bearer_auth(ACCESS_TOKEN)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(body["id"], USER_ID.to_string());
}

fn profile(id: UserId) -> PrivateUserProfile {
    PrivateUserProfile {
        id,
        email: "john.doe@example.com".to_owned(),
        title: UserTitle::example_data(),
        firstname: "John".to_owned(),
        lastname: "Doe".to_owned(),
        display_name: DisplayName::example_data(),
        avatar_url: "https://example.com/avatar.png".to_owned(),
        dashboard_theme: None,
        conference_theme: None,
        language: Language::example_data(),
        tariff_status: TariffStatus::Paid,
        used_storage: 0,
    }
}
