// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    Error, HttpMessage,
    dev::{ServiceRequest, ServiceResponse},
    http::header::USER_AGENT,
};
use opentalk_controller_settings::Logging;
use opentalk_tracing::{Bootstrap, SdkTracerProvider, TracingParams};
use opentelemetry::propagation::Extractor;
use snafu::ResultExt as _;
use tracing::Span;
use tracing_actix_web::{RequestId, RootSpanBuilder};

use crate::Result;

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "CONTROLLER_LOG";

// If these default values are adjusted, that change should be synchronized
// into `example/controller.toml` for transparency towards administrators
// and documentation purposes.
const DEFAULT_LOGGING_DIRECTIVES: &str = "error,\
    opentalk=info,\
    pinky_swear=off,\
    rustls=warn,\
    mio=error,\
    lapin=warn,\
    execution_id=trace";

/// Installs a temporary subscriber used until [`init`] can run, capturing log
/// events emitted while the settings are still being loaded. Pass the returned
/// guard to [`init`].
pub fn bootstrap() -> Bootstrap {
    opentalk_tracing::bootstrap(&[LOG_ENV], DEFAULT_LOGGING_DIRECTIVES)
}

// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `CONTROLLER_LOG` > `RUST_LOG` > `[tracing] default_directives` >
/// [`DEFAULT_LOGGING_DIRECTIVES`]. When an OTLP endpoint is configured, spans
/// are additionally exported to it.
pub fn init(bootstrap: Bootstrap, settings: &Logging) -> Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        bootstrap,
        Some(settings),
        TracingParams {
            service_name: "controller",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[LOG_ENV],
        },
    )
    .with_whatever_context(|e| format!("Failed to initialize tracing: {e:#}"))
}

pub struct ReducedSpanBuilder;

impl RootSpanBuilder for ReducedSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let span = create_span(request);
        opentalk_tracing::propagation::set_parent(&span, &ActixHeaderExtractor(request.headers()));
        span
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        match &outcome {
            Ok(response) => {
                if let Some(error) = response.response().error() {
                    handle_error(span, error)
                } else {
                    span.record("http.status_code", response.response().status().as_u16());
                    span.record("otel.status_code", "OK");
                }
            }
            Err(error) => handle_error(span, error),
        };
    }
}

struct ActixHeaderExtractor<'a>(&'a actix_web::http::header::HeaderMap);

impl Extractor for ActixHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.to_str().ok()
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}

fn handle_error(span: Span, error: &Error) {
    let response_error = error.as_response_error();
    span.record("exception.message", tracing::field::display(response_error));
    span.record("exception.details", tracing::field::debug(response_error));
    let status_code = response_error.status_code();
    span.record("http.status_code", status_code.as_u16());

    if status_code.is_client_error() {
        span.record("otel.status_code", "OK");
    } else {
        span.record("otel.status_code", "ERROR");
    }
}

fn create_span(request: &ServiceRequest) -> Span {
    let user_agent = request
        .headers()
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let http_route: std::borrow::Cow<'static, str> = request
        .match_pattern()
        .map(Into::into)
        .unwrap_or_else(|| "default".into());

    let connection_info = request.connection_info();
    let request_id = request.extensions().get::<RequestId>().cloned().unwrap();
    let span = tracing::info_span!(
        "HTTP request",
        http.method = %request.method().as_str(),
        http.route = %http_route,
        http.flavor = ?request.version(),
        http.scheme = %connection_info.scheme(),
        http.host = %connection_info.host(),
        http.user_agent = %user_agent,
        http.target = %request.uri(),
        http.status_code = tracing::field::Empty,
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
        request_id = %request_id,
        trace_id = tracing::field::Empty,
        exception.message = tracing::field::Empty,
        // Not proper OpenTelemetry, but their terminology is fairly exception-centric
        exception.details = tracing::field::Empty,
    );

    span
}
