use anyhow::anyhow;
use redis::{aio::ConnectionManager, cmd, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

use crate::{config::RedisConfig, error::ApiResult};

#[derive(Clone)]
pub struct Cache {
    connection: ConnectionManager,
}

impl Cache {
    pub async fn new(config: &RedisConfig) -> ApiResult<Self> {
        info!("Connecting to Redis...");

        let client = Client::open(config.url.as_str())?;
        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e))?;

        info!("Redis connected successfully");
        Ok(Cache { connection })
    }

    pub async fn get<T>(&self, key: &str) -> ApiResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.connection.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> ApiResult<()>
    where
        T: Serialize,
    {
        let mut conn = self.connection.clone();
        let serialized = serde_json::to_string(value)?;

        match ttl {
            Some(duration) => {
                // Fix: Convert u64 to usize safely
                let seconds = duration
                    .as_secs()
                    .try_into()
                    .map_err(|_| crate::error::ApiError::Internal(anyhow!("Duration too large")))?;
                let _: () = conn.set_ex(key, serialized, seconds).await?;
            }
            None => {
                let _: () = conn.set(key, serialized).await?;
            }
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> ApiResult<()> {
        let mut conn = self.connection.clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> ApiResult<bool> {
        let mut conn = self.connection.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn increment(&self, key: &str, by: i64, ttl: Option<Duration>) -> ApiResult<i64> {
        let mut conn = self.connection.clone();
        let result: i64 = conn.incr(key, by).await?;

        if let Some(duration) = ttl {
            let seconds = duration
                .as_secs()
                .try_into()
                .map_err(|_| crate::error::ApiError::Internal(anyhow!("Duration too large")))?;
            let _: () = conn.expire(key, seconds).await?;
        }

        Ok(result)
    }

    pub async fn health_check(&self) -> ApiResult<()> {
        let mut conn = self.connection.clone();
        // Fix: Use cmd to execute PING command
        let _: String = cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_operations() {
        // Skip if REDIS_URL is not set
        if std::env::var("REDIS_URL").is_err() {
            return;
        }

        let config = RedisConfig {
            url: std::env::var("REDIS_URL").unwrap(),
            pool_max_open: 10,
            pool_max_idle: 5,
            pool_timeout: 30,
            pool_expire: 300,
        };

        let cache = Cache::new(&config).await;
        assert!(cache.is_ok());

        if let Ok(cache) = cache {
            let test_key = "test_key";
            let test_value = json!({"test": "value"});

            // Test set and get
            assert!(cache.set(test_key, &test_value, None).await.is_ok());
            let retrieved: Option<serde_json::Value> = cache.get(test_key).await.unwrap();
            assert_eq!(retrieved, Some(test_value));

            // Test delete
            assert!(cache.delete(test_key).await.is_ok());
            let retrieved: Option<serde_json::Value> = cache.get(test_key).await.unwrap();
            assert_eq!(retrieved, None);

            // Test health check
            assert!(cache.health_check().await.is_ok());
        }
    }
}
