// Simple rate limiting implementation without external dependencies
// This avoids potential compatibility issues with governor crate

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::{cache::Cache, config::RateLimitConfig, error::ApiResult};

#[derive(Clone)]
pub struct SimpleRateLimiter {
    cache: Cache,
    config: RateLimitConfig,
    local_cache: std::sync::Arc<RwLock<HashMap<String, (u32, Instant)>>>,
}

impl SimpleRateLimiter {
    pub fn new(cache: Cache, config: RateLimitConfig) -> Self {
        Self {
            cache,
            config,
            local_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> ApiResult<bool> {
        // Try Redis first (for distributed rate limiting)
        if let Ok(allowed) = self.check_redis_rate_limit(identifier).await {
            return Ok(allowed);
        }

        // Fallback to in-memory rate limiting
        self.check_local_rate_limit(identifier).await
    }

    async fn check_redis_rate_limit(&self, identifier: &str) -> ApiResult<bool> {
        let key = format!("rate_limit:{}", identifier);
        let window = Duration::from_secs(60); // 1 minute window

        // Get current count
        let current_count: i64 = self.cache.increment(&key, 1, Some(window)).await?;

        debug!(
            "Redis rate limit check for {}: {}/{}",
            identifier, current_count, self.config.requests_per_minute
        );

        if current_count > self.config.requests_per_minute as i64 {
            warn!(
                "Redis rate limit exceeded for {}: {}",
                identifier, current_count
            );
            return Ok(false);
        }

        Ok(true)
    }

    async fn check_local_rate_limit(&self, identifier: &str) -> ApiResult<bool> {
        let now = Instant::now();
        let window = Duration::from_secs(60);

        let mut cache = self.local_cache.write().await;

        // Clean up old entries
        cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < window);

        // Check current count for this identifier
        let entry = cache.entry(identifier.to_string()).or_insert((0, now));

        // Reset count if window has passed
        if now.duration_since(entry.1) >= window {
            entry.0 = 0;
            entry.1 = now;
        }

        entry.0 += 1;

        debug!(
            "Local rate limit check for {}: {}/{}",
            identifier, entry.0, self.config.requests_per_minute
        );

        if entry.0 > self.config.requests_per_minute {
            warn!("Local rate limit exceeded for {}: {}", identifier, entry.0);
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn get_remaining_requests(&self, identifier: &str) -> ApiResult<i64> {
        // Try Redis first
        if let Ok(remaining) = self.get_redis_remaining(identifier).await {
            return Ok(remaining);
        }

        // Fallback to local cache
        self.get_local_remaining(identifier).await
    }

    async fn get_redis_remaining(&self, identifier: &str) -> ApiResult<i64> {
        let key = format!("rate_limit:{}", identifier);

        match self.cache.get::<i64>(&key).await? {
            Some(count) => Ok((self.config.requests_per_minute as i64 - count).max(0)),
            None => Ok(self.config.requests_per_minute as i64),
        }
    }

    async fn get_local_remaining(&self, identifier: &str) -> ApiResult<i64> {
        let cache = self.local_cache.read().await;

        match cache.get(identifier) {
            Some((count, _)) => Ok((self.config.requests_per_minute as i64 - *count as i64).max(0)),
            None => Ok(self.config.requests_per_minute as i64),
        }
    }

    pub async fn reset_rate_limit(&self, identifier: &str) -> ApiResult<()> {
        // Reset in Redis
        let redis_key = format!("rate_limit:{}", identifier);
        let _ = self.cache.delete(&redis_key).await;

        // Reset in local cache
        let mut cache = self.local_cache.write().await;
        cache.remove(identifier);

        Ok(())
    }
}

// Simple alias for backwards compatibility
pub type RateLimiter = SimpleRateLimiter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RedisConfig;

    #[tokio::test]
    async fn test_local_rate_limiter() {
        let redis_config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            pool_max_open: 10,
            pool_max_idle: 5,
            pool_timeout: 30,
            pool_expire: 300,
        };

        // This will fail gracefully and use local cache
        let cache = Cache::new(&redis_config).await.unwrap_or_else(|_| {
            // Mock cache that always fails (for testing local fallback)
            panic!("This test requires a mock cache implementation")
        });

        let rate_limit_config = RateLimitConfig {
            requests_per_minute: 5,
            burst_size: 2,
            cleanup_interval: 60,
        };

        let rate_limiter = SimpleRateLimiter::new(cache, rate_limit_config);
        let test_id = "test_user";

        // This test would need proper mocking to work
        // For now, it's just a structure example
    }
}
