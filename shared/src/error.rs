use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Timezone parsing error: {0}")]
    TimezoneParsing(String),

    #[error("Date parsing error: {0}")]
    DateParsing(String),

    #[error("Calculation error: {0}")]
    Calculation(String),

    #[error("Address parsing error: {0}")]
    AddressParsing(#[from] std::net::AddrParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Decimal parsing error: {0}")]
    DecimalParsing(#[from] rust_decimal::Error),

    #[error("Timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Redis(ref e) => {
                tracing::error!("Redis error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            ApiError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            ApiError::Config(ref e) => {
                tracing::error!("Configuration error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Serialization(ref e) => {
                tracing::error!("Serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Internal(ref e) => {
                tracing::error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::TimezoneParsing(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::DateParsing(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::Calculation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::AddressParsing(ref e) => {
                tracing::error!("Address parsing error: {}", e);
                (StatusCode::BAD_REQUEST, "Invalid address format")
            }
            ApiError::Io(ref e) => {
                tracing::error!("IO error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Uuid(ref e) => {
                tracing::error!("UUID error: {}", e);
                (StatusCode::BAD_REQUEST, "Invalid UUID format")
            }
            ApiError::DecimalParsing(ref e) => {
                tracing::error!("Decimal parsing error: {}", e);
                (StatusCode::BAD_REQUEST, "Invalid decimal value")
            }
            ApiError::Timeout(ref e) => {
                tracing::error!("Timeout error: {}", e);
                (StatusCode::REQUEST_TIMEOUT, "Request timeout")
            }
            ApiError::Http(ref msg) => (StatusCode::BAD_GATEWAY, msg.as_str()),
            ApiError::Authentication(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            ApiError::Authorization(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            ApiError::Network(ref msg) => (StatusCode::BAD_GATEWAY, msg.as_str()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

// Custom From implementations for types that need special handling
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::Validation(format!("Validation failed: {}", err))
    }
}

impl From<chrono::ParseError> for ApiError {
    fn from(err: chrono::ParseError) -> Self {
        ApiError::DateParsing(format!("Invalid date format: {}", err))
    }
}

impl From<hyper::Error> for ApiError {
    fn from(err: hyper::Error) -> Self {
        ApiError::Http(format!("HTTP error: {}", err))
    }
}

// Convenience constructors for common error patterns
impl ApiError {
    pub fn validation<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Validation(message.to_string())
    }

    pub fn invalid_input<T: std::fmt::Display>(message: T) -> Self {
        ApiError::InvalidInput(message.to_string())
    }

    pub fn not_found<T: std::fmt::Display>(resource: T) -> Self {
        ApiError::NotFound(format!("{} not found", resource))
    }

    pub fn calculation<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Calculation(message.to_string())
    }

    pub fn timezone_parsing<T: std::fmt::Display>(message: T) -> Self {
        ApiError::TimezoneParsing(message.to_string())
    }

    pub fn date_parsing<T: std::fmt::Display>(message: T) -> Self {
        ApiError::DateParsing(message.to_string())
    }

    pub fn authentication<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Authentication(message.to_string())
    }

    pub fn authorization<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Authorization(message.to_string())
    }

    pub fn network<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Network(message.to_string())
    }

    pub fn internal<T: std::fmt::Display>(message: T) -> Self {
        ApiError::Internal(anyhow::anyhow!(message.to_string()))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_validation_error() {
        let error = ApiError::validation("Invalid email format");
        let response = error.into_response();
        // In a real test, you'd extract the status code and body to verify
        // This is just a basic compile test
        assert!(response.status() == StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_error() {
        let error = ApiError::not_found("User");
        let response = error.into_response();
        assert!(response.status() == StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_rate_limit_error() {
        let error = ApiError::RateLimitExceeded;
        let response = error.into_response();
        assert!(response.status() == StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_from_conversions() {
        // Test automatic From implementations
        let _: ApiError = sqlx::Error::RowNotFound.into();
        let _: ApiError =
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "test")).into();
        let _: ApiError = std::net::AddrParseError {}.into();
    }
}
