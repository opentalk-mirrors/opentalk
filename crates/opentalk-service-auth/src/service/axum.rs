// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Middleware implementation for axum

use std::{
    future::{Ready, ready},
    sync::Arc,
};

use axum::{extract::Request, http::header::AUTHORIZATION, response::IntoResponse};
use futures::future::Either;
use tower::{Layer, Service};

use crate::service::{
    ApiKeyAuthorization,
    decoding_keys::{DecodingError, DecodingKeys},
    invalid_token_error, missing_token_error, trim_bearer_prefix,
};

impl<S> Layer<S> for ApiKeyAuthorization {
    type Service = ApiKeyAuthorizationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ApiKeyAuthorizationService {
            inner,
            decoding_keys: self.decoding_keys.clone(),
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ApiKeyAuthorizationService<S> {
    inner: S,
    decoding_keys: Arc<DecodingKeys>,
}

impl<S> Service<Request> for ApiKeyAuthorizationService<S>
where
    S: Service<Request, Response = axum::response::Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let Some(header) = req.headers().get(AUTHORIZATION) else {
            return Either::Right(ready(Ok(missing_token_error().into_response())));
        };

        let Some(token) = trim_bearer_prefix(header.as_bytes()) else {
            return Either::Right(ready(Ok(
                invalid_token_error(DecodingError::MalformedJwt).into_response()
            )));
        };

        if let Err(err) = self.decoding_keys.validate_jwt(token) {
            return Either::Right(ready(Ok(invalid_token_error(err).into_response())));
        }

        Either::Left(self.inner.call(req))
    }
}
