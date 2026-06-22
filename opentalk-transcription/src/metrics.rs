// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};

pub(crate) struct PrometheusState {
    pub(crate) registry: prometheus::Registry,
}

impl PrometheusState {
    pub(crate) fn new() -> Self {
        Self {
            registry: prometheus::Registry::new(),
        }
    }

    pub(crate) fn metrics(&self) -> Result<String> {
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = String::new();
        let metric_families = self.registry.gather();
        encoder
            .encode_utf8(&metric_families, &mut buffer)
            .context("Failed to encode prometheus metrics")?;

        Ok(buffer)
    }
}
