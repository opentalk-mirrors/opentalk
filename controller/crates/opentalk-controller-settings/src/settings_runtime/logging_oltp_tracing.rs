// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Logging OLTP tracing configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggingOltpTracing {
    /// The OLTP endpoint.
    pub endpoint: String,

    /// The name of this service.
    pub service_name: String,

    /// The namespace of this service.
    pub service_namespace: String,

    /// The instance id of this service.
    pub service_instance_id: String,
}
