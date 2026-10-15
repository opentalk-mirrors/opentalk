// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Runs in its own test binary so the global propagator starts uninstalled,
//! letting us assert that incoming trace headers are not parsed when the OTLP
//! endpoint is disabled.

use std::cell::Cell;

use opentalk_tracing::{TracingParams, TracingSettings, bootstrap, init};
use opentelemetry::propagation::Extractor;

/// Records whether the propagator queried the carrier for trace headers.
struct SpyExtractor {
    accessed: Cell<bool>,
}

impl Extractor for SpyExtractor {
    fn get(&self, _key: &str) -> Option<&str> {
        self.accessed.set(true);
        None
    }

    fn keys(&self) -> Vec<&str> {
        self.accessed.set(true);
        Vec::new()
    }
}

#[test]
fn incoming_trace_headers_are_not_parsed_without_otlp() {
    let bootstrap = bootstrap(&[], "info");
    let settings = TracingSettings::default();
    let _provider = init(
        bootstrap,
        Some(&settings),
        TracingParams {
            service_name: "opentalk-tracing-test",
            service_version: "0.0.0",
            default_directives: "info",
            filter_env_vars: &[],
        },
    )
    .expect("tracing init without an OTLP endpoint should succeed");

    let span = tracing::info_span!("request");
    let extractor = SpyExtractor {
        accessed: Cell::new(false),
    };

    opentalk_tracing::propagation::set_parent(&span, &extractor);

    assert!(
        !extractor.accessed.get(),
        "traceparent/tracestate must not be parsed when OTLP export is disabled"
    );
}
