// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_roomserver_types::rate_limit::RateLimitSettings;
use tokio::time::Instant;

/// The interval at which a client is informed to slow down when overstepping the rate limit.
const SLOW_DOWN_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct RateLimit {
    tokens_per_second: u16,
    bucket_size: u16,
    tokens: f64,
    last_refill: Instant,
    last_slow_down: Instant,
}

impl From<RateLimitSettings> for RateLimit {
    fn from(settings: RateLimitSettings) -> Self {
        Self::new(settings.tokens_per_second, settings.token_bucket_size)
    }
}

impl RateLimit {
    pub fn new(tokens_per_second: u16, bucket_size: u16) -> Self {
        Self {
            tokens_per_second,
            bucket_size,
            tokens: bucket_size.into(),
            last_refill: Instant::now(),
            last_slow_down: Instant::now() - SLOW_DOWN_INTERVAL,
        }
    }

    pub async fn wait_for_token(&mut self) -> bool {
        let mut slow_down = false;
        loop {
            self.refill();

            if self.tokens >= 1.0 {
                self.tokens -= 1.0;
                return slow_down;
            }

            let missing = 1.0 - self.tokens;
            let wait_seconds = missing / f64::from(self.tokens_per_second);
            tokio::time::sleep(Duration::from_secs_f64(wait_seconds)).await;

            if Instant::now().duration_since(self.last_slow_down) > SLOW_DOWN_INTERVAL {
                self.last_slow_down = Instant::now();
                slow_down = true;
            }
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed <= 0.0 {
            return;
        }

        let added = elapsed * f64::from(self.tokens_per_second);
        self.tokens = (self.tokens + added).min(self.bucket_size.into());
        self.last_refill = now;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::FutureExt as _;

    use crate::message_router::rate_limit::RateLimit;

    #[test_log::test(tokio::test(start_paused = true))]
    async fn rate_limit() {
        let mut rate_limit = RateLimit::new(1, 2);
        // Consume 2 tokens
        rate_limit.wait_for_token().now_or_never().unwrap();
        rate_limit.wait_for_token().now_or_never().unwrap();

        // No tokens left
        assert_eq!(rate_limit.wait_for_token().now_or_never(), None);
        tokio::time::advance(Duration::from_secs(1)).await;

        // 1 token refilled
        rate_limit.wait_for_token().now_or_never().unwrap();
    }
}
