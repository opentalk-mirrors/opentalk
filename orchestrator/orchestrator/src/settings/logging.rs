// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

/// The logging configuration for the orchestrator
#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct Logging {
    /// The logging directives to use for the orchestrator. Has the same format as the `RUST_LOG`
    /// environment variable.
    pub(crate) directives: Option<Vec<String>>,
}

impl Logging {
    pub fn directives(&self) -> Option<String> {
        self.directives.as_ref().map(|filter| filter.join(","))
    }
}
