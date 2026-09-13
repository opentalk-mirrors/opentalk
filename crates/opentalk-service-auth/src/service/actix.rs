// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Middleware implementation for actix

use std::{
    future::{Ready, ready},
    sync::Arc,
};

use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::ResponseError,
    http::header::AUTHORIZATION,
};
use futures::future::Either;

use crate::service::{
    ApiKeyAuthorization,
    decoding_keys::{DecodingError, DecodingKeys},
    invalid_token_error, missing_token_error, trim_bearer_prefix,
};

impl<S> Transform<S, ServiceRequest> for ApiKeyAuthorization
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthorizationService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthorizationService {
            service,
            decoding_keys: self.decoding_keys.clone(),
        }))
    }
}

pub struct ApiKeyAuthorizationService<S> {
    service: S,
    decoding_keys: Arc<DecodingKeys>,
}

impl<S> Service<ServiceRequest> for ApiKeyAuthorizationService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let Some(header) = req.headers().get(AUTHORIZATION) else {
            return Either::Right(ready(Ok(
                req.into_response(missing_token_error().error_response())
            )));
        };

        let Some(token) = trim_bearer_prefix(header.as_bytes()) else {
            return Either::Right(ready(Ok(req.into_response(
                invalid_token_error(DecodingError::MalformedJwt).error_response(),
            ))));
        };

        if let Err(err) = self.decoding_keys.validate_jwt(token) {
            return Either::Right(ready(Ok(
                req.into_response(invalid_token_error(err).error_response())
            )));
        }

        Either::Left(self.service.call(req))
    }
}
