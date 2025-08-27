use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dua {
    pub id: Uuid,
    pub title: String,
    pub arabic_text: String,
    pub transliteration: Option<String>,
    pub translation: String,
    pub reference: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub audio_url: Option<String>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDuaRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 2000,
        message = "Arabic text must be between 1 and 2000 characters"
    ))]
    pub arabic_text: String,

    #[validate(length(max = 2000, message = "Transliteration cannot exceed 2000 characters"))]
    pub transliteration: Option<String>,

    #[validate(length(
        min = 1,
        max = 3000,
        message = "Translation must be between 1 and 3000 characters"
    ))]
    pub translation: String,

    #[validate(length(max = 500, message = "Reference cannot exceed 500 characters"))]
    pub reference: Option<String>,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Category must be between 1 and 100 characters"
    ))]
    pub category: String,

    pub tags: Option<Vec<String>>,

    #[validate(url(message = "Audio URL must be a valid URL"))]
    pub audio_url: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDuaRequest {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(
        min = 1,
        max = 2000,
        message = "Arabic text must be between 1 and 2000 characters"
    ))]
    pub arabic_text: Option<String>,

    #[validate(length(max = 2000, message = "Transliteration cannot exceed 2000 characters"))]
    pub transliteration: Option<String>,

    #[validate(length(
        min = 1,
        max = 3000,
        message = "Translation must be between 1 and 3000 characters"
    ))]
    pub translation: Option<String>,

    #[validate(length(max = 500, message = "Reference cannot exceed 500 characters"))]
    pub reference: Option<String>,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Category must be between 1 and 100 characters"
    ))]
    pub category: Option<String>,

    pub tags: Option<Vec<String>>,

    #[validate(url(message = "Audio URL must be a valid URL"))]
    pub audio_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchDuaQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub verified: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<DuaSortOrder>,
}

#[derive(Debug, Deserialize)]
#[derive(Clone)]
#[serde(rename_all = "lowercase")]
pub enum DuaSortOrder {
    CreatedAsc,
    CreatedDesc,
    TitleAsc,
    TitleDesc,
    CategoryAsc,
    CategoryDesc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuaListResponse {
    pub duas: Vec<Dua>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
pub struct DuaResponse {
    pub dua: Dua,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuaCategoriesResponse {
    pub categories: Vec<DuaCategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuaCategory {
    pub name: String,
    pub count: i64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuaStatsResponse {
    pub total_duas: i64,
    pub verified_duas: i64,
    pub total_categories: i64,
    pub most_popular_category: Option<String>,
    pub recent_additions: i64,
}

impl Default for DuaSortOrder {
    fn default() -> Self {
        DuaSortOrder::CreatedDesc
    }
}

impl DuaSortOrder {
    pub fn to_sql(&self) -> &'static str {
        match self {
            DuaSortOrder::CreatedAsc => "created_at ASC",
            DuaSortOrder::CreatedDesc => "created_at DESC",
            DuaSortOrder::TitleAsc => "title ASC",
            DuaSortOrder::TitleDesc => "title DESC",
            DuaSortOrder::CategoryAsc => "category ASC",
            DuaSortOrder::CategoryDesc => "category DESC",
        }
    }
}

impl CreateDuaRequest {
    pub fn into_dua(self) -> Dua {
        let now = Utc::now();
        Dua {
            id: Uuid::new_v4(),
            title: self.title,
            arabic_text: self.arabic_text,
            transliteration: self.transliteration,
            translation: self.translation,
            reference: self.reference,
            category: self.category,
            tags: self.tags.unwrap_or_default(),
            audio_url: self.audio_url,
            is_verified: false, // New duas start as unverified
            created_at: now,
            updated_at: now,
        }
    }
}

impl SearchDuaQuery {
    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_limit(&self) -> u32 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn get_offset(&self) -> u32 {
        (self.get_page() - 1) * self.get_limit()
    }

    pub fn get_sort_order(&self) -> DuaSortOrder {
        self.sort.clone().unwrap_or_default()
    }
}