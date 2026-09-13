// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

use actix_web::{
    HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse},
};
use tracing::{Instrument, Span, field};

use crate::authorization::{Admission, AuthorizationTarget, Authorizer};

/// OpenTalk API endpoints authorization middleware.
#[derive(Debug)]
pub struct AuthorizationService<S> {
    service: Rc<RefCell<S>>,
    authorizer: Authorizer,
}

impl<S> AuthorizationService<S> {
    pub(super) fn new(service: Rc<RefCell<S>>, authorizer: Authorizer) -> Self {
        Self {
            service,
            authorizer,
        }
    }
}

type ResultFuture<O, E> = Pin<Box<dyn Future<Output = Result<O, E>>>>;

impl<S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static>
    Service<ServiceRequest> for AuthorizationService<S>
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = ResultFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[tracing::instrument(name = "AuthorizationMiddleware", level = "debug", skip_all, fields(authorized = field::Empty, target = field::Empty))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let span = Span::current();
        let authorizer = self.authorizer.clone();
        let service = self.service.clone();
        Box::pin(
            async move {
                let span = Span::current();
                let target = AuthorizationTarget::try_from(&req);
                span.record("target", field::debug(target.as_ref().ok()));
                let admission = if let Ok(target) = target {
                    authorizer.authorize(target).await
                } else {
                    tracing::error!("Could not parse path for {:?}, denying access", req.path());
                    Ok(Admission::Denied)
                };
                span.record("authorized", field::debug(admission.as_ref().ok().copied()));

                match admission {
                    Ok(Admission::Allowed) => service.call(req).await,
                    Ok(Admission::Denied) => {
                        Ok(req.into_response(HttpResponse::Forbidden().finish()))
                    }
                    Ok(Admission::AuthenticationRequired) => {
                        Ok(req.into_response(HttpResponse::Unauthorized().finish()))
                    }
                    Err(e) => {
                        tracing::error!("Attempt to request authorization failed: {e:?}");
                        Ok(req.into_response(HttpResponse::InternalServerError().finish()))
                    }
                }
            }
            .instrument(span),
        )
    }
}
