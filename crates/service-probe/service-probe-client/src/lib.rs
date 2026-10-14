// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Service probe client
//!
//! This crate provides an easy way to access the a HTTP server that can be used for
//! making the status of a service transparent to observers.
#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

pub use service_probe_common::ServiceState;
use snafu::Snafu;
use url::Url;

/// The error that can happen during startup of the service probe.
#[derive(Debug, Snafu)]
pub enum ProbeClientError {
    /// The socket cannot be used for providing the service probe.
    ServiceProbeEndpointUnavailable {
        /// The source error
        source: reqwest::Error,
    },
}

/// fetch service state from given endpoint
pub async fn fetch_service_state(endpoint: &Url) -> Result<ServiceState, ProbeClientError> {
    log::info!("Fetching state from {endpoint}");
    let body = reqwest::get(endpoint.clone())
        .await
        .map_err(|err| ProbeClientError::ServiceProbeEndpointUnavailable { source: err })?
        .text()
        .await;

    match body {
        Ok(content) => {
            if content.lines().any(|line| line == "READY") {
                Ok(ServiceState::Ready)
            } else {
                Ok(ServiceState::Up)
            }
        }

        Err(err) => Err(ProbeClientError::ServiceProbeEndpointUnavailable { source: err }),
    }
}

/// returns true if the service state of the given endpoint is ready
pub async fn is_ready(endpoint: &Url) -> Result<bool, ProbeClientError> {
    match fetch_service_state(endpoint).await {
        Ok(state) => Ok(state == ServiceState::Ready),
        Err(err) => Err(err),
    }
}
