// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Logging {
    pub default_directives: Option<Vec<String>>,

    pub otlp_tracing_endpoint: Option<String>,

    pub service_name: Option<String>,

    pub service_namespace: Option<String>,

    pub service_instance_id: Option<String>,
}
