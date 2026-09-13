// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{pin::Pin, sync::Arc, time::Instant};

use actix_http::body::{BodySize, MessageBody};
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures::{
    Future, FutureExt,
    future::{Ready, ready},
};
use opentalk_controller_service::metrics::EndpointMetrics;
use opentelemetry::{Key, KeyValue};

#[derive(Clone)]
pub struct RequestMetrics {
    metrics: Arc<EndpointMetrics>,
}

impl RequestMetrics {
    pub fn new(metrics: Arc<EndpointMetrics>) -> Self {
        Self { metrics }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestMetricsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestMetricsMiddleware {
            service,
            metrics: self.metrics.clone(),
        }))
    }
}

const HANDLER_KEY: Key = Key::from_static_str("handler");
const METHOD_KEY: Key = Key::from_static_str("method");
const STATUS_KEY: Key = Key::from_static_str("status");

pub struct RequestMetricsMiddleware<S> {
    service: S,
    metrics: Arc<EndpointMetrics>,
}

impl<S, B> Service<ServiceRequest> for RequestMetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse<B>, Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let metrics = self.metrics.clone();

        let handler = KeyValue::new(
            HANDLER_KEY,
            req.match_pattern().unwrap_or_else(|| "default".to_string()),
        );
        let method = KeyValue::new(METHOD_KEY, req.method().to_string());

        let service = self.service.call(req);

        async move {
            let start = Instant::now();

            let result = service.await;

            let duration = start.elapsed();

            if let Ok(resp) = result {
                let status = KeyValue::new(STATUS_KEY, resp.status().as_u16() as i64);
                let labels = [handler, method, status];

                metrics
                    .request_durations
                    .record(duration.as_secs_f64(), &labels);

                if let BodySize::Sized(size) = resp.response().body().size() {
                    metrics.response_sizes.record(size, &labels);
                }

                Ok(resp)
            } else {
                result
            }
        }
        .boxed_local()
    }
}
