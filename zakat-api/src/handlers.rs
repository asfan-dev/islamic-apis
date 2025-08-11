use axum::{Extension, Json};
use shared::{cache::Cache, database::Database, error::ApiResult};
use tracing::info;
use validator::Validate;

use crate::{
    calculations::ZakatCalculator,
    models::{ZakatCalculationRequest, ZakatInfoResponse},
    repository::ZakatRepository,
    services::ZakatService,
};

pub async fn calculate_zakat(
    Json(request): Json<ZakatCalculationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Calculating zakat for type: {:?}", request.calculation_type);

    // Validate request - ValidationErrors automatically converts to ApiError
    request.validate()?;

    let calculator = ZakatCalculator::new();
    let response = calculator.calculate_zakat(request).await?;

    Ok(Json(serde_json::to_value(response)?))
}

pub async fn save_calculation(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    Json(request): Json<ZakatCalculationRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Saving zakat calculation for user: {:?}", request.user_id);

    // Validate request
    request.validate()?;

    let repository = ZakatRepository::new(database);
    let service = ZakatService::new(repository, cache);

    // Calculate first
    let calculator = ZakatCalculator::new();
    let calculation_result = calculator.calculate_zakat(request.clone()).await?;

    // Save if user_id is provided - fix partial move by using reference
    if let Some(ref user_id) = request.user_id {
        let saved_calculation = service
            .save_calculation(user_id.clone(), request, calculation_result.clone())
            .await?;

        Ok(Json(serde_json::json!({
            "calculation": calculation_result,
            "saved": saved_calculation
        })))
    } else {
        Ok(Json(serde_json::to_value(calculation_result)?))
    }
}

