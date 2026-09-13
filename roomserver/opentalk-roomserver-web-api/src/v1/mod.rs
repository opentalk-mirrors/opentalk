// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use super::Router;
pub use crate::v1::rooms::{RoomAction, RoomBackend};
use crate::{
    livekit_proxy::LiveKitProxyBackend,
    v1::{signaling::SignalingBackend, user::UserBackend},
};

pub mod cors;
pub mod rooms;
pub mod signaling;
pub mod user;

use opentalk_service_auth::service::ApiKeyAuthorization;
use utoipa::openapi::security::{Http, HttpAuthScheme};

pub trait Backend:
    Send + Sync + Clone + Sized + RoomBackend + UserBackend + SignalingBackend + LiveKitProxyBackend
{
}

pub fn routes<B: Backend + 'static>(state: B, auth_middleware: ApiKeyAuthorization) -> Router<B> {
    Router::<B>::new()
        .merge(rooms::routes())
        .merge(user::routes())
        .layer(auth_middleware)
        .merge(signaling::routes(state))
}

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::SecurityScheme;

        let components = openapi.components.as_mut().unwrap();

        let http_scheme = Http::builder()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("api token")
            .description("The roomservers API token is expected to be in the `Authorization` header with the format: `bearer <token>`".into())
            .build();

        components.add_security_scheme("API-Token", SecurityScheme::Http(http_scheme));

        let http_scheme = Http::builder()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .description("The access token to authorize against the LiveKit server. This can also be provided via query parameters. This token is expected in the `Authorization` header with the format: `bearer <token>`".into())
            .build();

        components.add_security_scheme("Livekit-Token", SecurityScheme::Http(http_scheme));
    }
}
