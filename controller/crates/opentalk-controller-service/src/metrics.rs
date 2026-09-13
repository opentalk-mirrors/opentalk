// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides some metrics functions and the like.

use opentalk_mail_worker_protocol::MailTask;
use opentelemetry::{
    Key, KeyValue,
    metrics::{Counter, Histogram, Meter},
};
use opentelemetry_sdk::metrics::{Aggregation, Instrument, MeterProviderBuilder, Stream};

const MAIL_TASK_KIND: Key = Key::from_static_str("mail_task_kind");
const REQ_DURATION_SECS: &str = "web.request_duration_seconds";
const RESP_SIZE_BYTES: &str = "web.response_sizes_bytes";
const ISSUED_EMAIL_TASK_COUNT: &str = "web.issued_email_tasks_count";

/// Metrics belonging to endpoints
#[derive(Debug)]
pub struct EndpointMetrics {
    /// A histogram for the request durations
    pub request_durations: Histogram<f64>,
    /// A histogram for the response sizes
    pub response_sizes: Histogram<u64>,
    /// A counter for the issued email tasks
    pub issued_email_tasks_count: Counter<u64>,
}

impl EndpointMetrics {
    /// Appends views to the meter provider builder
    pub fn append_views(provider_builder: MeterProviderBuilder) -> MeterProviderBuilder {
        provider_builder
            .with_view(|instrument: &Instrument| {
                if instrument.name() == REQ_DURATION_SECS {
                    Stream::builder()
                        .with_aggregation(Aggregation::ExplicitBucketHistogram {
                            boundaries: vec![0.005, 0.01, 0.25, 0.5, 1.0, 2.0],
                            record_min_max: false,
                        })
                        .build()
                        .ok()
                } else {
                    None
                }
            })
            .with_view(|instrument: &Instrument| {
                if instrument.name() == RESP_SIZE_BYTES {
                    Stream::builder()
                        .with_aggregation(Aggregation::ExplicitBucketHistogram {
                            boundaries: vec![100.0, 1_000.0, 10_000.0, 100_000.0],
                            record_min_max: false,
                        })
                        .build()
                        .ok()
                } else {
                    None
                }
            })
    }

    /// Creates new [`EndpointMetrics`]
    pub fn new(meter: &Meter) -> Self {
        Self {
            request_durations: meter
                .f64_histogram(REQ_DURATION_SECS)
                .with_description("HTTP response time measured in actix-web middleware")
                .with_unit("seconds")
                .with_boundaries(vec![0.005, 0.01, 0.25, 0.5, 1.0, 2.0])
                .build(),
            response_sizes: meter
                .u64_histogram(RESP_SIZE_BYTES)
                .with_description(
                    "HTTP response size for sized responses measured in actix-web middleware",
                )
                .with_unit("bytes")
                .build(),
            issued_email_tasks_count: meter
                .u64_counter(ISSUED_EMAIL_TASK_COUNT)
                .with_description("Number of issued email tasks")
                .build(),
        }
    }

    /// Increment the number of issued email tasks
    pub fn increment_issued_email_tasks_count(&self, mail_task: &MailTask) {
        self.issued_email_tasks_count
            .add(1, &[KeyValue::new(MAIL_TASK_KIND, mail_task.as_kind_str())]);
    }
}
