// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    cell::RefCell,
    future::{Ready, ready},
    rc::Rc,
};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};

use super::AuthorizationService;
use crate::authorization::Authorizer;

/// Transform for authorizing requests to OpenTalk API endpoints.
#[derive(Debug, Clone)]
pub struct AuthorizationTransform {
    authorizer: Authorizer,
}

impl AuthorizationTransform {
    /// Create a new authorization transform instance
    pub fn new(authorizer: Authorizer) -> Self {
        Self { authorizer }
    }
}

impl<S> Transform<S, ServiceRequest> for AuthorizationTransform
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Transform = AuthorizationService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizationService::new(
            Rc::new(RefCell::new(service)),
            self.authorizer.clone(),
        )))
    }
}
