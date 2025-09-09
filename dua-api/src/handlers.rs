use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use shared::{
    cache::Cache,
    database::Database,
    error::{ApiError, ApiResult},
};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::*,
    repository::DuaRepository,
    services::DuaService,
};

// ============= DUA ENDPOINTS =============

pub async fn list_duas(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Query(params): Query<DuaQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing duas with params: {:?}", params);
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let response = service.list_duas_with_filters(params).await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(id_or_slug): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting dua: {}", id_or_slug);
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    // Check if it's a UUID or slug
    let dua = if let Ok(id) = Uuid::parse_str(&id_or_slug) {
        service.get_dua_by_id(id, params.get("include").cloned()).await?
    } else {
        service.get_dua_by_slug(&id_or_slug, params.get("include").cloned()).await?
    };
    
    match dua {
        Some(dua) => Ok(Json(serde_json::to_value(dua)?)),
        None => Err(ApiError::NotFound(format!("Dua {} not found", id_or_slug))),
    }
}

pub async fn get_random_dua(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Query(params): Query<DuaQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting random dua with filters");
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let dua = service.get_random_dua(params).await?;
    
    match dua {
        Some(dua) => Ok(Json(serde_json::to_value(dua)?)),
        None => Err(ApiError::NotFound("No duas found matching criteria".to_string())),
    }
}

// ============= TRANSLATION ENDPOINTS =============

pub async fn get_dua_translations(
    Extension(database): Extension<Database>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting translations for dua: {}", id);
    
    let repository = DuaRepository::new(database);
    let translations = repository.get_dua_translations(id).await?;
    
    Ok(Json(serde_json::json!({
        "dua_id": id,
        "translations": translations,
        "total": translations.len()
    })))
}

pub async fn list_all_translations(
    Extension(database): Extension<Database>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing all translations");
    
    let repository = DuaRepository::new(database);
    let translations = repository.list_all_translations().await?;
    
    Ok(Json(serde_json::json!({
        "translations": translations,
        "total": translations.len()
    })))
}

// ============= CATEGORY ENDPOINTS =============

pub async fn list_categories(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing categories");
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let categories = service.list_categories().await?;
    Ok(Json(serde_json::to_value(categories)?))
}

pub async fn get_category(
    Extension(database): Extension<Database>,
    Path(slug): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting category: {}", slug);
    
    let repository = DuaRepository::new(database);
    let category = repository.get_category_by_slug(&slug).await?
        .ok_or_else(|| ApiError::NotFound(format!("Category {} not found", slug)))?;
    
    Ok(Json(serde_json::to_value(category)?))
}

pub async fn get_category_duas(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(slug): Path<String>,
    Query(mut params): Query<DuaQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting duas for category: {}", slug);
    
    // Set the category filter
    params.category = Some(slug.clone());
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let response = service.list_duas_with_filters(params).await?;
    Ok(Json(serde_json::to_value(response)?))
}

// ============= TAG ENDPOINTS =============

pub async fn list_tags(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing tags");
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let tags = service.list_tags().await?;
    Ok(Json(serde_json::to_value(tags)?))
}

pub async fn get_tag_duas(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(slug): Path<String>,
    Query(mut params): Query<DuaQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting duas for tag: {}", slug);
    
    // Set the tag filter
    params.tag = Some(slug.clone());
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let response = service.list_duas_with_filters(params).await?;
    Ok(Json(serde_json::to_value(response)?))
}

// ============= BUNDLE ENDPOINTS =============

pub async fn list_bundles(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing bundles");
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let bundles = service.list_bundles().await?;
    Ok(Json(serde_json::to_value(bundles)?))
}

pub async fn get_bundle(
    Extension(database): Extension<Database>,
    Path(slug): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting bundle: {}", slug);
    
    let repository = DuaRepository::new(database);
    let bundle = repository.get_bundle_by_slug(&slug).await?
        .ok_or_else(|| ApiError::NotFound(format!("Bundle {} not found", slug)))?;
    
    Ok(Json(serde_json::to_value(bundle)?))
}

pub async fn get_bundle_items(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Path(slug): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting items for bundle: {}", slug);
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let response = service.get_bundle_items(&slug).await?;
    Ok(Json(serde_json::to_value(response)?))
}

// ============= SOURCE ENDPOINTS =============

pub async fn list_sources(
    Extension(database): Extension<Database>,
    Query(params): Query<SourceQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Listing sources");
    
    let repository = DuaRepository::new(database);
    let sources = repository.list_sources(&params).await?;
    
    Ok(Json(serde_json::json!({
        "sources": sources,
        "total": sources.len()
    })))
}

pub async fn get_source(
    Extension(database): Extension<Database>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting source: {}", id);
    
    let repository = DuaRepository::new(database);
    let source = repository.get_source_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Source {} not found", id)))?;
    
    Ok(Json(serde_json::to_value(source)?))
}

pub async fn get_source_duas(
    Extension(database): Extension<Database>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting duas for source: {}", id);
    
    let repository = DuaRepository::new(database);
    let duas = repository.get_duas_by_source(id).await?;
    
    Ok(Json(serde_json::json!({
        "source_id": id,
        "duas": duas,
        "total": duas.len()
    })))
}

// ============= MEDIA ENDPOINTS =============

pub async fn get_dua_media(
    Extension(database): Extension<Database>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Getting media for dua: {}", id);
    
    let repository = DuaRepository::new(database);
    let media = repository.get_dua_media(id).await?;
    
    Ok(Json(serde_json::json!({
        "dua_id": id,
        "media": media,
        "total": media.len()
    })))
}

pub async fn search_media(
    Extension(database): Extension<Database>,
    Query(params): Query<MediaQueryParams>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Searching media");
    
    let repository = DuaRepository::new(database);
    let media = repository.search_media(&params).await?;
    
    Ok(Json(serde_json::json!({
        "media": media,
        "total": media.len()
    })))
}

// ============= SEARCH ENDPOINTS =============

pub async fn keyword_search(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params.get("limit")
        .and_then(|l| l.parse::<u32>().ok())
        .unwrap_or(20);
    
    info!("Keyword search for: {}", query);
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let results = service.keyword_search(&query, limit).await?;
    
    Ok(Json(serde_json::json!({
        "query": query,
        "results": results,
        "total": results.len()
    })))
}

pub async fn semantic_search(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Json(request): Json<SemanticSearchRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Semantic search for: {}", request.query);
    
    request.validate()
        .map_err(|e| ApiError::Validation(format!("Validation failed: {}", e)))?;
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let results = service.semantic_search(request).await?;
    Ok(Json(serde_json::to_value(results)?))
}

pub async fn autocomplete(
    Extension(database): Extension<Database>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params.get("limit")
        .and_then(|l| l.parse::<u32>().ok())
        .unwrap_or(10);
    
    info!("Autocomplete for: {}", query);
    
    let repository = DuaRepository::new(database);
    let suggestions = repository.get_suggestions(&query, limit).await?;
    
    Ok(Json(serde_json::json!({
        "query": query,
        "suggestions": suggestions
    })))
}

// ============= STATS ENDPOINT =============

pub async fn get_stats(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching statistics");
    
    let repository = DuaRepository::new(database);
    let service = DuaService::new(repository, cache);
    
    let stats = service.get_stats().await?;
    Ok(Json(serde_json::to_value(stats)?))
}

// ============= HEALTH CHECK =============

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
        "service": "dua-api",
        "version": "1.0.0",
        "database": "connected",
        "cache": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}