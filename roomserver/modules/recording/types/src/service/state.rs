// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};
use url::Url;

/// Recording service specific state provided in the recording module to recording services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingServiceState {
    // The streams to be sent initially to the recorder
    pub streaming_targets: BTreeMap<StreamingTargetId, ServiceStreamingTarget>,
}

/// Information about a streaming target sent to the streaming service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStreamingTarget {
    /// The target Url to which the stream shall be streamed to
    pub location: Url,
}
