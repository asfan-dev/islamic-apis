use shared::{cache::Cache, error::ApiResult};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    models::{
        CreateDuaRequest, Dua, DuaCategoriesResponse, DuaCategory, DuaListResponse, DuaResponse,
        DuaStatsResponse, SearchDuaQuery, UpdateDuaRequest,
    },
    repository::DuaRepository,
};

pub struct DuaService {
    repository: DuaRepository,
    cache: Cache,
}

impl DuaService {
    pub fn new(repository: DuaRepository, cache: Cache) -> Self {
        Self { repository, cache }
    }

    pub async fn create_dua(&self, request: CreateDuaRequest) -> ApiResult<DuaResponse> {
        let dua = request.into_dua();
        let created_dua = self.repository.create(dua).await?;

        // Invalidate relevant caches
        self.invalidate_list_caches().await;

        Ok(DuaResponse { dua: created_dua })
    }

    pub async fn get_dua_by_id(&self, id: Uuid) -> ApiResult<Option<DuaResponse>> {
        let cache_key = format!("dua:{}", id);

        // Try cache first
        if let Ok(Some(cached_dua)) = self.cache.get::<Dua>(&cache_key).await {
            debug!("Returning cached dua for ID: {}", id);
            return Ok(Some(DuaResponse { dua: cached_dua }));
        }

        // Fetch from database
        if let Some(dua) = self.repository.get_by_id(id).await? {
            // Cache for 1 hour
            if let Err(e) = self
                .cache
                .set(&cache_key, &dua, Some(Duration::from_secs(3600)))
                .await
            {
                warn!("Failed to cache dua {}: {}", id, e);
            }

            Ok(Some(DuaResponse { dua }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_dua(
        &self,
        id: Uuid,
        request: UpdateDuaRequest,
    ) -> ApiResult<Option<DuaResponse>> {
        if let Some(updated_dua) = self.repository.update(id, request).await? {
            // Invalidate caches
            let cache_key = format!("dua:{}", id);
            if let Err(e) = self.cache.delete(&cache_key).await {
                warn!("Failed to invalidate cache for dua {}: {}", id, e);
            }
            self.invalidate_list_caches().await;

            Ok(Some(DuaResponse { dua: updated_dua }))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_dua(&self, id: Uuid) -> ApiResult<bool> {
        let deleted = self.repository.delete(id).await?;

        if deleted {
            // Invalidate caches
            let cache_key = format!("dua:{}", id);
            if let Err(e) = self.cache.delete(&cache_key).await {
                warn!("Failed to invalidate cache for dua {}: {}", id, e);
            }
            self.invalidate_list_caches().await;
        }

        Ok(deleted)
    }

    pub async fn search_duas(&self, query: SearchDuaQuery) -> ApiResult<DuaListResponse> {
        let cache_key = self.create_search_cache_key(&query);

        // Try cache first for frequently used searches
        if query.q.is_none() && query.category.is_none() && query.tags.is_none() {
            if let Ok(Some(cached_response)) = self.cache.get::<DuaListResponse>(&cache_key).await {
                debug!("Returning cached search results");
                return Ok(cached_response);
            }
        }

        // Fetch from database
        let (duas, total) = self.repository.search(query).await?;
        let total_pages = ((total as f64) / (query.get_limit() as f64)).ceil() as u32;

        let response = DuaListResponse {
            duas,
            total,
            page: query.get_page(),
            limit: query.get_limit(),
            total_pages,
        };

        // Cache simple queries for 10 minutes
        if query.q.is_none() && query.category.is_none() && query.tags.is_none() {
            if let Err(e) = self
                .cache
                .set(&cache_key, &response, Some(Duration::from_secs(600)))
                .await
            {
                warn!("Failed to cache search results: {}", e);
            }
        }

        Ok(response)
    }

    pub async fn get_categories(&self) -> ApiResult<DuaCategoriesResponse> {
        let cache_key = "dua_categories";

        // Try cache first
        if let Ok(Some(cached_categories)) = self.cache.get::<Vec<DuaCategory>>(&cache_key).await {
            debug!("Returning cached categories");
            return Ok(DuaCategoriesResponse {
                categories: cached_categories,
            });
        }

        // Fetch from database
        let categories = self.repository.get_categories().await?;

        // Cache for 30 minutes
        if let Err(e) = self
            .cache
            .set(&cache_key, &categories, Some(Duration::from_secs(1800)))
            .await
        {
            warn!("Failed to cache categories: {}", e);
        }

        Ok(DuaCategoriesResponse { categories })
    }

    pub async fn get_stats(&self) -> ApiResult<DuaStatsResponse> {
        let cache_key = "dua_stats";

        // Try cache first
        if let Ok(Some(cached_stats)) = self.cache.get::<DuaStatsResponse>(&cache_key).await {
            debug!("Returning cached stats");
            return Ok(cached_stats);
        }

        // Fetch from database
        let (total_duas, verified_duas, total_categories, most_popular_category, recent_additions) =
            self.repository.get_stats().await?;

        let stats = DuaStatsResponse {
            total_duas,
            verified_duas,
            total_categories,
            most_popular_category,
            recent_additions,
        };

        // Cache for 15 minutes
        if let Err(e) = self
            .cache
            .set(&cache_key, &stats, Some(Duration::from_secs(900)))
            .await
        {
            warn!("Failed to cache stats: {}", e);
        }

        Ok(stats)
    }

    pub async fn verify_dua(&self, id: Uuid, verified: bool) -> ApiResult<Option<DuaResponse>> {
        if let Some(updated_dua) = self.repository.verify_dua(id, verified).await? {
            // Invalidate caches
            let cache_key = format!("dua:{}", id);
            if let Err(e) = self.cache.delete(&cache_key).await {
                warn!("Failed to invalidate cache for dua {}: {}", id, e);
            }
            self.invalidate_list_caches().await;

            Ok(Some(DuaResponse { dua: updated_dua }))
        } else {
            Ok(None)
        }
    }

    async fn invalidate_list_caches(&self) {
        let cache_patterns = ["dua_search:*", "dua_categories", "dua_stats"];

        for pattern in &cache_patterns {
            if let Err(e) = self.cache.delete(pattern).await {
                warn!("Failed to invalidate cache pattern {}: {}", pattern, e);
            }
        }
    }

    fn create_search_cache_key(&self, query: &SearchDuaQuery) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.q.hash(&mut hasher);
        query.category.hash(&mut hasher);
        query.tags.hash(&mut hasher);
        query.verified.hash(&mut hasher);
        query.page.hash(&mut hasher);
        query.limit.hash(&mut hasher);

        if let Some(ref sort) = query.sort {
            std::mem::discriminant(sort).hash(&mut hasher);
        }

        format!("dua_search:{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let service = DuaService::new(
            DuaRepository::new(shared::database::Database { pool: todo!() }),
            Cache::new(&shared::config::RedisConfig {
                url: "redis://localhost".to_string(),
                pool_max_open: 10,
                pool_max_idle: 5,
                pool_timeout: 30,
                pool_expire: 300,
            })
            .await
            .unwrap(),
        );

        let query1 = SearchDuaQuery {
            q: Some("test".to_string()),
            category: None,
            tags: None,
            verified: None,
            page: Some(1),
            limit: Some(20),
            sort: None,
        };

        let query2 = SearchDuaQuery {
            q: Some("test".to_string()),
            category: None,
            tags: None,
            verified: None,
            page: Some(1),
            limit: Some(20),
            sort: None,
        };

        let key1 = service.create_search_cache_key(&query1);
        let key2 = service.create_search_cache_key(&query2);

        assert_eq!(key1, key2);
    }
}
