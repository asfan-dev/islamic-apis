use axum::{
    middleware,
    routing::{get, post},
    Extension, Router, Server,
};
use dotenv::dotenv;
use shared::{
    cache::Cache,
    config::AppConfig,
    middleware::{cors_layer, rate_limit_middleware, timeout_layer, trace_layer},
    rate_limit::RateLimiter,
    ApiResult,
};
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod calculations;
mod handlers;
mod models;
mod preferred;
mod services;

use handlers::prayer_times_handler;
use preferred::PreferredMethodMap;

#[tokio::main]
async fn main() -> ApiResult<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        shared::error::ApiError::internal(format!("Failed to set tracing subscriber: {}", e))
    })?;

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize cache (Redis)
    let cache = Cache::new(&config.redis).await?;
    info!("Cache connected successfully");

    // Initialize rate limiter
    let rate_limiter = RateLimiter::new(cache.clone(), config.rate_limit.clone());

    // Load preferred methods
    let preferred_methods = Arc::new(PreferredMethodMap::load("preferred.csv")?);
    info!("Preferred methods loaded successfully");

    // Build the application
    let app = Router::new()
        .route("/api/v1/prayer-times", post(prayer_times_handler))
        .route("/health", get(health_check))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(timeout_layer())
        .layer(cors_layer())
        .layer(trace_layer())
        .layer(Extension(cache))
        .layer(Extension(preferred_methods));

    // Start the server - using axum 0.6 syntax like the working zakat example
    let addr: SocketAddr = config.bind_address().parse()?;
    info!("Starting Prayer Times API server on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn health_check(Extension(cache): Extension<Cache>) -> ApiResult<&'static str> {
    cache.health_check().await?;
    Ok("OK")
}
