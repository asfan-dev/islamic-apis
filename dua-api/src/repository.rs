use shared::{
    database::Database,
    error::{ApiResult},
};
use sqlx::{ QueryBuilder, Row};
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

        let result = sqlx::query_as::<_, Dua>(
            r#"
            INSERT INTO duas (
                id, title, arabic_text, transliteration, translation, 
                reference, category, tags, audio_url, is_verified, 
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(dua.id)
        .bind(dua.title)
        .bind(dua.arabic_text)
        .bind(dua.transliteration)
        .bind(dua.translation)
        .bind(dua.reference)
        .bind(dua.category)
        .bind(&dua.tags)
        .bind(dua.audio_url)
        .bind(dua.is_verified)
        .bind(dua.created_at)
        .bind(dua.updated_at)
        .fetch_one(&self.db.pool)
        .await?;

        info!("Created dua with ID: {}", result.id);
        Ok(result)
    }

    pub async fn get_by_id(&self, id: Uuid) -> ApiResult<Option<Dua>> {
        debug!("Fetching dua by ID: {}", id);

        let result = sqlx::query_as::<_, Dua>("SELECT * FROM duas WHERE id = $1")
            .bind(id)
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

        // Build dynamic update query using QueryBuilder
        let mut query_builder = QueryBuilder::new("UPDATE duas SET ");
        let mut has_updates = false;

        if let Some(title) = update_request.title {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("title = ");
            query_builder.push_bind(title);
            has_updates = true;
        }

        if let Some(arabic_text) = update_request.arabic_text {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("arabic_text = ");
            query_builder.push_bind(arabic_text);
            has_updates = true;
        }

        if let Some(transliteration) = update_request.transliteration {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("transliteration = ");
            query_builder.push_bind(transliteration);
            has_updates = true;
        }

        if let Some(translation) = update_request.translation {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("translation = ");
            query_builder.push_bind(translation);
            has_updates = true;
        }

        if let Some(reference) = update_request.reference {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("reference = ");
            query_builder.push_bind(reference);
            has_updates = true;
        }

        if let Some(category) = update_request.category {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("category = ");
            query_builder.push_bind(category);
            has_updates = true;
        }

        if let Some(tags) = update_request.tags {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("tags = ");
            query_builder.push_bind(tags);
            has_updates = true;
        }

        if let Some(audio_url) = update_request.audio_url {
            if has_updates {
                query_builder.push(", ");
            }
            query_builder.push("audio_url = ");
            query_builder.push_bind(audio_url);
            has_updates = true;
        }

        if !has_updates {
            return self.get_by_id(id).await;
        }

        query_builder.push(", updated_at = NOW() WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let result = query_builder
            .build_query_as::<Dua>()
            .fetch_optional(&self.db.pool)
            .await?;

        if result.is_some() {
            info!("Updated dua with ID: {}", id);
        }

        Ok(result)
    }

    pub async fn delete(&self, id: Uuid) -> ApiResult<bool> {
        debug!("Deleting dua with ID: {}", id);

        let rows_affected = sqlx::query("DELETE FROM duas WHERE id = $1")
            .bind(id)
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

    // Prepare search terms that will live for the entire function
    let search_term = search_query.q.as_ref().map(|q| format!("%{}%", q));
    let category_term = search_query.category.as_ref().map(|cat| format!("%{}%", cat));

    // Build WHERE clause
    if search_term.is_some() {
        where_conditions.push(format!(
            "(title ILIKE ${} OR arabic_text ILIKE ${} OR transliteration ILIKE ${} OR translation ILIKE ${} OR reference ILIKE ${})",
            param_count, param_count, param_count, param_count, param_count
        ));
        param_count += 1;
    }

    if category_term.is_some() {
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
    if let Some(ref term) = search_term {
        count_query = count_query.bind(term);
    }
    if let Some(ref term) = category_term {
        count_query = count_query.bind(term);
    }
    if let Some(ref tags) = search_query.tags {
        count_query = count_query.bind(tags);
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
    if let Some(ref term) = search_term {
        data_query = data_query.bind(term);
    }
    if let Some(ref term) = category_term {
        data_query = data_query.bind(term);
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

        let rows = sqlx::query(
            r#"
            SELECT 
                category as name, 
                COUNT(*) as count
            FROM duas 
            GROUP BY category 
            ORDER BY count DESC, category ASC
            "#,
        )
        .fetch_all(&self.db.pool)
        .await?;

        let categories: Vec<DuaCategory> = rows
            .into_iter()
            .map(|row| DuaCategory {
                name: row.get("name"),
                count: row.get::<i64, _>("count"),
                description: None,
            })
            .collect();

        debug!("Found {} categories", categories.len());
        Ok(categories)
    }

    pub async fn get_stats(&self) -> ApiResult<(i64, i64, i64, Option<String>, i64)> {
        debug!("Fetching dua statistics");

        let total_duas = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM duas")
            .fetch_one(&self.db.pool)
            .await?;

        let verified_duas = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM duas WHERE is_verified = true")
            .fetch_one(&self.db.pool)
            .await?;

        let total_categories = sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT category) FROM duas")
            .fetch_one(&self.db.pool)
            .await?;

        let most_popular_category = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT category 
            FROM duas 
            GROUP BY category 
            ORDER BY COUNT(*) DESC 
            LIMIT 1
            "#,
        )
        .fetch_one(&self.db.pool)
        .await?;

        let recent_additions = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM duas WHERE created_at > NOW() - INTERVAL '30 days'",
        )
        .fetch_one(&self.db.pool)
        .await?;

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

        let result = sqlx::query_as::<_, Dua>(
            "UPDATE duas SET is_verified = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(verified)
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?;

        if result.is_some() {
            info!("Updated verification status for dua with ID: {}", id);
        }

        Ok(result)
    }
}
