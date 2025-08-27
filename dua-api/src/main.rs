use axum::{
    middleware,
    routing::{delete, get, post, put},
    Extension, Router, Server,
};
use dotenv::dotenv;
use shared::{
    cache::Cache,
    config::AppConfig,
    database::Database,
    middleware::{
        cors_layer, rate_limit_middleware, timeout_layer, trace_layer,
    },
    SimpleRateLimiter,
    ApiResult,
};
use std::net::SocketAddr;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod handlers;
mod models;
mod repository;
mod services;

use handlers::{
    create_dua, delete_dua, get_dua_by_id, get_duas, health_check, search_duas, update_dua,
    get_categories, get_stats, verify_dua,
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
        .route("/api/v1/duas", get(get_duas))
        .route("/api/v1/duas", post(create_dua))
        .route("/api/v1/duas/:id", get(get_dua_by_id))
        .route("/api/v1/duas/:id", put(update_dua))
        .route("/api/v1/duas/:id", delete(delete_dua))
        .route("/api/v1/duas/search", get(search_duas))
        .route("/api/v1/duas/categories", get(get_categories))
        .route("/api/v1/duas/stats", get(get_stats))
        .route("/api/v1/duas/:id/verify", put(verify_dua))
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

    // Start the server
    let addr: SocketAddr = config.bind_address().parse()?;
    info!("Starting Dua API server on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}