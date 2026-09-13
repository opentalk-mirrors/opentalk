// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::{
    future::{Future, Ready, ready},
    pin::Pin,
    task::{Context, Poll},
};
use std::rc::Rc;

use actix_http::{HttpMessage, header::Header};
use actix_web::{
    ResponseError,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    web::Data,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use openidconnect::AccessToken;
use opentalk_controller_service::oidc::OidcTokenHandler;
use opentalk_types_api_v1::error::{ApiError, AuthenticationError};
use snafu::Report;
use tracing::Instrument;

/// Middleware factory for [`ServiceAuthMiddleware`]
pub struct ServiceAuth {
    oidc_ctx: Data<dyn OidcTokenHandler>,
}

impl ServiceAuth {
    pub fn new(oidc_ctx: Data<dyn OidcTokenHandler>) -> Self {
        Self { oidc_ctx }
    }
}

impl<S> Transform<S, ServiceRequest> for ServiceAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = ServiceAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ServiceAuthMiddleware {
            service: Rc::new(service),
            oidc_ctx: self.oidc_ctx.clone(),
        }))
    }
}

/// Middleware which extracts and verifies an access-token from the request
///
/// Inserts a `RealmRoles` struct into the request for other services to inspect.
pub struct ServiceAuthMiddleware<S> {
    service: Rc<S>,

    oidc_ctx: Data<dyn OidcTokenHandler>,
}

type ResultFuture<O, E> = Pin<Box<dyn Future<Output = Result<O, E>>>>;

impl<S> Service<ServiceRequest> for ServiceAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = ResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[tracing::instrument(level = "debug", name = "ServiceAuthMiddleware", skip_all)]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let oidc_ctx = self.oidc_ctx.clone();

        let auth = match Authorization::<Bearer>::parse(&req) {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("Unable to parse access token, {}", Report::from_error(e));
                let error = ApiError::unauthorized()
                    .with_message("Unable to parse access token")
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken);

                let response = req.into_response(error.error_response());
                return Box::pin(ready(Ok(response)));
            }
        };

        let access_token = AccessToken::new(auth.into_scheme().token().to_string());

        Box::pin(
            async move {
                match oidc_ctx.check_access_token(&access_token).await {
                    Ok(realm_roles) => {
                        req.extensions_mut().insert(realm_roles);
                        service.call(req).await
                    }
                    Err(err) => Ok(req.into_response(err.error_response())),
                }
            }
            .instrument(tracing::debug_span!("ServiceAuthMiddleware::async::call")),
        )
    }
}
