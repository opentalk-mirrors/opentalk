// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

// These tests can unfortunately not be run in the CI. They require an up-to-date roomserver
// container to reflect the latest API. Running them in the CI would require building such a
// container before the tests, otherwise changes to the roomserver API could break the tests.

use std::{assert_matches, str::FromStr};

use opentalk_roomserver_client::{
    ApiError, Client, Error, PostStorageQuotaError, RequestTokenError,
};
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, ClientParameters},
    public_user_profile::PublicUserProfile,
    room_kind::RoomKind,
    room_parameters::RoomParameters,
    room_parameters_patch::RoomParametersPatch,
};
use opentalk_service_auth::ApiKey;
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{
    events::EventTitle,
    rooms::{RoomId, RoomPassword},
    roomserver::DeviceSecret,
    users::UserId,
    utils::ExampleData,
};
use opentalk_types_signaling::Role;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor, logs::consumer::logging_consumer::LoggingConsumer},
    runners::AsyncRunner,
};
use url::Url;

const ROOMSERVER_PORT: u16 = 11333;

fn api_key() -> ApiKey {
    ApiKey::new("roomserver", "secret")
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn put_room() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    client
        .put_room(RoomId::from_u128(0x1), RoomParameters::example_data())
        .await
        .unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn patch_room() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());
    let room_id = RoomId::from_u128(0x1);

    client
        .put_room(room_id, RoomParameters::example_data())
        .await
        .unwrap();

    let patch = RoomParametersPatch {
        password: Some(Some(RoomPassword::from_str("new password").unwrap())),
        title: Some(EventTitle::from_str_lossy("New Event Title")),
        show_meeting_details: None,
    };
    client.patch_room(room_id, patch).await.unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn delete_room() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());
    let room_id = RoomId::from_u128(0x1);

    client
        .put_room(room_id, RoomParameters::example_data())
        .await
        .unwrap();

    client.delete_room(room_id).await.unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn delete_non_existing_room() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());
    let room_id = RoomId::from_u128(0x1);

    client.delete_room(room_id).await.unwrap();
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn post_storage_quota_not_found() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());
    let user_id = UserId::from_u128(0x1);
    let quota = Quota {
        total: Some(2048),
        used: 1024,
    };

    let err = client.post_storage_quota(user_id, quota).await.unwrap_err();
    assert_matches!(err, Error::ApiError(ApiError {
            code, ..
        }) if code == PostStorageQuotaError::NotFound);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn post_storage_quota() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());
    let user_id = UserId::from_u128(0x1);
    let quota = Quota {
        total: Some(2048),
        used: 1024,
    };
    let room_id = RoomId::from_u128(0x1);

    // Create a a room with the user id as creator
    let parameters = RoomParameters {
        created_by: PublicUserProfile {
            id: user_id,
            ..PublicUserProfile::example_data()
        },
        ..RoomParameters::example_data()
    };
    client
        .put_room(room_id, parameters)
        .await
        .expect("Failed to create room");

    client
        .post_storage_quota(user_id, quota)
        .await
        .expect("Failed to post storage quota");
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_token() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    let access = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            Some(RoomParameters::example_data()),
        )
        .await
        .unwrap();

    // Requesting a new token yields a different token each time
    let access2 = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            Some(RoomParameters::example_data()),
        )
        .await
        .unwrap();

    assert_ne!(access.token, access2.token);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_token_without_room_params() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    let error = client
        .request_token(
            RoomId::from_u128(0x1),
            ClientParameters::example_data(),
            None,
        )
        .await
        .unwrap_err();

    let Error::ApiError(api_error) = error else {
        panic!("Expected ApiError, received {error:#?}");
    };

    assert_eq!(api_error.code, RequestTokenError::RoomParametersMissing);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_recorder_token_e2ee_enabled() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    let room_id = RoomId::from_u128(1);
    let recorder_client_params = ClientParameters {
        device_secret: DeviceSecret::from_str("recorder device secret")
            .expect("Device secret must be valid"),
        kind: ClientKind::Recorder {
            room: RoomKind::Main,
        },
        role: Role::User,
    };
    let room_parameters = RoomParameters {
        e2e_encryption: true,
        ..RoomParameters::example_data()
    };

    // Recorders requesting a token for a new room fails when e2ee is enabled
    let error = client
        .request_token(room_id, recorder_client_params, Some(room_parameters))
        .await
        .unwrap_err();

    let Error::ApiError(api_error) = error else {
        panic!("Expected ApiError, received {error:#?}");
    };

    assert_eq!(api_error.code, RequestTokenError::E2eeEnabled);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_call_in_token_e2ee_enabled() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    let room_id = RoomId::from_u128(1);
    let call_in_client_params = ClientParameters {
        device_secret: DeviceSecret::from_str("call-in device secret")
            .expect("Device secret must be valid"),
        kind: ClientKind::CallIn,
        role: Role::User,
    };
    let room_parameters = RoomParameters {
        e2e_encryption: true,
        ..RoomParameters::example_data()
    };

    // Requesting a call-in token for a new room fails when e2ee is enabled
    let error = client
        .request_token(room_id, call_in_client_params, Some(room_parameters))
        .await
        .unwrap_err();

    let Error::ApiError(api_error) = error else {
        panic!("Expected ApiError, received {error:#?}");
    };

    assert_eq!(api_error.code, RequestTokenError::E2eeEnabled);
}

#[test_log::test(tokio::test)]
#[ignore = "Requires an up-to-date roomserver container"]
async fn request_token_guest_access_disabled() {
    let (_container, base_url) = spawn_roomserver().await;
    let client = Client::new(base_url, api_key());

    let room_id = RoomId::from_u128(1);
    let guest_client_params = ClientParameters {
        device_secret: DeviceSecret::from_str("guest device secret")
            .expect("Device secret must be valid"),
        kind: ClientKind::Unregistered,
        role: Role::User,
    };
    let room_parameters = RoomParameters {
        guest_access: false,
        ..RoomParameters::example_data()
    };

    // Guests requesting a token for a new room fails when guest access is disabled
    let error = client
        .request_token(
            room_id,
            guest_client_params.clone(),
            Some(room_parameters.clone()),
        )
        .await
        .unwrap_err();

    let Error::ApiError(api_error) = error else {
        panic!("Expected ApiError, received {error:#?}");
    };

    assert_eq!(api_error.code, RequestTokenError::GuestAccessDisabled);
}

async fn spawn_roomserver() -> (ContainerAsync<GenericImage>, Url) {
    ensure_crypto_provider();

    let config = std::env::current_dir()
        .unwrap()
        .join("../example/roomserver.toml");

    let container = GenericImage::new(
        "git.opentalk.dev:5050/opentalk/backend/services/roomserver",
        "latest",
    )
    .with_wait_for(WaitFor::message_on_stdout("Listening on"))
    .with_exposed_port(ROOMSERVER_PORT.tcp())
    .with_network("bridge")
    .with_mount(Mount::bind_mount(
        config.to_string_lossy(),
        "/etc/opentalk/roomserver.toml",
    ))
    .with_log_consumer(LoggingConsumer::new())
    .start()
    .await
    .unwrap();

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(ROOMSERVER_PORT).await.unwrap();
    let base_url = format!("http://{host}:{port}");

    (container, base_url.parse().unwrap())
}
