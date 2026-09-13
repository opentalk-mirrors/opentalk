// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr as _;

use axum::http::{HeaderValue, Uri};
use opentalk_types_common::roomserver::Token;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};
use tracing::{Instrument, Level, span};

use crate::v1::signaling::SignalingBackend;

pub(super) fn cors_layer<B: SignalingBackend + 'static>(
    state: B,
    allowed_methods: impl Into<AllowMethods>,
) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::async_predicate(|origin, parts| {
            let token = extract_token(&parts.uri);
            handle_request(state, origin, token).instrument(span!(Level::INFO, "CORS"))
        }))
        .allow_methods(allowed_methods)
}

fn extract_token(uri: &Uri) -> Option<Token> {
    uri.path()
        .split('/')
        .next_back()
        .and_then(|s| Token::from_str(s).ok())
}

async fn handle_request<B: SignalingBackend + 'static>(
    state: B,
    origin: HeaderValue,
    token: Option<Token>,
) -> bool {
    let Some(token) = token else {
        tracing::debug!("Received request with invalid token");
        return false;
    };

    let Some(room_id) = state.room_id(token).await else {
        tracing::error!("Failed to determine room id");
        return false;
    };

    let Ok(allowed_origins, ..) = state.allowed_origins(room_id).await else {
        tracing::error!("Failed to determine allowed origins");
        return false;
    };

    if allowed_origins.contains(&"*".to_string()) {
        return true;
    }

    let Ok(origin) = origin.to_str().map(str::to_owned) else {
        tracing::error!("Failed to serialize origin");
        return false;
    };

    let allow = allowed_origins.contains(&origin);
    if !allow {
        tracing::debug!(
            "Received request from origin {origin}, which is not in the allowed origins \
                                    list for room {room_id}.\n\
                                    Allowed origins are: {}",
            allowed_origins.join(", ")
        );
    }

    allow
}

#[cfg(test)]
mod tests {
    use axum::http::Uri;

    #[test]
    fn extract_token_valid() {
        let uri = Uri::from_static("/signaling/62b18d13-5816-4754-8071-5af56300ce7e");
        let token = super::extract_token(&uri);
        assert_eq!(
            token.unwrap().to_string(),
            "62b18d13-5816-4754-8071-5af56300ce7e"
        );
    }

    #[test]
    fn extract_token_invalid() {
        let uri = Uri::from_static("/signaling/");
        let token = super::extract_token(&uri);
        assert!(token.is_none());
    }
}
