// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;
use serde::{Deserialize, Serialize};

pub const DEFAULT_TOKENS_PER_SECOND: u16 = 10;
pub const DEFAULT_TOKEN_BUCKET_SIZE: u16 = 30;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RateLimitSettings::example_data())))]
pub struct RateLimitSettings {
    /// The number of tokens that are added to the bucket per second
    pub tokens_per_second: u16,
    /// The maximum amount of tokens that a token bucket can hold at a time
    pub token_bucket_size: u16,
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            tokens_per_second: DEFAULT_TOKENS_PER_SECOND,
            token_bucket_size: DEFAULT_TOKEN_BUCKET_SIZE,
        }
    }
}

impl ExampleData for RateLimitSettings {
    fn example_data() -> Self {
        Self::default()
    }
}
