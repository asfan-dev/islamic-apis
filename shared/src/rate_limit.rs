use std::time::Duration;
use tracing::{debug, warn};

use crate::{cache::Cache, config::RateLimitConfig, error::ApiResult};

#[derive(Clone)]
pub struct RateLimiter {
    cache: Cache,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(cache: Cache, config: RateLimitConfig) -> Self {
        Self { cache, config }
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> ApiResult<bool> {
        let key = format!("rate_limit:{}", identifier);
        let window = Duration::from_secs(60); // 1 minute window

        // Get current count
        let current_count: i64 = self.cache.increment(&key, 1, Some(window)).await?;

        debug!(
            "Rate limit check for {}: {}/{}",
            identifier, current_count, self.config.requests_per_minute
        );

        if current_count > self.config.requests_per_minute as i64 {
            warn!("Rate limit exceeded for {}: {}", identifier, current_count);
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn get_remaining_requests(&self, identifier: &str) -> ApiResult<i64> {
        let key = format!("rate_limit:{}", identifier);

        match self.cache.get::<i64>(&key).await? {
            Some(count) => Ok((self.config.requests_per_minute as i64 - count).max(0)),
            None => Ok(self.config.requests_per_minute as i64),
        }
    }

    pub async fn reset_rate_limit(&self, identifier: &str) -> ApiResult<()> {
        let key = format!("rate_limit:{}", identifier);
        self.cache.delete(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    #[tokio::test]
    async fn test_rate_limiter() {
        // Skip if REDIS_URL is not set
        if std::env::var("REDIS_URL").is_err() {
            return;
        }

        let redis_config = RedisConfig {
            url: std::env::var("REDIS_URL").unwrap(),
            pool_max_open: 10,
            pool_max_idle: 5,
            pool_timeout: 30,
            pool_expire: 300,
        };

        let cache = Cache::new(&redis_config).await.unwrap();

        let rate_limit_config = RateLimitConfig {
            requests_per_minute: 5,
            burst_size: 2,
            cleanup_interval: 60,
        };

        let rate_limiter = RateLimiter::new(cache, rate_limit_config);
        let test_id = "test_user";

        // Reset any existing rate limit
        rate_limiter.reset_rate_limit(test_id).await.unwrap();

        // Test normal requests
        for i in 1..=5 {
            let allowed = rate_limiter.check_rate_limit(test_id).await.unwrap();
            assert!(allowed, "Request {} should be allowed", i);
        }

        // Test rate limit exceeded
        let allowed = rate_limiter.check_rate_limit(test_id).await.unwrap();
        assert!(!allowed, "Request should be rate limited");

        // Clean up
        rate_limiter.reset_rate_limit(test_id).await.unwrap();
    }
}
