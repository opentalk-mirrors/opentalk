// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashMap, time::Instant};

use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_chat::RateLimitSettings;

#[derive(Debug)]
pub(super) struct RateLimit {
    settings: RateLimitSettings,
    buckets: HashMap<ConnectionId, Bucket>,
}

impl RateLimit {
    pub(super) fn new(settings: RateLimitSettings) -> Self {
        Self {
            settings,
            buckets: HashMap::new(),
        }
    }

    pub(super) fn insert_connection(&mut self, connection_id: ConnectionId) {
        let bucket = Bucket::new(self.settings.token_bucket_size.get());
        self.buckets.insert(connection_id, bucket);
    }

    pub(super) fn remove_connection(&mut self, connection_id: ConnectionId) {
        self.buckets.remove(&connection_id);
    }

    /// Consumes a token from the bucket
    ///
    /// Returns false if the bucket is empty
    pub(super) fn consume_token(&mut self, connection_id: ConnectionId) -> RateLimitState {
        let bucket = self.buckets.entry(connection_id).or_insert_with(|| {
            tracing::warn!(
                "Connection id '{connection_id}' not found in rate limit buckets, inserting."
            );
            Bucket::new(self.settings.token_bucket_size.get())
        });

        // Update the tokens based on the time since the last request
        bucket.add_tokens(&self.settings);

        let tokens = bucket.tokens;
        tracing::debug!(
            "Connection id '{connection_id}' has {tokens} of {} left",
            self.settings.token_bucket_size
        );

        if tokens == 0 {
            return RateLimitState::TooManyRequests {
                retry_after_ms: self
                    .settings
                    .refill_time_to_slow_down_threshold_ms()
                    .try_into()
                    .unwrap_or(u32::MAX),
            };
        }

        // Check if the connection should be slowed down, we subtract a small epsilon to account for
        // floating point inaccuracies
        let slow_down = (tokens as f32 / self.settings.token_bucket_size.get() as f32)
            - f32::EPSILON
            <= 1.0 - self.settings.slow_down_threshold;

        // Rate limit is enforced when overstepping the limit, so we consume the token after
        // checking
        bucket.consume_token();

        if slow_down {
            RateLimitState::SlowDown {
                recommended_wait_ms: self
                    .settings
                    .token_interval_ms()
                    .try_into()
                    .unwrap_or(u32::MAX),
            }
        } else {
            RateLimitState::Ok
        }
    }
}

#[derive(Debug)]
pub(super) struct Bucket {
    tokens: u64,
    timestamp: Instant,
}

impl Bucket {
    fn new(tokens: u64) -> Self {
        Self {
            tokens,
            timestamp: Instant::now(),
        }
    }

    /// Add tokens based on the time since the last request
    fn add_tokens(&mut self, settings: &RateLimitSettings) {
        let seconds = Instant::now().duration_since(self.timestamp).as_secs();
        let added_tokens = seconds * settings.tokens_per_second.get();
        self.tokens = std::cmp::min(self.tokens + added_tokens, settings.token_bucket_size.get());
    }

    /// Consume a token from the bucket and update the timestamp of the last request
    fn consume_token(&mut self) {
        self.tokens = self.tokens.saturating_sub(1);
        self.timestamp = Instant::now();
    }
}

pub(super) enum RateLimitState {
    Ok,
    SlowDown {
        /// The time the client should wait before sending another chat message
        recommended_wait_ms: u32,
    },
    TooManyRequests {
        /// The time the client should wait before sending another chat message
        retry_after_ms: u32,
    },
}
