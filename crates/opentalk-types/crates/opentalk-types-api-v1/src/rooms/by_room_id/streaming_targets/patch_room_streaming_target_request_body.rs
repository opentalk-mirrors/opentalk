// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use crate::rooms::streaming_targets::UpdateStreamingTargetKind;

/// The body of a *PATCH /rooms/{room_id}/streaming_targets/{streaming_target_id}* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PatchRoomStreamingTargetRequestBody::example_data()))
)]
pub struct PatchRoomStreamingTargetRequestBody {
    /// The name of the streaming target
    pub name: Option<String>,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: Option<UpdateStreamingTargetKind>,
}

impl ExampleData for PatchRoomStreamingTargetRequestBody {
    fn example_data() -> Self {
        Self {
            name: Some("My OwnCast Stream".to_string()),
            kind: Some(UpdateStreamingTargetKind::example_data()),
        }
    }
}
