// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use super::{EventExceptionResource, EventResource};

/// Return type of the `GET /events` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
#[allow(clippy::large_enum_variant)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventOrException::example_data()))
)]
pub enum EventOrException {
    /// Event resource
    Event(EventResource),
    /// Event exception resource
    Exception(EventExceptionResource),
}

impl ExampleData for EventOrException {
    fn example_data() -> Self {
        Self::Event(EventResource::example_data())
    }
}
