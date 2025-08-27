use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use shared::{
    cache::Cache,
    database::Database,
    error::{ApiError, ApiResult},
};
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{CreateDuaRequest, SearchDuaQuery, UpdateDuaRequest},
    repository::DuaRepository,
    services::DuaService,
};

pub async fn create_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Json(request): Json<CreateDuaRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Creating new dua: {}", request.title);

    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let response = service.create_dua(request).await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_duas(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Query(query): Query<SearchDuaQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching duas with query: {:?}", query);

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let response = service.search_duas(query).await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_dua_by_id(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching dua by ID: {}", id);

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    match service.get_dua_by_id(id).await? {
        Some(response) => Ok(Json(serde_json::to_value(response)?)),
        None => Err(ApiError::NotFound(format!("Dua with ID {} not found", id))),
    }
}

pub async fn update_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDuaRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Updating dua with ID: {}", id);

    // Validate request
    request
        .validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    match service.update_dua(id, request).await? {
        Some(response) => Ok(Json(serde_json::to_value(response)?)),
        None => Err(ApiError::NotFound(format!("Dua with ID {} not found", id))),
    }
}

pub async fn delete_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Deleting dua with ID: {}", id);

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let deleted = service.delete_dua(id).await?;

    if deleted {
        Ok(Json(serde_json::json!({
            "message": "Dua deleted successfully",
            "id": id
        })))
    } else {
        Err(ApiError::NotFound(format!("Dua with ID {} not found", id)))
    }
}

pub async fn search_duas(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Query(query): Query<SearchDuaQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Searching duas with query: {:?}", query);

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let response = service.search_duas(query).await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_categories(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching dua categories");

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let response = service.get_categories().await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_stats(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching dua statistics");

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    let response = service.get_stats().await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn verify_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(id): Path<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Updating verification status for dua: {}", id);

    let verified = request
        .get("verified")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| {
            ApiError::InvalidInput("'verified' field is required and must be a boolean".to_string())
        })?;

    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);

    match service.verify_dua(id, verified).await? {
        Some(response) => Ok(Json(serde_json::to_value(response)?)),
        None => Err(ApiError::NotFound(format!("Dua with ID {} not found", id))),
    }
}

pub async fn health_check(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check database connection
    database.health_check().await?;

    // Check cache connection
    cache.health_check().await?;

    Ok(Json(serde_json::json!({
        "status": "OK",
        "database": "connected",
        "cache": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateDuaRequest;

    #[test]
    fn test_create_dua_request_validation() {
        let valid_request = CreateDuaRequest {
            title: "Test Dua".to_string(),
            arabic_text: "بسم الله".to_string(),
            transliteration: Some("Bismillah".to_string()),
            translation: "In the name of Allah".to_string(),
            reference: Some("Quran 1:1".to_string()),
            category: "Daily Duas".to_string(),
            tags: Some(vec!["daily".to_string(), "basic".to_string()]),
            audio_url: Some("https://example.com/audio.mp3".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateDuaRequest {
            title: "".to_string(), // Empty title
            arabic_text: "بسم الله".to_string(),
            transliteration: None,
            translation: "In the name of Allah".to_string(),
            reference: None,
            category: "Daily Duas".to_string(),
            tags: None,
            audio_url: None,
        };

        assert!(invalid_request.validate().is_err());
    }
}