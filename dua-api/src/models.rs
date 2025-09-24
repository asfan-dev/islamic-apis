use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use serde::de::{self, Deserializer};


fn deserialize_bool_from_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        String(String),
    }

    match Option::<BoolOrString>::deserialize(deserializer)? {
        Some(BoolOrString::Bool(b)) => Ok(Some(b)),
        Some(BoolOrString::String(s)) => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(Some(true)),
            "false" | "0" | "no" => Ok(Some(false)),
            _ => Err(de::Error::custom(format!("Invalid boolean value: {}", s))),
        },
        None => Ok(None),
    }
}

// ============= ENUMS =============

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "source_type_enum", rename_all = "PascalCase")]
pub enum SourceType {
    Quran,
    Hadith,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "authenticity_enum", rename_all = "PascalCase")]
pub enum Authenticity {
    Quranic,
    Sahih,
    Hasan,
    Daif,
    Unclassified,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "invocation_time_enum", rename_all = "snake_case")]
pub enum InvocationTime {
    Morning,
    Evening,
    AfterSalah,
    BeforeSleep,
    AfterWaking,
    BeforeWudu,
    AfterWudu,
    EnteringHome,
    LeavingHome,
    EnteringMasjid,
    LeavingMasjid,
    Anytime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_trigger_enum", rename_all = "snake_case")]
pub enum EventTrigger {
    WakingUp,
    Dressing,
    EatingStart,
    EatingEnd,
    TravelStart,
    Rain,
    Thunder,
    Grief,
    Anxiety,
    Illness,
    Istikharah,
    Funeral,
    VisitingSick,
    Protection,
    Repentance,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "posture_enum", rename_all = "snake_case")]
pub enum Posture {
    Standing,
    Sitting,
    Sujud,
    AfterSalam,
    QunutWitr,
    Khutbah,
    Tawaf,
    Sai,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "hands_raising_enum", rename_all = "snake_case")]
pub enum HandsRaising {
    Raise,
    DontRaise,
    ContextDependent,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audible_mode_enum", rename_all = "snake_case")]
pub enum AudibleMode {
    Silent,
    Soft,
    Aloud,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "addressing_mode_enum", rename_all = "snake_case")]
pub enum AddressingMode {
    SingularFirstPerson,
    PluralFirstPerson,
    ThirdPerson,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "media_type_enum", rename_all = "snake_case")]
pub enum MediaType {
    Audio,
    Video,
    Image,
    Svg,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reciter_style_enum", rename_all = "snake_case")]
pub enum ReciterStyle {
    Mujawwad,
    Murattal,
    Spoken,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "license_enum")]
pub enum License {
    #[sqlx(rename = "CC0")]
    CC0,
    #[sqlx(rename = "CC-BY")]
    CCBY,
    #[sqlx(rename = "Public Domain")]
    PublicDomain,
    #[sqlx(rename = "All Rights Reserved")]
    AllRightsReserved,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "review_status_enum", rename_all = "snake_case")]
pub enum ReviewStatus {
    Unreviewed,
    Reviewed,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "relation_type_enum", rename_all = "snake_case")]
