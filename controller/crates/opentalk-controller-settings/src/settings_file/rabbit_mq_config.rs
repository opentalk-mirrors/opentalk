// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct RabbitMqConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_connections: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_channels_per_connection: Option<u32>,

    /// Mail sending is disabled when this is None
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mail_task_queue: Option<String>,

    /// Recording is disabled if this isn't set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_task_queue: Option<String>,

    /// Time to live for messages sent through RabbitMQ
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ttl_seconds: Option<u64>,
}
