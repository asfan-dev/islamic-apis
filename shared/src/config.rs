use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout: u64,
    pub pool_expire: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub cleanup_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rate_limit: RateLimitConfig,
    pub rust_log: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("database.max_connections", 100)?
            .set_default("database.min_connections", 5)?
            .set_default("database.connect_timeout", 30)?
            .set_default("database.idle_timeout", 600)?
            .set_default("redis.pool_max_open", 100)?
            .set_default("redis.pool_max_idle", 20)?
            .set_default("redis.pool_timeout", 30)?
            .set_default("redis.pool_expire", 300)?
            .set_default("rate_limit.requests_per_minute", 100)?
            .set_default("rate_limit.burst_size", 10)?
            .set_default("rate_limit.cleanup_interval", 60)?
            .set_default("rust_log", "info")?
            .add_source(Environment::default().separator("__"))
            .build()?;

        config.try_deserialize()
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