pub enum RelationType {
    Related,
    SeeAlso,
    Replaces,
    Contradicts,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status_enum", rename_all = "snake_case")]
pub enum Status {
    Active,
    Draft,
    Deprecated,
}

// ============= CORE MODELS =============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dua {
    pub id: Uuid,
    pub title: String,
    pub arabic_text: String,
    pub transliteration: Option<String>,
    pub translation: String,
    pub slug: String,
    pub status: String,
    pub version: i32,
    pub popularity_score: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaTranslation {
    pub id: Uuid,
    pub dua_id: Uuid,
    pub language_code: String,
    pub title: Option<String>,
    pub translation: Option<String>,
    pub transliteration: Option<String>,
    pub slug: Option<String>,
    pub seo_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaTag {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaVariant {
    pub id: Uuid,
    pub dua_id: Uuid,
    pub variant_type: String,
    pub arabic_text: Option<String>,
    pub transliteration: Option<String>,
    pub translation: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaSource {
    pub id: Uuid,
    pub dua_id: Uuid,
    pub source_type: SourceType,
    pub reference_text: Option<String>,
    pub book_name: Option<String>,
    pub chapter: Option<String>,
    pub hadith_number: Option<String>,
    pub authenticity: Authenticity,
    pub takhrij: Option<String>,
    pub isnad: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaContext {
    pub id: Uuid,
    pub dua_id: Uuid,
    pub invocation_time: Vec<String>,
    pub event_trigger: Vec<String>,
    pub posture: Vec<String>,
    pub repetition_count: Option<i32>,
    pub hands_raising_rule: Option<String>,
    pub audible_mode: Option<String>,
    pub addressing_mode: Option<String>,
    pub etiquette_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaMedia {
    pub id: Uuid,
    pub dua_id: Uuid,
    pub media_type: String,
    pub url: String,
    pub file_path: Option<String>,
    pub file_size: Option<i32>,
    pub duration: Option<i32>,
    pub reciter_name: Option<String>,
    pub reciter_style: Option<String>,
    pub language_code: Option<String>,
    pub license: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaBundle {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub bundle_type: Option<String>,
    pub is_ruqyah: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaBundleItem {
    pub id: Uuid,
    pub bundle_id: Uuid,
    pub dua_id: Uuid,
    pub sort_order: i32,
    pub repetitions: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DuaRelation {
    pub id: Uuid,
    pub source_dua_id: Uuid,
    pub target_dua_id: Uuid,
    pub relation_type: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============= COMPOSITE MODELS =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuaWithRelations {
    #[serde(flatten)]
    pub dua: Dua,
    pub sources: Option<Vec<DuaSource>>,
    pub context: Option<DuaContext>,
    pub media: Option<Vec<DuaMedia>>,
    pub categories: Option<Vec<DuaCategory>>,
    pub tags: Option<Vec<DuaTag>>,
    pub translations: Option<Vec<DuaTranslation>>,
    pub variants: Option<Vec<DuaVariant>>,
}

// ============= REQUEST/RESPONSE MODELS =============

#[derive(Debug, Deserialize, Clone)]
pub struct DuaQueryParams {
    // Pagination
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    
    // Sorting
    pub sort: Option<String>,
    pub order: Option<String>,
    
    // Field selection
    pub fields: Option<String>,
    pub include: Option<String>,
    
    // Language
    pub lang: Option<String>,
    
    // Search
    pub q: Option<String>,
    pub translit: Option<bool>,
    pub phonetic: Option<bool>,
    
    // Filters
    pub invocation_time: Option<String>,
    pub event_trigger: Option<String>,
    pub dua_type: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub source_type: Option<String>,
    pub authenticity: Option<String>,
    pub posture: Option<String>,
    pub hands_raising_rule: Option<String>,
    pub audible_mode: Option<String>,
    pub addressing_mode: Option<String>,
    pub repetitions: Option<String>,
    pub calendar: Option<String>,
    pub bundle: Option<String>,
    
    #[serde(default, deserialize_with = "deserialize_bool_from_string")]
    pub ruqyah: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_bool_from_string")]
    pub has_audio: Option<bool>,

    pub reciter_style: Option<String>,
    pub popularity_min: Option<f64>,
}

impl DuaQueryParams {
    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).min(100).max(1)
    }

    pub fn get_offset(&self) -> u32 {
        (self.get_page() - 1) * self.get_per_page()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuaListResponse {
    pub duas: Vec<DuaWithRelations>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryListResponse {
    pub categories: Vec<DuaCategory>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagListResponse {
    pub tags: Vec<DuaTag>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleListResponse {
    pub bundles: Vec<DuaBundle>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleItemsResponse {
    pub bundle: DuaBundle,
    pub items: Vec<DuaWithRelations>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceListResponse {
    pub sources: Vec<DuaSource>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaListResponse {
    pub media: Vec<DuaMedia>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SemanticSearchRequest {
    #[validate(length(min = 1, max = 500))]
    pub query: String,
    pub limit: Option<u32>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<DuaWithRelations>,
    pub total: i64,
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestResponse {
    pub query: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_duas: i64,
    pub verified_duas: i64,
    pub total_categories: i64,
    pub total_tags: i64,
    pub total_bundles: i64,
    pub most_popular_category: Option<String>,
    pub recent_additions: i64,
}

// Media search parameters
#[derive(Debug, Deserialize)]
pub struct MediaQueryParams {
    pub media_type: Option<String>,
    pub license: Option<String>,
    pub reciter: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// Source search parameters
#[derive(Debug, Deserialize)]
pub struct SourceQueryParams {
    pub source_type: Option<String>,
    pub authenticity: Option<String>,
    pub q: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}