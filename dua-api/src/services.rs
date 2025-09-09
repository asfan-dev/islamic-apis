use shared::{cache::Cache, error::ApiResult};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    models::*,
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

    // ============= DUA OPERATIONS =============

    pub async fn list_duas_with_filters(&self, params: DuaQueryParams) -> ApiResult<DuaListResponse> {
        let cache_key = self.create_search_cache_key(&params);
        
        // Try cache for simple queries
        if params.q.is_none() && params.category.is_none() && params.tag.is_none() {
            if let Ok(Some(cached_response)) = self.cache.get::<DuaListResponse>(&cache_key).await {
                debug!("Returning cached dua list");
                return Ok(cached_response);
            }
        }
        
        // Fetch from database
        let (duas, total) = self.repository.list_with_filters(&params).await?;
        
        // Load relations if requested
        let mut duas_with_relations = Vec::new();
        for dua in duas {
            let dua_with_relations = self.load_dua_relations(dua, params.include.as_deref()).await?;
            duas_with_relations.push(dua_with_relations);
        }
        
        let total_pages = ((total as f64) / (params.get_per_page() as f64)).ceil() as u32;
        
        let response = DuaListResponse {
            duas: duas_with_relations,
            total,
            page: params.get_page(),
            per_page: params.get_per_page(),
            total_pages,
        };
        
        // Cache simple queries for 10 minutes
        if params.q.is_none() && params.category.is_none() && params.tag.is_none() {
            if let Err(e) = self.cache.set(&cache_key, &response, Some(Duration::from_secs(600))).await {
                warn!("Failed to cache dua list: {}", e);
            }
        }
        
        Ok(response)
    }

    pub async fn get_dua_by_id(&self, id: Uuid, include: Option<String>) -> ApiResult<Option<DuaWithRelations>> {
        let cache_key = format!("dua:{}", id);
        
        // Try cache first
        if include.is_none() {
            if let Ok(Some(cached_dua)) = self.cache.get::<DuaWithRelations>(&cache_key).await {
                debug!("Returning cached dua for ID: {}", id);
                return Ok(Some(cached_dua));
            }
        }
        
        // Fetch from database
        if let Some(dua) = self.repository.get_dua_by_id(id).await? {
            let dua_with_relations = self.load_dua_relations(dua, include.as_deref()).await?;
            
            // Cache for 1 hour if no includes
            if include.is_none() {
                if let Err(e) = self.cache.set(&cache_key, &dua_with_relations, Some(Duration::from_secs(3600))).await {
                    warn!("Failed to cache dua {}: {}", id, e);
                }
            }
            
            Ok(Some(dua_with_relations))
        } else {
            Ok(None)
        }
    }

    pub async fn get_dua_by_slug(&self, slug: &str, include: Option<String>) -> ApiResult<Option<DuaWithRelations>> {
        if let Some(dua) = self.repository.get_dua_by_slug(slug).await? {
            let dua_with_relations = self.load_dua_relations(dua, include.as_deref()).await?;
            Ok(Some(dua_with_relations))
        } else {
            Ok(None)
        }
    }

    pub async fn get_random_dua(&self, params: DuaQueryParams) -> ApiResult<Option<DuaWithRelations>> {
        if let Some(dua) = self.repository.get_random_dua(&params).await? {
            let dua_with_relations = self.load_dua_relations(dua, params.include.as_deref()).await?;
            Ok(Some(dua_with_relations))
        } else {
            Ok(None)
        }
    }

    async fn load_dua_relations(&self, dua: Dua, include: Option<&str>) -> ApiResult<DuaWithRelations> {
        let mut dua_with_relations = DuaWithRelations {
            dua: dua.clone(),
            sources: None,
            context: None,
            media: None,
            categories: None,
            tags: None,
            translations: None,
            variants: None,
        };
        
        if let Some(include) = include {
            let includes: Vec<&str> = include.split(',').collect();
            
            if includes.contains(&"sources") {
                dua_with_relations.sources = Some(self.repository.get_dua_sources(dua.id).await?);
            }
            
            if includes.contains(&"context") {
                dua_with_relations.context = self.repository.get_dua_context(dua.id).await?;
            }
            
            if includes.contains(&"media") {
                dua_with_relations.media = Some(self.repository.get_dua_media(dua.id).await?);
            }
            
            if includes.contains(&"categories") {
                dua_with_relations.categories = Some(self.repository.get_dua_categories(dua.id).await?);
            }
            
            if includes.contains(&"tags") {
                dua_with_relations.tags = Some(self.repository.get_dua_tags(dua.id).await?);
            }
            
            if includes.contains(&"translations") {
                dua_with_relations.translations = Some(self.repository.get_dua_translations(dua.id).await?);
            }
            
            if includes.contains(&"variants") {
                dua_with_relations.variants = Some(self.repository.get_dua_variants(dua.id).await?);
            }
        }
        
        Ok(dua_with_relations)
    }

    // ============= CATEGORIES =============

    pub async fn list_categories(&self) -> ApiResult<CategoryListResponse> {
        let cache_key = "categories:all";
        
        // Try cache first
        if let Ok(Some(cached_categories)) = self.cache.get::<CategoryListResponse>(&cache_key).await {
            debug!("Returning cached categories");
            return Ok(cached_categories);
        }
        
        // Fetch from database
        let categories = self.repository.list_categories().await?;
        let total = categories.len() as i64;
        
        let response = CategoryListResponse {
            categories,
            total,
        };
        
        // Cache for 30 minutes
        if let Err(e) = self.cache.set(&cache_key, &response, Some(Duration::from_secs(1800))).await {
            warn!("Failed to cache categories: {}", e);
        }
        
        Ok(response)
    }

    // ============= TAGS =============

    pub async fn list_tags(&self) -> ApiResult<TagListResponse> {
        let cache_key = "tags:all";
        
        // Try cache first
        if let Ok(Some(cached_tags)) = self.cache.get::<TagListResponse>(&cache_key).await {
            debug!("Returning cached tags");
            return Ok(cached_tags);
        }
        
        // Fetch from database
        let tags = self.repository.list_tags().await?;
        let total = tags.len() as i64;
        
        let response = TagListResponse {
            tags,
            total,
        };
        
        // Cache for 30 minutes
        if let Err(e) = self.cache.set(&cache_key, &response, Some(Duration::from_secs(1800))).await {
            warn!("Failed to cache tags: {}", e);
        }
        
        Ok(response)
    }

    // ============= BUNDLES =============

    pub async fn list_bundles(&self) -> ApiResult<BundleListResponse> {
        let cache_key = "bundles:all";
        
        // Try cache first
        if let Ok(Some(cached_bundles)) = self.cache.get::<BundleListResponse>(&cache_key).await {
            debug!("Returning cached bundles");
            return Ok(cached_bundles);
        }
        
        // Fetch from database
        let bundles = self.repository.list_bundles().await?;
        let total = bundles.len() as i64;
        
        let response = BundleListResponse {
            bundles,
            total,
        };
        
        // Cache for 30 minutes
        if let Err(e) = self.cache.set(&cache_key, &response, Some(Duration::from_secs(1800))).await {
            warn!("Failed to cache bundles: {}", e);
        }
        
        Ok(response)
    }

    pub async fn get_bundle_items(&self, bundle_slug: &str) -> ApiResult<BundleItemsResponse> {
        let cache_key = format!("bundle:{}:items", bundle_slug);
        
        // Try cache first
        if let Ok(Some(cached_response)) = self.cache.get::<BundleItemsResponse>(&cache_key).await {
            debug!("Returning cached bundle items for: {}", bundle_slug);
            return Ok(cached_response);
        }
        
        // Fetch bundle and its items
        let bundle = self.repository.get_bundle_by_slug(bundle_slug).await?
            .ok_or_else(|| shared::error::ApiError::NotFound(format!("Bundle {} not found", bundle_slug)))?;
        
        let duas = self.repository.get_bundle_items(bundle_slug).await?;
        
        // Load full dua relations
        let mut items = Vec::new();
        for dua in duas {
            let dua_with_relations = self.load_dua_relations(dua, None).await?;
            items.push(dua_with_relations);
        }
        
        let total = items.len() as i64;
        
        let response = BundleItemsResponse {
            bundle,
            items,
            total,
        };
        
        // Cache for 30 minutes
        if let Err(e) = self.cache.set(&cache_key, &response, Some(Duration::from_secs(1800))).await {
            warn!("Failed to cache bundle items: {}", e);
        }
        
        Ok(response)
    }

    // ============= SEARCH =============

    pub async fn keyword_search(&self, query: &str, limit: u32) -> ApiResult<Vec<DuaWithRelations>> {
        let duas = self.repository.keyword_search(query, limit).await?;
        
        let mut results = Vec::new();
        for dua in duas {
            let dua_with_relations = self.load_dua_relations(dua, None).await?;
            results.push(dua_with_relations);
        }
        
        Ok(results)
    }

    pub async fn semantic_search(&self, request: SemanticSearchRequest) -> ApiResult<SearchResponse> {
        let limit = request.limit.unwrap_or(20);
        let threshold = request.threshold.unwrap_or(0.5);
        
        let duas = self.repository.semantic_search(&request.query, limit, threshold).await?;
        
        let mut results = Vec::new();
        for dua in duas {
            let dua_with_relations = self.load_dua_relations(dua, None).await?;
            results.push(dua_with_relations);
        }
        
        let total = results.len() as i64;
        
        Ok(SearchResponse {
            results,
            total,
            query: request.query,
        })
    }

    // ============= STATISTICS =============

    pub async fn get_stats(&self) -> ApiResult<StatsResponse> {
        let cache_key = "stats:global";
        
        // Try cache first
        if let Ok(Some(cached_stats)) = self.cache.get::<StatsResponse>(&cache_key).await {
            debug!("Returning cached stats");
            return Ok(cached_stats);
        }
        
        // Fetch from database
        let stats = self.repository.get_stats().await?;
        
        // Cache for 15 minutes
        if let Err(e) = self.cache.set(&cache_key, &stats, Some(Duration::from_secs(900))).await {
            warn!("Failed to cache stats: {}", e);
        }
        
        Ok(stats)
    }

    // ============= CACHE MANAGEMENT =============

    async fn invalidate_list_caches(&self) {
        let cache_patterns = [
            "dua_search:*",
            "categories:all",
            "tags:all",
            "bundles:all",
            "stats:global",
        ];
        
        for pattern in &cache_patterns {
            if let Err(e) = self.cache.delete(pattern).await {
                warn!("Failed to invalidate cache pattern {}: {}", pattern, e);
            }
        }
    }

    fn create_search_cache_key(&self, query: &DuaQueryParams) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        query.q.hash(&mut hasher);
        query.category.hash(&mut hasher);
        query.tag.hash(&mut hasher);
        query.invocation_time.hash(&mut hasher);
        query.event_trigger.hash(&mut hasher);
        query.source_type.hash(&mut hasher);
        query.authenticity.hash(&mut hasher);
        query.page.hash(&mut hasher);
        query.per_page.hash(&mut hasher);
        query.sort.hash(&mut hasher);
        query.order.hash(&mut hasher);
        
        format!("dua_search:{:x}", hasher.finish())
    }
}