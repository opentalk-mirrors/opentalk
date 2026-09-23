// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry::{
    global,
    propagation::{Extractor, Injector},
    trace::TraceContextExt as _,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub(crate) fn init() {
    global::set_text_map_propagator(TraceContextPropagator::new());
}

pub fn set_parent(span: &Span, extractor: &impl Extractor) {
    let parent = global::get_text_map_propagator(|propagator| propagator.extract(extractor));

    if parent.span().span_context().is_valid() {
        let _ = span.set_parent(parent);
    }
}

pub fn set_parent_from_headers(span: &Span, headers: &HeaderMap) {
    set_parent(span, &HeaderMapExtractor(headers));
}

pub fn inject_current_context(headers: &mut HeaderMap) {
    let context = Span::current().context();
    if context.span().span_context().is_valid() {
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&context, &mut HeaderMapInjector(headers));
        });
    }
}

pub fn current_context_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    inject_current_context(&mut headers);
    headers
}

struct HeaderMapExtractor<'a>(&'a HeaderMap);

impl Extractor for HeaderMapExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.to_str().ok()
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(HeaderName::as_str).collect()
    }
}

struct HeaderMapInjector<'a>(&'a mut HeaderMap);

impl Injector for HeaderMapInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        let Ok(name) = HeaderName::from_bytes(key.as_bytes()) else {
            return;
        };
        let Ok(value) = HeaderValue::from_str(&value) else {
            return;
        };
        self.0.insert(name, value);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use opentelemetry::{
        Context,
        trace::{SpanContext, SpanId, TraceFlags, TraceId, TraceState, TracerProvider as _},
    };
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

    use super::*;

    static INIT: Once = Once::new();

    fn init_propagator() {
        INIT.call_once(init);
    }

    fn with_tracing(test: impl FnOnce()) {
        let provider = SdkTracerProvider::builder().build();
        let tracer = provider.tracer("trace-propagation-test");
        let subscriber =
            tracing_subscriber::registry().with(tracing_opentelemetry::layer().with_tracer(tracer));
        let _guard = subscriber.set_default();
        test();
    }

    #[test]
    fn extracts_remote_parent() {
        init_propagator();
        with_tracing(|| {
            let mut headers = HeaderMap::new();
            headers.insert(
                "traceparent",
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
                    .parse()
                    .unwrap(),
            );
            headers.insert("tracestate", "vendor=value".parse().unwrap());
            let span = tracing::info_span!("request");

            set_parent_from_headers(&span, &headers);

            let otel_context = span.context();
            let otel_span = otel_context.span();
            let span_context = otel_span.span_context();
            assert_eq!(
                span_context.trace_id(),
                TraceId::from_hex("0af7651916cd43dd8448eb211c80319c").unwrap()
            );
            // The extracted parent is remote, but this context belongs to the new local span.
            assert!(!span_context.is_remote());
            assert_eq!(span_context.trace_state().header(), "vendor=value");
        });
    }

    #[test]
    fn injects_current_context() {
        init_propagator();
        with_tracing(|| {
            let span = tracing::info_span!("request");
            let context = Context::new().with_remote_span_context(SpanContext::new(
                TraceId::from_hex("0af7651916cd43dd8448eb211c80319c").unwrap(),
                SpanId::from_hex("b7ad6b7169203331").unwrap(),
                TraceFlags::SAMPLED,
                true,
                TraceState::from_key_value([("vendor", "value")]).unwrap(),
            ));
            span.set_parent(context).unwrap();
            let _guard = span.enter();
            let mut headers = HeaderMap::new();

            inject_current_context(&mut headers);

            let traceparent = headers.get("traceparent").unwrap().to_str().unwrap();
            assert!(traceparent.starts_with("00-0af7651916cd43dd8448eb211c80319c-"));
            assert!(traceparent.ends_with("-01"));
            assert_eq!(headers.get("tracestate").unwrap(), "vendor=value");
        });
    }

    #[test]
    fn ignores_malformed_parent() {
        init_propagator();
        with_tracing(|| {
            let mut headers = HeaderMap::new();
            headers.insert("traceparent", "invalid".parse().unwrap());
            let span = tracing::info_span!("request");

            set_parent_from_headers(&span, &headers);

            assert!(span.context().span().span_context().is_valid());
        });
    }
}
