use shared::{database::Database, error::ApiResult};
use sqlx::QueryBuilder;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::*;

pub struct DuaRepository {
    db: Database,
}

impl DuaRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // ============= DUA CRUD OPERATIONS =============

    pub async fn create_dua(&self, dua: Dua) -> ApiResult<Dua> {
        debug!("Creating new dua with title: {}", dua.title);

        let result = sqlx::query_as::<_, Dua>(
            r#"
            INSERT INTO duas (
                id, title, arabic_text, transliteration, translation, 
                slug, status, version, popularity_score, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(dua.id)
        .bind(dua.title)
        .bind(dua.arabic_text)
        .bind(dua.transliteration)
        .bind(dua.translation)
        .bind(dua.slug)
        .bind(dua.status)
        .bind(dua.version)
        .bind(dua.popularity_score)
        .bind(dua.created_at)
        .bind(dua.updated_at)
        .fetch_one(&self.db.pool)
        .await?;

        info!("Created dua with ID: {}", result.id);
        Ok(result)
    }

    pub async fn get_dua_by_id(&self, id: Uuid) -> ApiResult<Option<Dua>> {
        debug!("Fetching dua by ID: {}", id);

        let result = sqlx::query_as::<_, Dua>("SELECT * FROM duas WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db.pool)
            .await?;

        Ok(result)
    }

    pub async fn get_dua_by_slug(&self, slug: &str) -> ApiResult<Option<Dua>> {
        debug!("Fetching dua by slug: {}", slug);

        let result = sqlx::query_as::<_, Dua>("SELECT * FROM duas WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&self.db.pool)
            .await?;

        Ok(result)
    }

    // ============= LIST WITH FILTERS =============

    pub async fn list_with_filters(&self, params: &DuaQueryParams) -> ApiResult<(Vec<Dua>, i64)> {
        let mut query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT DISTINCT d.* FROM duas d");
        let mut count_query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT COUNT(DISTINCT d.id) FROM duas d");
        
        // Add JOINs based on filters
        if params.category.is_some() {
            query.push(" JOIN dua_category_map dcm ON d.id = dcm.dua_id");
            query.push(" JOIN dua_categories c ON dcm.category_id = c.id");
            count_query.push(" JOIN dua_category_map dcm ON d.id = dcm.dua_id");
            count_query.push(" JOIN dua_categories c ON dcm.category_id = c.id");
        }
        
        if params.tag.is_some() {
            query.push(" JOIN dua_tag_map dtm ON d.id = dtm.dua_id");
            query.push(" JOIN dua_tags t ON dtm.tag_id = t.id");
            count_query.push(" JOIN dua_tag_map dtm ON d.id = dtm.dua_id");
            count_query.push(" JOIN dua_tags t ON dtm.tag_id = t.id");
        }
        
        if params.invocation_time.is_some() || params.event_trigger.is_some() || 
           params.posture.is_some() || params.hands_raising_rule.is_some() ||
           params.audible_mode.is_some() || params.addressing_mode.is_some() {
            query.push(" JOIN dua_context ctx ON d.id = ctx.dua_id");
            count_query.push(" JOIN dua_context ctx ON d.id = ctx.dua_id");
        }
        
        if params.source_type.is_some() || params.authenticity.is_some() {
            query.push(" JOIN dua_sources s ON d.id = s.dua_id");
            count_query.push(" JOIN dua_sources s ON d.id = s.dua_id");
        }
        
        if params.bundle.is_some() {
            query.push(" JOIN dua_bundle_items dbi ON d.id = dbi.dua_id");
            query.push(" JOIN dua_bundles b ON dbi.bundle_id = b.id");
            count_query.push(" JOIN dua_bundle_items dbi ON d.id = dbi.dua_id");
            count_query.push(" JOIN dua_bundles b ON dbi.bundle_id = b.id");
        }
        
        if params.has_audio.is_some() || params.reciter_style.is_some() {
            query.push(" JOIN dua_media m ON d.id = m.dua_id");
            count_query.push(" JOIN dua_media m ON d.id = m.dua_id");
        }
        
        // Build WHERE clause
        let mut where_conditions = Vec::new();
        
        if let Some(ref q) = params.q {
            where_conditions.push(format!(
                "(d.title ILIKE '%{}%' OR d.arabic_text ILIKE '%{}%' OR d.translation ILIKE '%{}%' OR d.transliteration ILIKE '%{}%')",
                q, q, q, q
            ));
        }
        
        if let Some(ref category) = params.category {
            where_conditions.push(format!("c.slug = '{}'", category));
        }
        
        if let Some(ref tag) = params.tag {
            where_conditions.push(format!("t.slug = '{}'", tag));
        }
        
        if let Some(ref invocation_time) = params.invocation_time {
            where_conditions.push(format!("'{}' = ANY(ctx.invocation_time)", invocation_time));
        }
        
        if let Some(ref event_trigger) = params.event_trigger {
            where_conditions.push(format!("'{}' = ANY(ctx.event_trigger)", event_trigger));
        }
        
        if let Some(ref posture) = params.posture {
            where_conditions.push(format!("'{}' = ANY(ctx.posture)", posture));
        }
        
        if let Some(ref hands_raising) = params.hands_raising_rule {
            where_conditions.push(format!("ctx.hands_raising_rule = '{}'", hands_raising));
        }
        
        if let Some(ref audible) = params.audible_mode {
            where_conditions.push(format!("ctx.audible_mode = '{}'", audible));
        }
        
        if let Some(ref addressing) = params.addressing_mode {
            where_conditions.push(format!("ctx.addressing_mode = '{}'", addressing));
        }
        
        if let Some(ref source_type) = params.source_type {
            where_conditions.push(format!("s.source_type = '{}'", source_type));
        }
        
        if let Some(ref authenticity) = params.authenticity {
            where_conditions.push(format!("s.authenticity = '{}'", authenticity));
        }
        
        if let Some(ref bundle) = params.bundle {
            where_conditions.push(format!("b.slug = '{}'", bundle));
        }
        
        if let Some(ruqyah) = params.ruqyah {
            if ruqyah {
                where_conditions.push("b.is_ruqyah = true".to_string());
            }
        }
        
        if let Some(has_audio) = params.has_audio {
            if has_audio {
                where_conditions.push("m.media_type = 'audio'".to_string());
            }
        }
        
        if let Some(ref reciter_style) = params.reciter_style {
            where_conditions.push(format!("m.reciter_style = '{}'", reciter_style));
        }
        
        if let Some(popularity_min) = params.popularity_min {
            where_conditions.push(format!("d.popularity_score >= {}", popularity_min));
        }
        
        // Apply WHERE clause
        if !where_conditions.is_empty() {
            let where_clause = format!(" WHERE {}", where_conditions.join(" AND "));
            query.push(&where_clause);
            count_query.push(&where_clause);
        }
        
        // Get total count
        let total: i64 = sqlx::query_scalar(&count_query.sql())
            .fetch_one(&self.db.pool)
            .await?;
        
        // Apply sorting
        let sort_field = params.sort.as_deref().unwrap_or("created_at");
        let sort_order = params.order.as_deref().unwrap_or("desc");
        query.push(&format!(" ORDER BY d.{} {}", sort_field, sort_order.to_uppercase()));
        
        // Apply pagination
        let per_page = params.get_per_page();
        let offset = params.get_offset();
        query.push(&format!(" LIMIT {} OFFSET {}", per_page, offset));
        
        // Execute query
        let duas = query
            .build_query_as::<Dua>()
            .fetch_all(&self.db.pool)
            .await?;
        
        Ok((duas, total))
    }

    // ============= RANDOM DUA =============

    pub async fn get_random_dua(&self, params: &DuaQueryParams) -> ApiResult<Option<Dua>> {
        let mut query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT d.* FROM duas d");
        
        // Apply filters (similar to list_with_filters but with RANDOM())
        let mut where_conditions = Vec::new();
        
        if let Some(ref invocation_time) = params.invocation_time {
            query.push(" JOIN dua_context ctx ON d.id = ctx.dua_id");
            where_conditions.push(format!("'{}' = ANY(ctx.invocation_time)", invocation_time));
        }
        
        if let Some(ref event_trigger) = params.event_trigger {
            if !params.invocation_time.is_some() {
                query.push(" JOIN dua_context ctx ON d.id = ctx.dua_id");
            }
            where_conditions.push(format!("'{}' = ANY(ctx.event_trigger)", event_trigger));
        }
        
        if let Some(ref category) = params.category {
            query.push(" JOIN dua_category_map dcm ON d.id = dcm.dua_id");
            query.push(" JOIN dua_categories c ON dcm.category_id = c.id");
            where_conditions.push(format!("c.slug = '{}'", category));
        }
        
        if !where_conditions.is_empty() {
            query.push(&format!(" WHERE {}", where_conditions.join(" AND ")));
        }
        
        query.push(" ORDER BY RANDOM() LIMIT 1");
        
        let dua = query
            .build_query_as::<Dua>()
            .fetch_optional(&self.db.pool)
            .await?;
        
        Ok(dua)
    }

    // ============= RELATIONS LOADERS =============

    pub async fn get_dua_sources(&self, dua_id: Uuid) -> ApiResult<Vec<DuaSource>> {
        let sources = sqlx::query_as::<_, DuaSource>(
            "SELECT * FROM dua_sources WHERE dua_id = $1"
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(sources)
    }

    pub async fn get_dua_context(&self, dua_id: Uuid) -> ApiResult<Option<DuaContext>> {
        let context = sqlx::query_as::<_, DuaContext>(
            "SELECT * FROM dua_context WHERE dua_id = $1"
        )
        .bind(dua_id)
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(context)
    }

    pub async fn get_dua_media(&self, dua_id: Uuid) -> ApiResult<Vec<DuaMedia>> {
        let media = sqlx::query_as::<_, DuaMedia>(
            "SELECT * FROM dua_media WHERE dua_id = $1"
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(media)
    }

    pub async fn get_dua_categories(&self, dua_id: Uuid) -> ApiResult<Vec<DuaCategory>> {
        let categories = sqlx::query_as::<_, DuaCategory>(
            r#"
            SELECT c.* FROM dua_categories c
            JOIN dua_category_map dcm ON c.id = dcm.category_id
            WHERE dcm.dua_id = $1
            ORDER BY c.sort_order
            "#
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(categories)
    }

    pub async fn get_dua_tags(&self, dua_id: Uuid) -> ApiResult<Vec<DuaTag>> {
        let tags = sqlx::query_as::<_, DuaTag>(
            r#"
            SELECT t.* FROM dua_tags t
            JOIN dua_tag_map dtm ON t.id = dtm.tag_id
            WHERE dtm.dua_id = $1
            ORDER BY t.name
            "#
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(tags)
    }

    pub async fn get_dua_translations(&self, dua_id: Uuid) -> ApiResult<Vec<DuaTranslation>> {
        let translations = sqlx::query_as::<_, DuaTranslation>(
            "SELECT * FROM dua_translations WHERE dua_id = $1 ORDER BY language_code"
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(translations)
    }

    pub async fn get_dua_variants(&self, dua_id: Uuid) -> ApiResult<Vec<DuaVariant>> {
        let variants = sqlx::query_as::<_, DuaVariant>(
            "SELECT * FROM dua_variants WHERE dua_id = $1 ORDER BY variant_type"
        )
        .bind(dua_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(variants)
    }

    // ============= CATEGORIES =============

    pub async fn list_categories(&self) -> ApiResult<Vec<DuaCategory>> {
        let categories = sqlx::query_as::<_, DuaCategory>(
            "SELECT * FROM dua_categories ORDER BY sort_order, name"
        )
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(categories)
    }

    pub async fn get_category_by_slug(&self, slug: &str) -> ApiResult<Option<DuaCategory>> {
        let category = sqlx::query_as::<_, DuaCategory>(
            "SELECT * FROM dua_categories WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(category)
    }

    pub async fn get_duas_by_category(&self, category_slug: &str) -> ApiResult<Vec<Dua>> {
        let duas = sqlx::query_as::<_, Dua>(
            r#"
            SELECT d.* FROM duas d
            JOIN dua_category_map dcm ON d.id = dcm.dua_id
            JOIN dua_categories c ON dcm.category_id = c.id
            WHERE c.slug = $1
            ORDER BY d.popularity_score DESC, d.title
            "#
        )
        .bind(category_slug)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(duas)
    }

    // ============= TAGS =============

    pub async fn list_tags(&self) -> ApiResult<Vec<DuaTag>> {
        let tags = sqlx::query_as::<_, DuaTag>(
            "SELECT * FROM dua_tags ORDER BY name"
        )
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(tags)
    }

    pub async fn get_duas_by_tag(&self, tag_slug: &str) -> ApiResult<Vec<Dua>> {
        let duas = sqlx::query_as::<_, Dua>(
            r#"
            SELECT d.* FROM duas d
            JOIN dua_tag_map dtm ON d.id = dtm.dua_id
            JOIN dua_tags t ON dtm.tag_id = t.id
            WHERE t.slug = $1
            ORDER BY d.popularity_score DESC, d.title
            "#
        )
        .bind(tag_slug)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(duas)
    }

    // ============= BUNDLES =============

    pub async fn list_bundles(&self) -> ApiResult<Vec<DuaBundle>> {
        let bundles = sqlx::query_as::<_, DuaBundle>(
            "SELECT * FROM dua_bundles ORDER BY name"
        )
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(bundles)
    }

    pub async fn get_bundle_by_slug(&self, slug: &str) -> ApiResult<Option<DuaBundle>> {
        let bundle = sqlx::query_as::<_, DuaBundle>(
            "SELECT * FROM dua_bundles WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(bundle)
    }

    pub async fn get_bundle_items(&self, bundle_slug: &str) -> ApiResult<Vec<Dua>> {
        let duas = sqlx::query_as::<_, Dua>(
            r#"
            SELECT d.* FROM duas d
            JOIN dua_bundle_items dbi ON d.id = dbi.dua_id
            JOIN dua_bundles b ON dbi.bundle_id = b.id
            WHERE b.slug = $1
            ORDER BY dbi.sort_order
            "#
        )
        .bind(bundle_slug)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(duas)
    }

    // ============= SOURCES =============

    pub async fn list_sources(&self, params: &SourceQueryParams) -> ApiResult<Vec<DuaSource>> {
        let mut query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT * FROM dua_sources WHERE 1=1");
        
        if let Some(ref source_type) = params.source_type {
            query.push(&format!(" AND source_type = '{}'", source_type));
        }
        
        if let Some(ref authenticity) = params.authenticity {
            query.push(&format!(" AND authenticity = '{}'", authenticity));
        }
        
        if let Some(ref q) = params.q {
            query.push(&format!(" AND (book_name ILIKE '%{}%' OR reference_text ILIKE '%{}%')", q, q));
        }
        
        query.push(" ORDER BY created_at DESC");
        
        let sources = query
            .build_query_as::<DuaSource>()
            .fetch_all(&self.db.pool)
            .await?;
        
        Ok(sources)
    }

    pub async fn get_source_by_id(&self, id: Uuid) -> ApiResult<Option<DuaSource>> {
        let source = sqlx::query_as::<_, DuaSource>(
            "SELECT * FROM dua_sources WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?;
        
        Ok(source)
    }

    pub async fn get_duas_by_source(&self, source_id: Uuid) -> ApiResult<Vec<Dua>> {
        let duas = sqlx::query_as::<_, Dua>(
            r#"
            SELECT d.* FROM duas d
            JOIN dua_sources s ON d.id = s.dua_id
            WHERE s.id = $1
            "#
        )
        .bind(source_id)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(duas)
    }

    // ============= MEDIA =============

    pub async fn search_media(&self, params: &MediaQueryParams) -> ApiResult<Vec<DuaMedia>> {
        let mut query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new("SELECT * FROM dua_media WHERE 1=1");
        
        if let Some(ref media_type) = params.media_type {
            query.push(&format!(" AND media_type = '{}'", media_type));
        }
        
        if let Some(ref license) = params.license {
            query.push(&format!(" AND license = '{}'", license));
        }
        
        if let Some(ref reciter) = params.reciter {
            query.push(&format!(" AND reciter_name ILIKE '%{}%'", reciter));
        }
        
        query.push(" ORDER BY created_at DESC");
        
        let media = query
            .build_query_as::<DuaMedia>()
            .fetch_all(&self.db.pool)
            .await?;
        
        Ok(media)
    }

    // ============= SEARCH =============

    pub async fn keyword_search(&self, query: &str, limit: u32) -> ApiResult<Vec<Dua>> {
        let duas = sqlx::query_as::<_, Dua>(
            r#"
            SELECT * FROM duas
            WHERE title ILIKE $1 
               OR arabic_text ILIKE $1
               OR translation ILIKE $1
               OR transliteration ILIKE $1
            ORDER BY popularity_score DESC
            LIMIT $2
            "#
        )
        .bind(format!("%{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(duas)
    }

    pub async fn semantic_search(&self, query: &str, limit: u32, _threshold: f64) -> ApiResult<Vec<Dua>> {
        // This is a placeholder - in production, you'd integrate with a vector database
        // or embedding service for true semantic search
        
        let sql = r#"
            SELECT d.*
            FROM duas d
            WHERE d.title ILIKE $1 
               OR d.translation ILIKE $1
            ORDER BY d.popularity_score DESC
            LIMIT $2
        "#;
        
        let duas = sqlx::query_as::<_, Dua>(sql)
            .bind(format!("%{}%", query))
            .bind(limit as i64)
            .fetch_all(&self.db.pool)
            .await?;
        
        Ok(duas)
    }

    pub async fn get_suggestions(&self, query: &str, limit: u32) -> ApiResult<Vec<String>> {
        let results: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT title FROM duas
            WHERE title ILIKE $1
            UNION
            SELECT DISTINCT name FROM dua_categories
            WHERE name ILIKE $1
            UNION
            SELECT DISTINCT name FROM dua_tags
            WHERE name ILIKE $1
            ORDER BY 1
            LIMIT $2
            "#
        )
        .bind(format!("{}%", query))
        .bind(limit as i64)
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(results.into_iter().map(|(s,)| s).collect())
    }

    // ============= STATISTICS =============

    pub async fn get_stats(&self) -> ApiResult<StatsResponse> {
        let total_duas = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM duas")
            .fetch_one(&self.db.pool)
            .await?;

        let verified_duas = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM duas WHERE status = 'verified'"
        )
        .fetch_one(&self.db.pool)
        .await?;

        let total_categories = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dua_categories"
        )
        .fetch_one(&self.db.pool)
        .await?;

        let total_tags = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dua_tags"
        )
        .fetch_one(&self.db.pool)
        .await?;

        let total_bundles = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dua_bundles"
        )
        .fetch_one(&self.db.pool)
        .await?;

        let most_popular_category = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT c.name 
            FROM dua_categories c
            JOIN dua_category_map dcm ON c.id = dcm.category_id
            GROUP BY c.id, c.name
            ORDER BY COUNT(*) DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?
        .flatten();

        let recent_additions = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM duas WHERE created_at > NOW() - INTERVAL '30 days'"
        )
        .fetch_one(&self.db.pool)
        .await?;

        Ok(StatsResponse {
            total_duas,
            verified_duas,
            total_categories,
            total_tags,
            total_bundles,
            most_popular_category,
            recent_additions,
        })
    }

    // ============= TRANSLATIONS =============

    pub async fn list_all_translations(&self) -> ApiResult<Vec<DuaTranslation>> {
        let translations = sqlx::query_as::<_, DuaTranslation>(
            "SELECT * FROM dua_translations ORDER BY dua_id, language_code"
        )
        .fetch_all(&self.db.pool)
        .await?;
        
        Ok(translations)
    }
}