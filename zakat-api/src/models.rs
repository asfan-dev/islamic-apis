use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ZakatType {
    Wealth,    // Cash, savings, investments
    Gold,      // Gold jewelry and items
    Silver,    // Silver jewelry and items
    Business,  // Business assets and inventory
    Livestock, // Cattle, sheep, goats, camels
    Crops,     // Agricultural produce
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    SAR,
    AED,
    PKR,
    INR,
    BDT,
    MYR,
    IDR,
    TRY,
    EGP,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ZakatCalculationRequest {
    pub calculation_type: ZakatType,

    #[validate(custom = "validate_amount")]
    pub amount: Decimal,

    pub currency: Currency,

    // Optional fields for different calculation types
    pub gold_weight_grams: Option<Decimal>,
    pub silver_weight_grams: Option<Decimal>,
    pub gold_purity_karats: Option<u8>, // 14, 18, 22, 24

    // Livestock specific
    pub cattle_count: Option<u32>,
    pub sheep_goat_count: Option<u32>,
    pub camel_count: Option<u32>,

    // Business specific
    pub business_assets: Option<Decimal>,
    pub business_liabilities: Option<Decimal>,
    pub inventory_value: Option<Decimal>,

    // Crops specific
    pub crop_type: Option<String>,
    pub irrigation_method: Option<IrrigationMethod>,

    // Optional user info for saving calculation
    pub user_id: Option<String>,
    pub save_calculation: Option<bool>,
}

fn validate_amount(amount: &Decimal) -> Result<(), ValidationError> {
    if *amount < Decimal::ZERO {
        return Err(ValidationError::new("Amount must be non-negative"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IrrigationMethod {
    Natural, // Rain-fed (10% zakat)
    Manual,  // Artificially irrigated (5% zakat)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatCalculationResponse {
    pub calculation_id: Uuid,
    pub calculation_type: ZakatType,
    pub input_amount: Decimal,
    pub currency: Currency,
    pub nisab_threshold: Decimal,
    pub zakat_due: Decimal,
    pub zakat_percentage: Decimal,
    pub is_zakat_applicable: bool,
    pub calculation_details: ZakatDetails,
    pub recommendations: Vec<String>,
    pub islamic_references: Vec<IslamicReference>,
    pub calculation_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZakatDetails {
    Wealth(WealthZakatDetails),
    Gold(MetalZakatDetails),
    Silver(MetalZakatDetails),
    Business(BusinessZakatDetails),
    Livestock(LivestockZakatDetails),
    Crops(CropsZakatDetails),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthZakatDetails {
    pub cash_savings: Decimal,
    pub investments: Decimal,
    pub total_wealth: Decimal,
    pub nisab_equivalent_gold: Decimal,
    pub nisab_equivalent_silver: Decimal,
    pub years_held: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalZakatDetails {
    pub weight_grams: Decimal,
    pub purity_percentage: Decimal,
    pub pure_weight_grams: Decimal,
    pub current_price_per_gram: Decimal,
    pub total_value: Decimal,
    pub nisab_weight_grams: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessZakatDetails {
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub net_assets: Decimal,
    pub inventory_value: Decimal,
    pub cash_equivalents: Decimal,
    pub zakatable_amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivestockZakatDetails {
    pub cattle_count: u32,
    pub sheep_goat_count: u32,
    pub camel_count: u32,
    pub total_animals: u32,
    pub zakat_animals_due: u32,
    pub alternative_cash_value: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropsZakatDetails {
    pub crop_type: String,
    pub harvest_amount: Decimal,
    pub irrigation_method: IrrigationMethod,
    pub zakat_percentage: Decimal,
    pub deductible_expenses: Option<Decimal>,
    pub net_harvest_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslamicReference {
    pub source: String,
    pub reference: String,
    pub arabic_text: Option<String>,
    pub translation: String,
}

// Updated SavedCalculation without FromRow derive - we'll handle conversion manually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedCalculation {
    pub id: Uuid,
    pub user_id: String,
    pub calculation_type: String,
    pub input_data: serde_json::Value,
    pub result_data: serde_json::Value,
    pub zakat_amount: Decimal,
    pub currency: String,
    pub created_at: DateTime<Utc>,
}

// Database representation using strings for Decimal fields
#[derive(Debug, Clone, FromRow)]
pub struct SavedCalculationRow {
    pub id: Uuid,
    pub user_id: String,
    pub calculation_type: String,
    pub input_data: serde_json::Value,
    pub result_data: serde_json::Value,
    pub zakat_amount: String, // Store as string in DB
    pub currency: String,
    pub created_at: DateTime<Utc>,
}

impl From<SavedCalculationRow> for SavedCalculation {
    fn from(row: SavedCalculationRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            calculation_type: row.calculation_type,
            input_data: row.input_data,
            result_data: row.result_data,
            zakat_amount: row.zakat_amount.parse().unwrap_or_default(),
            currency: row.currency,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationHistoryResponse {
    pub calculations: Vec<SavedCalculation>,
    pub total: i64,
    pub summary: CalculationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationSummary {
    pub total_zakat_calculated: Decimal,
    pub most_common_type: Option<ZakatType>,
    pub calculations_this_year: i64,
    pub average_zakat_amount: Decimal,
}

// Database representation for NisabRate
#[derive(Debug, Clone, FromRow)]
pub struct NisabRateRow {
    pub id: Uuid,
    pub metal_type: String,
    pub price_per_gram_usd: String, // Store as string in DB
    pub nisab_grams: String,        // Store as string in DB
    pub nisab_value_usd: String,    // Store as string in DB
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NisabRate {
    pub id: Uuid,
    pub metal_type: String,
    pub price_per_gram_usd: Decimal,
    pub nisab_grams: Decimal,
    pub nisab_value_usd: Decimal,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

impl From<NisabRateRow> for NisabRate {
    fn from(row: NisabRateRow) -> Self {
        Self {
            id: row.id,
            metal_type: row.metal_type,
            price_per_gram_usd: row.price_per_gram_usd.parse().unwrap_or_default(),
            nisab_grams: row.nisab_grams.parse().unwrap_or_default(),
            nisab_value_usd: row.nisab_value_usd.parse().unwrap_or_default(),
            last_updated: row.last_updated,
            source: row.source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NisabRatesResponse {
    pub gold: NisabRate,
    pub silver: NisabRate,
    pub currency_rates: std::collections::HashMap<Currency, Decimal>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ZakatInfoResponse {
    pub types: Vec<ZakatTypeInfo>,
    pub general_info: GeneralZakatInfo,
    pub nisab_info: NisabInfo,
    pub calculation_guidelines: Vec<CalculationGuideline>,
}

#[derive(Debug, Serialize)]
pub struct ZakatTypeInfo {
    pub zakat_type: ZakatType,
    pub title: String,
    pub description: String,
    pub rate_percentage: Decimal,
    pub nisab_criteria: String,
    pub conditions: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GeneralZakatInfo {
    pub definition: String,
    pub importance: String,
    pub who_must_pay: Vec<String>,
    pub who_receives: Vec<String>,
    pub general_conditions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct NisabInfo {
    pub definition: String,
    pub gold_nisab_grams: Decimal,
    pub silver_nisab_grams: Decimal,
    pub purpose: String,
    pub calculation_method: String,
}

#[derive(Debug, Serialize)]
pub struct CalculationGuideline {
    pub title: String,
    pub description: String,
    pub steps: Vec<String>,
    pub important_notes: Vec<String>,
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

impl Currency {
    pub fn to_string(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::SAR => "SAR",
            Currency::AED => "AED",
            Currency::PKR => "PKR",
            Currency::INR => "INR",
            Currency::BDT => "BDT",
            Currency::MYR => "MYR",
            Currency::IDR => "IDR",
            Currency::TRY => "TRY",
            Currency::EGP => "EGP",
        }
    }
}

impl ZakatType {
    pub fn to_string(&self) -> &'static str {
        match self {
            ZakatType::Wealth => "wealth",
            ZakatType::Gold => "gold",
            ZakatType::Silver => "silver",
            ZakatType::Business => "business",
            ZakatType::Livestock => "livestock",
            ZakatType::Crops => "crops",
        }
    }

    pub fn get_standard_rate(&self) -> Decimal {
        match self {
            ZakatType::Wealth | ZakatType::Gold | ZakatType::Silver | ZakatType::Business => {
                Decimal::new(25, 1) // 2.5%
            }
            ZakatType::Livestock => Decimal::new(0, 0), // Variable based on count
            ZakatType::Crops => Decimal::new(5, 0),     // 5% or 10% based on irrigation
        }
    }
}
