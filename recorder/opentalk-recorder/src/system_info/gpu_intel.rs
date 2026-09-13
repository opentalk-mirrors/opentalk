// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::atomic::Ordering,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::IS_FEASIBLE;
use crate::system_info::CURRENT_LOAD;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Metrics {
    pub(crate) period: Period,
    pub(crate) frequency: Frequency,
    pub(crate) interrupts: Interrupts,
    pub(crate) rc6: Rc6,
    pub(crate) engines: HashMap<String, Usage>,
}

impl Metrics {
    pub(crate) fn maximum_busy(&self) -> u8 {
        self.engines
            .values()
            .map(|usage| usage.busy as u8)
            .max()
            .unwrap_or_default()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Period {
    pub(crate) duration: f64,
    pub(crate) unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Frequency {
    pub(crate) requested: f64,
    pub(crate) actual: f64,
    pub(crate) unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Interrupts {
    pub(crate) count: f64,
    pub(crate) unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Rc6 {
    pub(crate) value: f64,
    pub(crate) unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Usage {
    pub(crate) busy: f64,
    pub(crate) sema: f64,
    pub(crate) wait: f64,
    pub(crate) unit: String,
}

pub(crate) fn gpu_intel_usage_poll(max_gpu_usage: u8, device: Option<String>) -> Result<()> {
    let mut command = &mut Command::new("intel_gpu_top");
    command = command.arg("-J").arg("-s").arg("500");
    if let Some(device) = device {
        command = command.arg("-d").arg(format!("drm:{device}"));
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .with_context(|| "STDOUT must be available since it wasn't taken before")?;
    let reader = BufReader::new(stdout);

    let mut data = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| "read new line for intel_gpu_top failed")?;
        if line.starts_with('[') {
            continue;
        }
        // The data sends '},' with a trailing comma, this "hacky way" resolves
        // the trailing comma at the end
        if !line.starts_with('}') {
            data.push_str(&line);
            continue;
        }
        // Add the missing '}' without a trailing comma
        data += "}";

        let metrics: Metrics = serde_json::from_str(&std::mem::take(&mut data))
            .with_context(|| "unable to parse Metrics from intel_gpu_top")?;

        let current_load = metrics.maximum_busy();

        IS_FEASIBLE.store(current_load <= max_gpu_usage, Ordering::Relaxed);
        CURRENT_LOAD.store(current_load, Ordering::Relaxed);
    }

    Ok(())
}
