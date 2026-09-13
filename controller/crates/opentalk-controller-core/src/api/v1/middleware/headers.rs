// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::pin::Pin;

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
};
use futures::{
    Future, FutureExt,
    future::{Ready, ready},
};
use tracing_actix_web::RequestId;

#[derive(Clone)]
pub struct Headers;

impl<S, B> Transform<S, ServiceRequest> for Headers
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = HeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HeadersMiddleware { service }))
    }
}

pub struct HeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for HeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse<B>, Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req.extensions().get::<RequestId>().cloned();
        let fut = self.service.call(req);

        async move {
            let mut res = fut.await?;
            if let Some(request_id) = request_id
                && !res.headers().contains_key("x-request-id")
            {
                res.headers_mut().insert(
                    HeaderName::from_static("x-request-id"),
                    HeaderValue::from_str(&request_id.to_string())?,
                );
            }
            Ok(res)
        }
        .boxed_local()
    }
}
