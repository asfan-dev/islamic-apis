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

use handlers::*;

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

    // Build the application router
    let app = Router::new()
        // ===== DUA ENDPOINTS =====
        .route("/v1/duas", get(list_duas))
        .route("/v1/duas/random", get(get_random_dua))
        .route("/v1/duas/:id", get(get_dua))
        
        // ===== TRANSLATION ENDPOINTS =====
        .route("/v1/duas/:id/translations", get(get_dua_translations))
        .route("/v1/translations", get(list_all_translations))
        
        // ===== CATEGORY ENDPOINTS =====
        .route("/v1/categories", get(list_categories))
        .route("/v1/categories/:slug", get(get_category))
        .route("/v1/categories/:slug/duas", get(get_category_duas))
        
        // ===== TAG ENDPOINTS =====
        .route("/v1/tags", get(list_tags))
        .route("/v1/tags/:slug/duas", get(get_tag_duas))
        
        // ===== BUNDLE ENDPOINTS =====
        .route("/v1/bundles", get(list_bundles))
        .route("/v1/bundles/:slug", get(get_bundle))
        .route("/v1/bundles/:slug/items", get(get_bundle_items))
        
        // ===== SOURCE ENDPOINTS =====
        .route("/v1/sources", get(list_sources))
        .route("/v1/sources/:id", get(get_source))
        .route("/v1/sources/:id/duas", get(get_source_duas))
        
        // ===== MEDIA ENDPOINTS =====
        .route("/v1/duas/:id/media", get(get_dua_media))
        .route("/v1/media", get(search_media))
        
        // ===== SEARCH ENDPOINTS =====
        .route("/v1/search", get(keyword_search))
        .route("/v1/search/semantic", post(semantic_search))
        .route("/v1/suggest", get(autocomplete))
        
        // ===== STATS ENDPOINT =====
        .route("/v1/stats", get(get_stats))
        
        // ===== HEALTH CHECK =====
        .route("/health", get(health_check))
        
        // Apply middleware layers
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
    info!("🚀 Starting Dua API server on {}", addr);
    info!("📚 API Documentation available at http://{}/docs", addr);
    info!("🏥 Health check available at http://{}/health", addr);

    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}