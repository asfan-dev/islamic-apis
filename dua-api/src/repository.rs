use chrono::Utc;
use shared::{
    database::Database,
    error::{ApiError, ApiResult},
};
use sqlx::{query, query_as, query_scalar};
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::{Dua, DuaCategory, SearchDuaQuery, UpdateDuaRequest};

pub struct DuaRepository {
    db: Database,
}

impl DuaRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, dua: Dua) -> ApiResult<Dua> {
        debug!("Creating new dua with title: {}", dua.title);

        let result = query_as!(
            Dua,
            r#"
            INSERT INTO duas (
                id, title, arabic_text, transliteration, translation, 
                reference, category, tags, audio_url, is_verified, 
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            dua.id,
            dua.title,
            dua.arabic_text,
            dua.transliteration,
            dua.translation,
            dua.reference,
            dua.category,
            &dua.tags,
            dua.audio_url,
            dua.is_verified,
            dua.created_at,
            dua.updated_at
        )
        .fetch_one(&self.db.pool)
        .await?;

        info!("Created dua with ID: {}", result.id);
        Ok(result)
    }

    pub async fn get_by_id(&self, id: Uuid) -> ApiResult<Option<Dua>> {
        debug!("Fetching dua by ID: {}", id);

        let result = query_as!(Dua, "SELECT * FROM duas WHERE id = $1", id)
            .fetch_optional(&self.db.pool)
            .await?;

        Ok(result)
    }

