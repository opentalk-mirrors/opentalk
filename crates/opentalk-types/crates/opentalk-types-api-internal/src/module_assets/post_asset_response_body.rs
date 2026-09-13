// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::module_assets::{AssetResource, Quota};

/// Response for the `POST /services/rooms/{room_id}/asset` endpoint
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostAssetResponseBody {
    /// The asset that was uploaded
    pub asset_resource: AssetResource,

    /// The total and used quota in bytes
    pub quota: Quota,
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use opentalk_types_common::{modules::module_id, utils::ExampleData as _};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::PostAssetResponseBody;
    use crate::module_assets::{AssetResource, Quota};

    #[test]
    fn serialize_post_asset_response_body() {
        let body = PostAssetResponseBody {
            asset_resource: AssetResource::example_data(),
            quota: Quota::example_data(),
        };
        let produced =
            serde_json::to_value(body).expect("PostAssetResponseBody must be serializable");

        assert_eq!(
            produced,
            json!(
            {
              "asset_resource": {
                "id": "00000000-0000-0000-0000-0000aabbcc00",
                "filename": "recording.webm",
                "namespace": "recording",
                "created_at": "2024-06-18T11:22:33Z",
                "kind": "record",
                "size": 98765432
              },
              "quota": {
                "total": 5368709120u64,
                "used": 2147483648u64,
              }
            }
            )
        );
    }

    #[test]
    fn deserialize_post_asset_response_body() {
        let json = json!({
            "asset_resource": {
              "id": "00000000-0000-0000-0000-0000aabbcc00",
              "filename": "recording.webm",
              "namespace": "recording",
              "created_at": "1970-01-01T00:00:00Z",
              "kind": "record",
              "size": 98765432
            },
            "quota": {
              "total": 2,
              "used": 1,
            }
        });
        let produced: PostAssetResponseBody = serde_json::from_value(json).unwrap();
        let expected = PostAssetResponseBody {
            asset_resource: AssetResource {
                id: "00000000-0000-0000-0000-0000aabbcc00".parse().unwrap(),
                filename: "recording.webm".to_string(),
                namespace: Some(module_id!("recording")),
                created_at: DateTime::UNIX_EPOCH,
                kind: "record".to_string(),
                size: 98765432.into(),
            },
            quota: Quota {
                total: Some(2),
                used: 1,
            },
        };

        assert_eq!(expected, produced);
    }
}
