// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context as _;
use axum::{
    Router,
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::get,
    serve::Listener as _,
};
use axum_prometheus::{
    AXUM_HTTP_REQUESTS_DURATION_SECONDS, GenericMetricLayer, PrometheusMetricLayerBuilder,
    metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle},
    utils::SECONDS_DURATION_BUCKETS,
};
use cidr::IpInet;
use opentalk_roomserver_room::metrics::{
    CONNECTION_MEETING_TIME, CONNECTION_MEETING_TIME_BUCKETS, ROOM_LIFE_TIME,
    ROOM_LIFE_TIME_BUCKETS,
};
use tokio::sync::watch;

use crate::{
    ApplicationState,
    tcp_multi_listener::{MultiAddr, MultiListener},
    wait_shutdown,
};

pub(super) fn build_prometheus_layer<'a>() -> (
    GenericMetricLayer<'a, PrometheusHandle, axum_prometheus::Handle>,
    PrometheusHandle,
) {
    PrometheusMetricLayerBuilder::new()
        .with_prefix("api")
        .enable_response_body_size(true)
        .with_metrics_from_fn(|| {
            PrometheusBuilder::new()
                .set_buckets_for_metric(
                    Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
                    SECONDS_DURATION_BUCKETS,
                )
                .expect("Setting prometheus buckets failed")
                .set_buckets_for_metric(
                    Matcher::Full(CONNECTION_MEETING_TIME.to_string()),
                    CONNECTION_MEETING_TIME_BUCKETS,
                )
                .expect("Setting prometheus meeting time buckets failed")
                .set_buckets_for_metric(
                    Matcher::Full(ROOM_LIFE_TIME.to_string()),
                    ROOM_LIFE_TIME_BUCKETS,
                )
                .expect("Setting prometheus room life time buckets failed")
                .install_recorder()
                .expect("Installing prometheus recorder failed")
        })
        .build_pair()
}

pub(crate) async fn run_metric_server<H>(
    listener: MultiListener,
    allowlist: Vec<IpInet>,
    metric_handle: H,
    app_state: watch::Receiver<ApplicationState>,
) -> Result<(), anyhow::Error>
where
    H: MetricHandle + Clone + Send + Sync + 'static,
{
    let ctx = MetricContext {
        metric_handle,
        allowlist,
    };

    let router = Router::<MetricContext<H>>::new()
        .route("/metrics", get(metrics))
        .with_state(ctx);
    tracing::info!(
        "Listening for metrics on {}",
        listener.local_addr().expect("Failed to get local address")
    );

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<MultiAddr>(),
    )
    .with_graceful_shutdown(wait_shutdown(app_state))
    .await
    .context("Failed to serve metrics")
}

pub(crate) trait MetricHandle {
    fn render(&self) -> String;
}

#[derive(Clone)]
struct MetricContext<H: MetricHandle> {
    pub metric_handle: H,
    pub allowlist: Vec<IpInet>,
}

impl<H: MetricHandle> MetricContext<H> {
    fn render(&mut self) -> String {
        self.metric_handle.render()
    }
}

/// Returns the metrics
///
/// Returns the metrics in a text format.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = StatusCode::OK, description = "Metrics were successfully retrieved"),
        (status = StatusCode::FORBIDDEN, description = "The client IP is not allowed to access the metrics endpoint"),
    ),
    security(),
    )]
async fn metrics<H: MetricHandle>(
    mut context: State<MetricContext<H>>,
    ConnectInfo(addr): ConnectInfo<MultiAddr>,
) -> Result<String, StatusCode> {
    let addr = match addr.addrs.as_slice() {
        [a] => a,
        _ => {
            tracing::error!(
                "Expected a single address in ConnectInfo, got multiple or none: {:?}",
                addr.addrs
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ip = addr.ip();
    if context.allowlist.iter().any(|net| net.contains(&ip)) {
        return Ok(context.render());
    }

    if context.allowlist.is_empty() {
        tracing::debug!(
            "An attempt to access the metrics endpoint from IP address {ip} was denied. Access to the metrics endpoint has not been configured."
        );
    } else {
        let allowed_nets = context
            .allowlist
            .iter()
            .map(|net| format!("\"{net}\""))
            .collect::<Vec<String>>()
            .join(", ");
        tracing::debug!(
            "An attempt to access the metrics endpoint from IP address {ip} was denied. Access allowed from: {allowed_nets}."
        );
    }

    Err(StatusCode::FORBIDDEN)
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::Duration,
    };

    use axum::serve::Listener as _;
    use cidr::IpInet;
    use opentalk_roomserver_common::application_state::ApplicationState;
    use reqwest::StatusCode;
    use tokio::{net::TcpStream, sync::watch};

    use super::run_metric_server;
    use crate::{metrics::MetricHandle, tcp_multi_listener::MultiListener};

    const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    const RESPONSE: &str = "metrics";

    #[derive(Clone, Copy)]
    struct MockMetricHandle;

    impl MetricHandle for MockMetricHandle {
        fn render(&self) -> String {
            RESPONSE.to_string()
        }
    }

    async fn wait_for_server(addr: SocketAddr) {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if TcpStream::connect(addr).await.is_ok() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("metrics server did not become ready in time");
    }

    /// Binds a metrics server to a random free port (port `0`, allocated by the OS)
    /// and starts it. Returns the address the server is listening on together with
    /// the [`watch::Sender`] that must be kept alive for the server to keep running.
    async fn start_server(allowlist: Vec<IpInet>) -> (SocketAddr, watch::Sender<ApplicationState>) {
        let (app_state, _) = watch::channel(ApplicationState::Running);

        let listener = MultiListener::bind([SocketAddr::new(LOCALHOST, 0)])
            .await
            .expect("failed to bind metrics listener");
        let addr = listener
            .local_addr()
            .expect("failed to get local address")
            .addrs[0];

        tokio::spawn(run_metric_server(
            listener,
            allowlist,
            MockMetricHandle,
            app_state.subscribe(),
        ));

        wait_for_server(addr).await;

        (addr, app_state)
    }

    #[test_log::test(tokio::test)]
    async fn allowlist_not_configured() {
        // Start the metric server without any allowed IPs
        let (addr, _app_state) = start_server(vec![]).await;

        let status = reqwest::get(format!("http://{addr}/metrics"))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test_log::test(tokio::test)]
    async fn localhost_forbidden() {
        // Start the metric server allowing only a different IP than localhost
        let (addr, _app_state) = start_server(vec![Ipv4Addr::new(192, 168, 0, 1).into()]).await;

        let status = reqwest::get(format!("http://{addr}/metrics"))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[test_log::test(tokio::test)]
    async fn localhost_allowed() {
        // Start the metric server allowing localhost
        let (addr, _app_state) = start_server(vec![LOCALHOST.into()]).await;

        let response = reqwest::get(format!("http://{addr}/metrics"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await.unwrap(), RESPONSE);
    }
}