    pub async fn update(
        &self,
        id: Uuid,
        update_request: UpdateDuaRequest,
    ) -> ApiResult<Option<Dua>> {
        debug!("Updating dua with ID: {}", id);

        // Build dynamic update query
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if update_request.title.is_some() {
            query_parts.push(format!("title = ${}", param_count));
            param_count += 1;
        }
        if update_request.arabic_text.is_some() {
            query_parts.push(format!("arabic_text = ${}", param_count));
            param_count += 1;
        }
        if update_request.transliteration.is_some() {
            query_parts.push(format!("transliteration = ${}", param_count));
            param_count += 1;
        }
        if update_request.translation.is_some() {
            query_parts.push(format!("translation = ${}", param_count));
            param_count += 1;
        }
        if update_request.reference.is_some() {
            query_parts.push(format!("reference = ${}", param_count));
            param_count += 1;
        }
        if update_request.category.is_some() {
            query_parts.push(format!("category = ${}", param_count));
            param_count += 1;
        }
        if update_request.tags.is_some() {
            query_parts.push(format!("tags = ${}", param_count));
            param_count += 1;
        }
        if update_request.audio_url.is_some() {
            query_parts.push(format!("audio_url = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            return self.get_by_id(id).await;
        }

        query_parts.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE duas SET {} WHERE id = ${} RETURNING *",
            query_parts.join(", "),
            param_count
        );

        let mut query = sqlx::query_as::<_, Dua>(&sql);

        // Bind parameters in the same order
        if let Some(title) = update_request.title {
            query = query.bind(title);
        }
        if let Some(arabic_text) = update_request.arabic_text {
            query = query.bind(arabic_text);
        }
        if let Some(transliteration) = update_request.transliteration {
            query = query.bind(transliteration);
        }
        if let Some(translation) = update_request.translation {
            query = query.bind(translation);
        }
        if let Some(reference) = update_request.reference {
            query = query.bind(reference);
        }
        if let Some(category) = update_request.category {
            query = query.bind(category);
        }
        if let Some(tags) = update_request.tags {
            query = query.bind(tags);
        }
        if let Some(audio_url) = update_request.audio_url {
            query = query.bind(audio_url);
        }

        query = query.bind(id);

        let result = query.fetch_optional(&self.db.pool).await?;

        if result.is_some() {
            info!("Updated dua with ID: {}", id);
        }

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> ApiResult<bool> {
        debug!("Deleting dua with ID: {}", id);

        let rows_affected = query!("DELETE FROM duas WHERE id = $1", id)
            .execute(&self.db.pool)
            .await?
            .rows_affected();

        let deleted = rows_affected > 0;
        if deleted {
            info!("Deleted dua with ID: {}", id);
        }

        Ok(deleted)
    }

    pub async fn search(&self, search_query: SearchDuaQuery) -> ApiResult<(Vec<Dua>, i64)> {
        debug!("Searching duas with query: {:?}", search_query);

        let mut where_conditions = Vec::new();
        let mut param_count = 1;

        // Build WHERE clause
        if search_query.q.is_some() {
            where_conditions.push(format!(
                "(title ILIKE ${} OR arabic_text ILIKE ${} OR transliteration ILIKE ${} OR translation ILIKE ${} OR reference ILIKE ${})",
                param_count, param_count + 1, param_count + 2, param_count + 3, param_count + 4
            ));
            param_count += 5;
        }

        if search_query.category.is_some() {
            where_conditions.push(format!("category ILIKE ${}", param_count));
            param_count += 1;
        }

        if search_query.tags.is_some() {
            where_conditions.push(format!("${} = ANY(tags)", param_count));
            param_count += 1;
        }

        if let Some(verified) = search_query.verified {
            where_conditions.push(format!("is_verified = ${}", param_count));
            param_count += 1;
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Count query
        let count_sql = format!("SELECT COUNT(*) FROM duas {}", where_clause);

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);

        // Bind count query parameters
        let mut current_param = 1;
        if let Some(ref q) = search_query.q {
            let search_term = format!("%{}%", q);
            for _ in 0..5 {
                count_query = count_query.bind(&search_term);
            }
            current_param += 5;
        }
        if let Some(ref category) = search_query.category {
            count_query = count_query.bind(format!("%{}%", category));
            current_param += 1;
        }
        if let Some(ref tags) = search_query.tags {
            count_query = count_query.bind(tags);
            current_param += 1;
        }
        if let Some(verified) = search_query.verified {
            count_query = count_query.bind(verified);
        }

        let total = count_query.fetch_one(&self.db.pool).await?;

        // Data query
        let data_sql = format!(
            "SELECT * FROM duas {} ORDER BY {} LIMIT ${} OFFSET ${}",
            where_clause,
            search_query.get_sort_order().to_sql(),
            param_count,
            param_count + 1
        );

        let mut data_query = sqlx::query_as::<_, Dua>(&data_sql);

        // Bind data query parameters (same as count query)
        if let Some(ref q) = search_query.q {
            let search_term = format!("%{}%", q);
            for _ in 0..5 {
                data_query = data_query.bind(&search_term);
            }
        }
        if let Some(ref category) = search_query.category {
            data_query = data_query.bind(format!("%{}%", category));
        }
        if let Some(ref tags) = search_query.tags {
            data_query = data_query.bind(tags);
        }
        if let Some(verified) = search_query.verified {
            data_query = data_query.bind(verified);
        }

        // Bind pagination parameters
        data_query = data_query
            .bind(search_query.get_limit() as i64)
            .bind(search_query.get_offset() as i64);

        let duas = data_query.fetch_all(&self.db.pool).await?;

        debug!("Found {} duas out of {} total", duas.len(), total);
        Ok((duas, total))
    }

    pub async fn get_categories(&self) -> ApiResult<Vec<DuaCategory>> {
        debug!("Fetching dua categories");

        let categories = query!(
            r#"
            SELECT 
                category as name, 
                COUNT(*) as count,
                NULL as description
            FROM duas 
            GROUP BY category 
            ORDER BY count DESC, category ASC
            "#
        )
        .fetch_all(&self.db.pool)
        .await?
        .into_iter()
        .map(|row| DuaCategory {
            name: row.name,
            count: row.count.unwrap_or(0),
            description: None, // Can be enhanced later
        })
        .collect();

        debug!("Found {} categories", categories.len());
        Ok(categories)
    }

    pub async fn get_stats(&self) -> ApiResult<(i64, i64, i64, Option<String>, i64)> {
        debug!("Fetching dua statistics");

        // Get total duas
        let total_duas = query_scalar!("SELECT COUNT(*) FROM duas")
            .fetch_one(&self.db.pool)
            .await?
            .unwrap_or(0);

        // Get verified duas
        let verified_duas = query_scalar!("SELECT COUNT(*) FROM duas WHERE is_verified = true")
            .fetch_one(&self.db.pool)
            .await?
            .unwrap_or(0);

        // Get total categories
        let total_categories = query_scalar!("SELECT COUNT(DISTINCT category) FROM duas")
            .fetch_one(&self.db.pool)
            .await?
            .unwrap_or(0);

        // Get most popular category
        let most_popular_category = query_scalar!(
            r#"
            SELECT category 
            FROM duas 
            GROUP BY category 
            ORDER BY COUNT(*) DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db.pool)
        .await?;

        // Get recent additions (last 30 days)
        let recent_additions = query_scalar!(
            "SELECT COUNT(*) FROM duas WHERE created_at > NOW() - INTERVAL '30 days'"
        )
        .fetch_one(&self.db.pool)
        .await?
        .unwrap_or(0);

        Ok((
            total_duas,
            verified_duas,
            total_categories,
            most_popular_category,
            recent_additions,
        ))
    }

    pub async fn verify_dua(&self, id: Uuid, verified: bool) -> ApiResult<Option<Dua>> {
        debug!("Setting verification status for dua {}: {}", id, verified);

        let result = query_as!(
            Dua,
            "UPDATE duas SET is_verified = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
            verified,
            id
        )
        .fetch_optional(&self.db.pool)
        .await?;

        if result.is_some() {
            info!("Updated verification status for dua with ID: {}", id);
        }

        Ok(result)
    }
}
