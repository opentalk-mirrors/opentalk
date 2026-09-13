// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Extractor for the HTTP `Host` header as a parsed [`Url`].

use std::future::{Ready, ready};

use actix_web::{FromRequest, dev::Payload};
use opentalk_types_api_v1::error::ApiError;
use url::Url;

/// Extractor for the HTTP `Host` header parsed as a [`Url`].
#[derive(Debug)]

pub struct Host(Url);

impl Host {
    /// Unwrap into inner [`Url`] value
    pub fn into_inner(self) -> Url {
        self.0
    }
}

impl FromRequest for Host {
    type Error = ApiError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut Payload) -> Self::Future {
        let conn = req.connection_info();
        let host = format!(
            "{scheme}://{host}",
            scheme = conn.scheme(),
            host = conn.host()
        );

        let result = Url::parse(&host)
            .map_err(|err| {
                tracing::error!("Failed to parse host: {err}");
                ApiError::internal().with_message("Failed to parse host name")
            })
            .map(Host);

        ready(result)
    }
}