pub async fn get_calculation_history(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = params
        .get("user_id")
        .ok_or_else(|| shared::error::ApiError::invalid_input("user_id parameter is required"))?;

    info!("Fetching calculation history for user: {}", user_id);

    let repository = ZakatRepository::new(database);
    let service = ZakatService::new(repository, cache);

    let response = service.get_calculation_history(user_id).await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_nisab_rates(
    Extension(database): Extension<Database>,
    Extension(cache): Extension<Cache>,
) -> ApiResult<Json<serde_json::Value>> {
    info!("Fetching current nisab rates");

    let repository = ZakatRepository::new(database);
    let service = ZakatService::new(repository, cache);

    let response = service.get_nisab_rates().await?;
    Ok(Json(serde_json::to_value(response)?))
}

pub async fn get_zakat_info() -> ApiResult<Json<ZakatInfoResponse>> {
    info!("Fetching zakat information");

    let info = create_zakat_info_response();
    Ok(Json(info))
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

fn create_zakat_info_response() -> ZakatInfoResponse {
    use crate::models::*;
    use rust_decimal_macros::dec;

    let types = vec![
        ZakatTypeInfo {
            zakat_type: ZakatType::Wealth,
            title: "Wealth Zakat".to_string(),
            description: "Zakat on cash, savings, and liquid investments".to_string(),
            rate_percentage: dec!(2.5),
            nisab_criteria: "Equivalent to 85g of gold or 595g of silver (whichever is lower)"
                .to_string(),
            conditions: vec![
                "Must be held for one full lunar year (Hawl)".to_string(),
                "Must exceed the nisab threshold".to_string(),
                "Must be legitimately owned".to_string(),
            ],
            examples: vec![
                "Bank savings exceeding nisab for 1 year".to_string(),
                "Cash reserves above nisab threshold".to_string(),
                "Investment portfolios (stocks, bonds, etc.)".to_string(),
            ],
        },
        ZakatTypeInfo {
            zakat_type: ZakatType::Gold,
            title: "Gold Zakat".to_string(),
            description: "Zakat on gold jewelry and items".to_string(),
            rate_percentage: dec!(2.5),
            nisab_criteria: "85 grams of pure gold".to_string(),
            conditions: vec![
                "Must be held for one full lunar year".to_string(),
                "Based on pure gold content".to_string(),
                "Some scholars exempt jewelry worn regularly".to_string(),
            ],
            examples: vec![
                "Gold jewelry above 85g pure gold".to_string(),
                "Gold coins and bars".to_string(),
                "Gold ornaments and decorative items".to_string(),
            ],
        },
        ZakatTypeInfo {
            zakat_type: ZakatType::Silver,
            title: "Silver Zakat".to_string(),
            description: "Zakat on silver jewelry and items".to_string(),
            rate_percentage: dec!(2.5),
            nisab_criteria: "595 grams of pure silver".to_string(),
            conditions: vec![
                "Must be held for one full lunar year".to_string(),
                "Based on pure silver content".to_string(),
                "Similar rulings to gold apply".to_string(),
            ],
            examples: vec![
                "Silver jewelry above 595g pure silver".to_string(),
                "Silver coins and bars".to_string(),
                "Silver decorative items".to_string(),
            ],
        },
        ZakatTypeInfo {
            zakat_type: ZakatType::Business,
            title: "Business Zakat".to_string(),
            description: "Zakat on business assets and inventory".to_string(),
            rate_percentage: dec!(2.5),
            nisab_criteria: "Net business assets equivalent to nisab".to_string(),
            conditions: vec![
                "Must be engaged in trade".to_string(),
                "Calculate annually on lunar year-end".to_string(),
                "Include inventory, cash, receivables".to_string(),
                "Deduct business liabilities".to_string(),
            ],
            examples: vec![
                "Retail store inventory".to_string(),
                "Manufacturing business assets".to_string(),
                "Trading company portfolios".to_string(),
            ],
        },
        ZakatTypeInfo {
            zakat_type: ZakatType::Livestock,
            title: "Livestock Zakat".to_string(),
            description: "Zakat on grazing animals".to_string(),
            rate_percentage: dec!(0.0),
            nisab_criteria: "Varies by animal type".to_string(),
            conditions: vec![
                "Animals must graze freely most of the year".to_string(),
                "Specific thresholds for each animal type".to_string(),
                "Usually paid in animals, not cash".to_string(),
            ],
            examples: vec![
                "40+ sheep or goats: 1 animal".to_string(),
                "30+ cattle: 1 calf".to_string(),
                "5+ camels: 1 sheep".to_string(),
            ],
        },
        ZakatTypeInfo {
            zakat_type: ZakatType::Crops,
            title: "Agricultural Zakat".to_string(),
            description: "Zakat on agricultural produce".to_string(),
            rate_percentage: dec!(10.0),
            nisab_criteria: "5 Awsuq (approximately 653 kg)".to_string(),
            conditions: vec![
                "Paid at harvest time".to_string(),
                "Rate depends on irrigation method".to_string(),
                "Applies to staple crops mainly".to_string(),
            ],
            examples: vec![
                "Rain-fed crops: 10% rate".to_string(),
                "Irrigated crops: 5% rate".to_string(),
                "Wheat, rice, dates, raisins".to_string(),
            ],
        },
    ];

    let general_info = GeneralZakatInfo {
        definition: "Zakat is one of the Five Pillars of Islam, a mandatory charitable contribution for qualifying Muslims.".to_string(),
        importance: "Zakat purifies wealth, helps the needy, and is a fundamental act of worship in Islam.".to_string(),
        who_must_pay: vec![
            "Adult Muslims who meet the criteria".to_string(),
            "Must possess nisab amount for one lunar year".to_string(),
            "Must be of sound mind".to_string(),
            "Must be financially stable".to_string(),
        ],
        who_receives: vec![
            "The poor and needy".to_string(),
            "Those employed to collect zakat".to_string(),
            "Those whose hearts are to be reconciled".to_string(),
            "To free slaves and help debtors".to_string(),
            "In the cause of Allah".to_string(),
            "For travelers in need".to_string(),
        ],
        general_conditions: vec![
            "Complete ownership of wealth".to_string(),
            "Wealth must exceed basic needs".to_string(),
            "Must be held for one lunar year".to_string(),
            "Must exceed nisab threshold".to_string(),
        ],
    };

    let nisab_info = NisabInfo {
        definition: "Nisab is the minimum threshold of wealth that makes Zakat obligatory".to_string(),
        gold_nisab_grams: dec!(85.0),
        silver_nisab_grams: dec!(595.0),
        purpose: "Ensures only those with sufficient wealth pay Zakat, protecting the poor from this obligation".to_string(),
        calculation_method: "Based on the value of 85g of gold or 595g of silver, whichever is lower".to_string(),
    };

    let calculation_guidelines = vec![
        CalculationGuideline {
            title: "Wealth Calculation".to_string(),
            description: "How to calculate Zakat on cash and liquid assets".to_string(),
            steps: vec![
                "Calculate total liquid wealth (cash, savings, investments)".to_string(),
                "Check if amount exceeds nisab threshold".to_string(),
                "Confirm wealth has been held for one lunar year".to_string(),
                "Calculate 2.5% of the total amount".to_string(),
            ],
            important_notes: vec![
                "Use current nisab rates for accurate calculation".to_string(),
                "Include all forms of liquid wealth".to_string(),
                "Deduct legitimate debts if applicable".to_string(),
            ],
        },
        CalculationGuideline {
            title: "Gold/Silver Calculation".to_string(),
            description: "How to calculate Zakat on precious metals".to_string(),
            steps: vec![
                "Determine total weight of gold/silver".to_string(),
                "Calculate pure metal content based on purity".to_string(),
                "Check if pure weight exceeds nisab".to_string(),
                "Calculate 2.5% of total value".to_string(),
            ],
            important_notes: vec![
                "Consider purity (karats for gold)".to_string(),
                "Some scholars exempt regularly worn jewelry".to_string(),
                "Use current market prices".to_string(),
            ],
        },
        CalculationGuideline {
            title: "Business Calculation".to_string(),
            description: "How to calculate Zakat on business assets".to_string(),
            steps: vec![
                "List all business assets (inventory, cash, receivables)".to_string(),
                "Calculate total business liabilities".to_string(),
                "Determine net zakatable business wealth".to_string(),
                "Calculate 2.5% if above nisab".to_string(),
            ],
            important_notes: vec![
                "Include all trade inventory".to_string(),
                "Deduct legitimate business debts".to_string(),
                "Calculate annually on business year-end".to_string(),
            ],
        },
    ];

    ZakatInfoResponse {
        types,
        general_info,
        nisab_info,
        calculation_guidelines,
    }
}
