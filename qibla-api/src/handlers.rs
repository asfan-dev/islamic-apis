use axum::{extract::Query, Extension, Json};
use serde::Deserialize;
use shared::{cache::Cache, error::ApiResult};
use std::time::Duration;
use tracing::{debug, info};
use validator::Validate;

use crate::{
    calculations::QiblaCalculator,
    models::QiblaRequest,
};

#[derive(Debug, Deserialize)]
pub struct QiblaQueryParams {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub elevation: Option<f64>,
    pub detailed: Option<bool>,
}

pub async fn qibla_handler(
    Extension(cache): Extension<Cache>,
    query: Option<Query<QiblaQueryParams>>,
    body: Option<Json<QiblaRequest>>,
) -> ApiResult<Json<serde_json::Value>> {
    // Handle both GET (query params) and POST (JSON body) requests
    let (request, detailed) = if let Some(Json(req)) = body {
        (req, false) // Default detailed to false for JSON body
    } else if let Some(Query(ref params)) = query {
        if let (Some(lat), Some(lng)) = (params.lat, params.lng) {
            let request = QiblaRequest {
                latitude: lat,
                longitude: lng,
                elevation: params.elevation,
            };
            let detailed = params.detailed.unwrap_or(false);
            (request, detailed)
        } else {
            return Err(shared::error::ApiError::InvalidInput(
                "Latitude and longitude are required as 'lat' and 'lng' parameters".to_string(),
            ));
        }
    } else {
        return Err(shared::error::ApiError::InvalidInput(
            "Request body or query parameters required".to_string(),
        ));
    };

    info!(
        "Processing qibla request for coordinates: {:.4}, {:.4}",
        request.latitude, request.longitude
    );

    // Validate the request
    request
        .validate()
        .map_err(|e| shared::error::ApiError::Validation(format!("Validation failed: {}", e)))?;

    // Create cache key
    let cache_key = create_cache_key(&request, detailed);

    // Try to get from cache first
    if let Ok(Some(cached_response)) = cache.get::<serde_json::Value>(&cache_key).await {
        debug!("Returning cached qibla calculation for key: {}", cache_key);
        return Ok(Json(cached_response));
    }

    // Create calculator
    let (lat, lng, elevation) = request.to_coordinates();
    let calculator = QiblaCalculator::new(lat, lng, elevation);

    // Calculate qibla direction
    let response = if detailed {
        let detailed_result = calculator.calculate_detailed_qibla()?;
        serde_json::to_value(detailed_result)?
    } else {
        let basic_result = calculator.calculate_qibla_direction()?;
        serde_json::to_value(basic_result)?
    };

    // Cache the response for 24 hours (qibla direction doesn't change frequently)
    if let Err(e) = cache
        .set(&cache_key, &response, Some(Duration::from_secs(86400)))
        .await
    {
        tracing::warn!("Failed to cache qibla response: {}", e);
    }

    info!("Successfully calculated qibla direction");
    Ok(Json(response))
}

pub async fn health_check(Extension(cache): Extension<Cache>) -> ApiResult<&'static str> {
    cache.health_check().await?;
    Ok("OK")
}

fn create_cache_key(request: &QiblaRequest, detailed: bool) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash coordinates with limited precision for better cache hits
    let lat_rounded = (request.latitude * 10000.0).round() / 10000.0;
    let lng_rounded = (request.longitude * 10000.0).round() / 10000.0;
    let elevation_rounded = request.elevation.unwrap_or(0.0).round();

    lat_rounded.to_bits().hash(&mut hasher);
    lng_rounded.to_bits().hash(&mut hasher);
    elevation_rounded.to_bits().hash(&mut hasher);
    detailed.hash(&mut hasher);

    format!("qibla:{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let request = QiblaRequest {
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: Some(10.0),
        };

        let key1 = create_cache_key(&request, false);
        let key2 = create_cache_key(&request, false);
        let key3 = create_cache_key(&request, true);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_similar_coordinates_same_cache() {
        let request1 = QiblaRequest {
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: Some(10.0),
        };

        let request2 = QiblaRequest {
            latitude: 40.71279, // Very slightly different
            longitude: -74.00599,
            elevation: Some(10.0),
        };

        let key1 = create_cache_key(&request1, false);
        let key2 = create_cache_key(&request2, false);

        // Should be the same due to rounding
        assert_eq!(key1, key2);
    }
}