use axum::{
    middleware,
    routing::{get, post},
    Extension, Router, Server,
};
use dotenv::dotenv;
use shared::{
    cache::Cache,
    config::AppConfig,
    middleware::{
        cors_layer, rate_limit_middleware, timeout_layer, trace_layer,
    },
    SimpleRateLimiter,
    ApiResult,
};
use std::net::SocketAddr;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod calculations;
mod handlers;
mod models;

use handlers::{health_check, qibla_handler};

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

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize cache (Redis)
    let cache = Cache::new(&config.redis).await?;
    info!("Cache connected successfully");

    // Initialize rate limiter
    let rate_limiter = SimpleRateLimiter::new(cache.clone(), config.rate_limit.clone());

    // Build the application
    let app = Router::new()
        .route("/api/v1/qibla", post(qibla_handler))
        .route("/api/v1/qibla", get(qibla_handler)) // Support GET for simple queries
        .route("/health", get(health_check))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .layer(timeout_layer())
        .layer(cors_layer())
        .layer(trace_layer())
        .layer(Extension(cache));

    // Start the server
    let addr: SocketAddr = config.bind_address().parse()?;
    info!("Starting Qibla API server on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}