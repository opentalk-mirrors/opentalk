// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling streaming.

mod room_streaming_target;
mod room_streaming_target_resource;
mod streaming_key;
mod streaming_kind;
mod streaming_link;
mod streaming_target;
mod streaming_target_id;
mod streaming_target_kind;
mod streaming_target_kind_resource;
mod streaming_target_resource;

pub use room_streaming_target::{RoomStreamingTarget, get_public_urls_from_room_streaming_targets};
pub use room_streaming_target_resource::RoomStreamingTargetResource;
pub use streaming_key::StreamingKey;
pub use streaming_kind::{StreamingKind, StreamingKindType};
pub use streaming_link::StreamingLink;
pub use streaming_target::StreamingTarget;
pub use streaming_target_id::StreamingTargetId;
pub use streaming_target_kind::StreamingTargetKind;
pub use streaming_target_kind_resource::StreamingTargetKindResource;
pub use streaming_target_resource::StreamingTargetResource;
