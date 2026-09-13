// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::utils::ExampleData;
use serde::{Deserialize, Serialize};

/// Quota information
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "api-v1/")
)]
pub struct Quota {
    /// The total quota available, if applicable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// The used quota
    pub used: u64,
}

impl Quota {
    /// Returns true when the quota has been exceeded
    pub fn is_exceeded(&self) -> bool {
        match self.total {
            Some(total) => self.used >= total,
            None => false,
        }
    }
}

impl ExampleData for Quota {
    fn example_data() -> Self {
        Self {
            total: Some(5 * 1024 * 1024 * 1024), // 5GiB
            used: 2 * 1024 * 1024 * 1024,        // 2GiB
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::utils::ExampleData as _;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::Quota;

    #[test]
    fn serialize_quota() {
        let quota = Quota::example_data();
        let produced = serde_json::to_value(quota).expect("Quota must be serializable");

        assert_eq!(
            produced,
            json!({
                "total": 5368709120u64,
                "used": 2147483648u64,
            })
        );
    }

    #[test]
    fn deserialize_quota() {
        let value = json!({
            "total": 10,
            "used": 5,
        });
        let produced: Quota = serde_json::from_value(value).unwrap();

        assert_eq!(
            produced,
            Quota {
                total: Some(10),
                used: 5
            }
        );
    }
}
