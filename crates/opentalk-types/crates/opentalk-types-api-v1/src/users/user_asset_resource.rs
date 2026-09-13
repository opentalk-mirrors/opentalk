// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_common::module_assets::AssetResource;
use opentalk_types_common::{events::EventId, rooms::RoomId, utils::ExampleData};

/// Information related to a specific asset
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UserAssetResource::example_data()))
)]
pub struct UserAssetResource {
    /// The asset resource
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub asset: AssetResource,

    /// The id of the room to which the asset belongs
    pub room_id: RoomId,

    /// The id of the event that is associated with the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub event_id: Option<EventId>,
}

impl UserAssetResource {
    /// Create a UserAssetResource from an asset, room id and event id
    pub fn new(asset: AssetResource, room_id: RoomId, event_id: Option<EventId>) -> Self {
        Self {
            asset,
            room_id,
            event_id,
        }
    }
}

impl ExampleData for UserAssetResource {
    fn example_data() -> Self {
        Self {
            asset: AssetResource::example_data(),
            room_id: RoomId::example_data(),
            event_id: Some(EventId::example_data()),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn user_asset_resource() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use opentalk_types_common::assets::AssetId;

        let asset_resource = UserAssetResource {
            asset: AssetResource {
                id: AssetId::from_u128(0xd339dde5_1161_4ad1_a3d8_3e27b5d84377),
                created_at: "2023-09-05T08:57:42Z".parse()?,
                namespace: Some("legal_vote".parse()?),
                kind: "protocol_pdf".to_string(),
                filename: "vote_protocol_2023-09something.pdf".to_string(),
                size: 230423.into(),
            },
            room_id: RoomId::from_u128(0xe693fdc6_2b4d_4623_a423_a191675908d7),
            event_id: Some(EventId::from_u128(0x660bc9f5_58a4_46a4_9621_23743c70e3b4)),
        };

        let expected_json = json!({
          "id": "d339dde5-1161-4ad1-a3d8-3e27b5d84377",
          "filename": "vote_protocol_2023-09something.pdf",
          "created_at": "2023-09-05T08:57:42Z",
          "size": 230423,
          "room_id": "e693fdc6-2b4d-4623-a423-a191675908d7",
          "event_id": "660bc9f5-58a4-46a4-9621-23743c70e3b4",
          "namespace": "legal_vote",
          "kind": "protocol_pdf"
        });

        let serialized = serde_json::to_value(asset_resource.clone())?;
        assert_eq!(expected_json, serialized);

        let deserialized = serde_json::from_value(expected_json)?;
        assert_eq!(asset_resource, deserialized);

        Ok(())
    }
}
