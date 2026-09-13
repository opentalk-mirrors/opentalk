// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use super::EventInstance;

/// Response for *GET /events/{event_id}/instances*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetEventInstancesResponseBody::example_data()))
)]
pub struct GetEventInstancesResponseBody(pub Vec<EventInstance>);

impl ExampleData for GetEventInstancesResponseBody {
    fn example_data() -> Self {
        Self(vec![EventInstance::example_data()])
    }
}
