// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use actix_http::{StatusCode, body::BoxBody};
use actix_web::{HttpResponse, HttpResponseBuilder, dev::PeerAddr, get, web::Data};
use itertools::Itertools as _;
use metrics_exporter_prometheus::{BuildError, Matcher, PrometheusBuilder, PrometheusHandle};
use opentalk_controller_service::{RedisMetrics, metrics::EndpointMetrics};
use opentalk_controller_settings::{RoomServerKind, SettingsProvider};
use opentalk_database::DatabaseMetrics;
use opentalk_roomserver_room::metrics::{
    CONNECTION_MEETING_TIME, CONNECTION_MEETING_TIME_BUCKETS, ROOM_LIFE_TIME,
    ROOM_LIFE_TIME_BUCKETS,
};
use opentelemetry::{global, otel_error};
use opentelemetry_sdk::{error::OTelSdkError, metrics::SdkMeterProvider};
use prometheus::{Encoder, Registry, TextEncoder};
use snafu::{Backtrace, ResultExt, Snafu};

use crate::Result;

#[derive(Debug, Snafu)]
pub enum MetricsInitError {
    #[snafu(display("Failed to configure the OpenTelemetry Prometheus exporter"))]
    OtelExporter {
        source: OTelSdkError,
        backtrace: Backtrace,
    },
    #[snafu(display("Failed to install the roomserver metrics recorder"))]
    RoomServerRecorder {
        source: BuildError,
        backtrace: Backtrace,
    },
}

pub struct CombinedMetrics {
    registry: Registry,
    pub(super) endpoint: Arc<EndpointMetrics>,
    pub(super) database: Arc<DatabaseMetrics>,
    pub(super) redis: Arc<RedisMetrics>,
    pub(super) roomserver: Option<PrometheusHandle>,
}

impl CombinedMetrics {
    pub fn try_init(roomserver_kind: &RoomServerKind) -> Result<Self, MetricsInitError> {
        let registry = prometheus::Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()
            .context(OtelExporterSnafu)?;

        let provider_builder = SdkMeterProvider::builder().with_reader(exporter);
        let provider_builder = EndpointMetrics::append_views(provider_builder);
        let provider_builder = DatabaseMetrics::append_views(provider_builder);
        let provider_builder = RedisMetrics::append_views(provider_builder);

        global::set_meter_provider(provider_builder.build());
        let meter = global::meter("ot-controller");

        let roomserver = match roomserver_kind {
            RoomServerKind::Internal { .. } => Some(
                PrometheusBuilder::new()
                    .set_buckets_for_metric(
                        Matcher::Full(CONNECTION_MEETING_TIME.to_string()),
                        CONNECTION_MEETING_TIME_BUCKETS,
                    )
                    .context(RoomServerRecorderSnafu)?
                    .set_buckets_for_metric(
                        Matcher::Full(ROOM_LIFE_TIME.to_string()),
                        ROOM_LIFE_TIME_BUCKETS,
                    )
                    .context(RoomServerRecorderSnafu)?
                    .install_recorder()
                    .context(RoomServerRecorderSnafu)?,
            ),
            RoomServerKind::External { .. } => None,
        };

        let endpoint = Arc::new(EndpointMetrics::new(&meter));
        let database = Arc::new(DatabaseMetrics::new(&meter));
        let redis = Arc::new(RedisMetrics::new(&meter));

        Ok(Self {
            registry,
            endpoint,
            database,
            redis,
            roomserver,
        })
    }
}

#[get("/metrics")]
pub async fn metrics(
    settings: Data<SettingsProvider>,
    PeerAddr(peer_addr): PeerAddr,
    metrics: Data<CombinedMetrics>,
) -> HttpResponse {
    let settings = settings.get();

    let allowlist = &settings.metrics.allowlist;
    let allowed = allowlist
        .iter()
        .any(|allowed_net| allowed_net.contains(&peer_addr.ip()));

    if !allowed {
        if allowlist.is_empty() {
            tracing::debug!(
                "An attempt to access the metrics endpoint from IP address {peer_addr} was denied. Access to the metrics endpoint has not been configured."
            );
        } else {
            let allowed_nets = allowlist.iter().map(|net| format!("\"{net}\"")).join(", ");
            tracing::debug!(
                "An attempt to access the metrics endpoint from IP address {peer_addr} was denied. Access allowed from: {allowed_nets}."
            );
        }
        return HttpResponse::new(StatusCode::FORBIDDEN);
    }

    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    let mut buf = Vec::new();
    if let Err(err) = encoder.encode(&metric_families[..], &mut buf) {
        otel_error!(name: "export_failure", error = err.to_string());
        return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut response = String::from_utf8(buf).unwrap_or_default();
    if let Some(roomserver) = &metrics.roomserver {
        response.push('\n');
        response.push_str(&roomserver.render());
    }

    HttpResponseBuilder::new(StatusCode::OK)
        .content_type("text/plain")
        .body(BoxBody::new(response))
}
