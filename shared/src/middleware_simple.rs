// Simplified middleware without compression to avoid build issues
// Use this if you're having tower-http compression problems

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

use crate::{error::ApiError, rate_limit::RateLimiter};

/// Creates CORS layer with permissive settings for public APIs
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(86400)) // 24 hours
}

/// Creates timeout layer with 30 second timeout
pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}

/// Creates tracing layer for request logging
pub fn trace_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown")
        .to_string();

    if !rate_limiter.check_rate_limit(&client_ip).await? {
        info!("Rate limit exceeded for IP: {}", client_ip);
        return Err(ApiError::RateLimitExceeded);
    }

    let response = next.run(req).await;
    Ok(response)
}

/// Request ID middleware for tracing
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    req.headers_mut()
        .insert("x-request-id", request_id.parse().unwrap());

    let response = next.run(req).await;
    response
}
