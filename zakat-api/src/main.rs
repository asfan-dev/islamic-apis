use axum::{
    middleware,
    routing::{get, post},
    Extension, Router, Server,
};
use dotenv::dotenv;
use shared::{
    cache::Cache,
    config::AppConfig,
    database::Database,
    error::ApiResult,
    middleware::{cors_layer, rate_limit_middleware, timeout_layer, trace_layer},
    SimpleRateLimiter,
};
use std::net::SocketAddr;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod calculations;
mod handlers;
mod models;
mod repository;
mod services;

use handlers::{
    calculate_zakat, get_calculation_history, get_nisab_rates, get_zakat_info, health_check,
    save_calculation,
};

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

    // Initialize database
    let database = Database::new(&config.database).await?;
    info!("Database connected successfully");

    // Initialize cache (Redis)
    let cache = Cache::new(&config.redis).await?;
    info!("Cache connected successfully");

    // Initialize rate limiter
    let rate_limiter = SimpleRateLimiter::new(cache.clone(), config.rate_limit.clone());

    // Build the application
    let app = Router::new()
        .route("/api/v1/zakat/calculate", post(calculate_zakat))
        .route("/api/v1/zakat/save", post(save_calculation))
        .route("/api/v1/zakat/history", get(get_calculation_history))
        .route("/api/v1/zakat/nisab", get(get_nisab_rates))
        .route("/api/v1/zakat/info", get(get_zakat_info))
        .route("/health", get(health_check))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(timeout_layer())
        .layer(cors_layer())
        .layer(trace_layer())
        .layer(Extension(database))
        .layer(Extension(cache));

    // Start the server - using axum 0.6 syntax
    let addr: SocketAddr = config.bind_address().parse()?;
    info!("Starting Zakat API server on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
