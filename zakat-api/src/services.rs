use chrono::Utc;
use rust_decimal::Decimal;
use shared::{cache::Cache, error::ApiResult};
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    models::{
        CalculationHistoryResponse, CalculationSummary, Currency, NisabRatesResponse,
        SavedCalculation, ZakatCalculationRequest, ZakatCalculationResponse,
    },
    repository::ZakatRepository,
};

pub struct ZakatService {
    repository: ZakatRepository,
    cache: Cache,
}

impl ZakatService {
    pub fn new(repository: ZakatRepository, cache: Cache) -> Self {
        Self { repository, cache }
    }

    pub async fn save_calculation(
        &self,
        user_id: String,
        request: ZakatCalculationRequest,
        result: ZakatCalculationResponse,
    ) -> ApiResult<SavedCalculation> {
        let calculation = SavedCalculation {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            calculation_type: request.calculation_type.to_string().to_string(),
            input_data: serde_json::to_value(&request)?,
            result_data: serde_json::to_value(&result)?,
            zakat_amount: result.zakat_due,
            currency: request.currency.to_string().to_string(),
            created_at: Utc::now(),
        };

        let saved = self.repository.save_calculation(calculation).await?;

        // Invalidate user's calculation history cache
        let cache_key = format!("zakat_history:{}", user_id);
        if let Err(e) = self.cache.delete(&cache_key).await {
            warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        // Invalidate stats cache
        if let Err(e) = self.cache.delete(&format!("zakat_stats:{}", user_id)).await {
            warn!(
                "Failed to invalidate stats cache for user {}: {}",
                user_id, e
            );
        }

        Ok(saved)
    }

    pub async fn get_calculation_history(
        &self,
        user_id: &str,
    ) -> ApiResult<CalculationHistoryResponse> {
        let cache_key = format!("zakat_history:{}", user_id);

        // Try cache first
        if let Ok(Some(cached_response)) = self
            .cache
            .get::<CalculationHistoryResponse>(&cache_key)
            .await
        {
            debug!("Returning cached calculation history for user: {}", user_id);
            return Ok(cached_response);
        }

        // Fetch from database
        let calculations = self.repository.get_user_calculations(user_id).await?;
        let (total_this_year, total_zakat, most_common_type) =
            self.repository.get_calculation_stats(user_id).await?;

        let summary = CalculationSummary {
            total_zakat_calculated: total_zakat,
            most_common_type: most_common_type.and_then(|t| parse_zakat_type(&t)),
            calculations_this_year: total_this_year,
            average_zakat_amount: if !calculations.is_empty() {
                total_zakat / Decimal::from(calculations.len())
            } else {
                Decimal::ZERO
            },
        };

        let response = CalculationHistoryResponse {
            total: calculations.len() as i64,
            calculations,
            summary,
        };

        // Cache for 30 minutes
        if let Err(e) = self
            .cache
            .set(&cache_key, &response, Some(Duration::from_secs(1800)))
            .await
        {
            warn!("Failed to cache calculation history: {}", e);
        }

        Ok(response)
    }

    pub async fn get_nisab_rates(&self) -> ApiResult<NisabRatesResponse> {
        let cache_key = "nisab_rates";

        // Try cache first
        if let Ok(Some(cached_rates)) = self.cache.get::<NisabRatesResponse>(&cache_key).await {
            debug!("Returning cached nisab rates");
            return Ok(cached_rates);
        }

        // Fetch from database
        let rates = self.repository.get_nisab_rates().await?;
        let currency_rates_map = self.repository.get_currency_rates().await?;

        // Convert to Currency enum map
        let mut currency_rates = std::collections::HashMap::new();
        for (code, rate) in currency_rates_map {
            if let Some(currency) = parse_currency(&code) {
                currency_rates.insert(currency, rate);
            }
        }

        // Find gold and silver rates
        let gold_rate = rates
            .iter()
            .find(|r| r.metal_type == "gold")
            .ok_or_else(|| shared::error::ApiError::not_found("Gold nisab rate"))?;

        let silver_rate = rates
            .iter()
            .find(|r| r.metal_type == "silver")
            .ok_or_else(|| shared::error::ApiError::not_found("Silver nisab rate"))?;

        let response = NisabRatesResponse {
            gold: gold_rate.clone(),
            silver: silver_rate.clone(),
            currency_rates,
            last_updated: Utc::now(),
        };

        // Cache for 1 hour
        if let Err(e) = self
            .cache
            .set(&cache_key, &response, Some(Duration::from_secs(3600)))
            .await
        {
            warn!("Failed to cache nisab rates: {}", e);
        }

        Ok(response)
    }

    pub async fn update_nisab_rates(
        &self,
        gold_price: Option<Decimal>,
        silver_price: Option<Decimal>,
    ) -> ApiResult<()> {
        if let Some(price) = gold_price {
            self.repository.update_nisab_rate("gold", price).await?;
        }

        if let Some(price) = silver_price {
            self.repository.update_nisab_rate("silver", price).await?;
        }

        // Invalidate cache
        if let Err(e) = self.cache.delete("nisab_rates").await {
            warn!("Failed to invalidate nisab rates cache: {}", e);
        }

        Ok(())
    }

    pub async fn update_currency_rates(
        &self,
        rates: std::collections::HashMap<String, Decimal>,
    ) -> ApiResult<()> {
        for (currency, rate) in rates {
            self.repository
                .update_currency_rate(&currency, rate)
                .await?;
        }

        // Invalidate cache
        if let Err(e) = self.cache.delete("nisab_rates").await {
            warn!("Failed to invalidate currency rates cache: {}", e);
        }

        Ok(())
    }

    pub async fn get_global_statistics(&self) -> ApiResult<serde_json::Value> {
        let cache_key = "global_zakat_stats";

        // Try cache first
        if let Ok(Some(cached_stats)) = self.cache.get::<serde_json::Value>(&cache_key).await {
            debug!("Returning cached global statistics");
            return Ok(cached_stats);
        }

        // Fetch from database
        let (total_calculations, total_zakat, unique_users) =
            self.repository.get_global_stats().await?;

        let stats = serde_json::json!({
            "total_calculations": total_calculations,
            "total_zakat_calculated": total_zakat,
            "unique_users": unique_users,
            "average_zakat_per_calculation": if total_calculations > 0 {
                total_zakat / Decimal::from(total_calculations)
            } else {
                Decimal::ZERO
            },
            "last_updated": Utc::now()
        });

        // Cache for 15 minutes
        if let Err(e) = self
            .cache
            .set(&cache_key, &stats, Some(Duration::from_secs(900)))
            .await
        {
            warn!("Failed to cache global statistics: {}", e);
        }

        Ok(stats)
    }

    pub async fn delete_user_data(&self, user_id: &str) -> ApiResult<u64> {
        let deleted_count = self.repository.delete_user_calculations(user_id).await?;

        // Invalidate user caches
        let cache_keys = [
            format!("zakat_history:{}", user_id),
            format!("zakat_stats:{}", user_id),
        ];

        for key in &cache_keys {
            if let Err(e) = self.cache.delete(key).await {
                warn!("Failed to invalidate cache {}: {}", key, e);
            }
        }

        Ok(deleted_count)
    }

    pub async fn get_user_summary(&self, user_id: &str) -> ApiResult<serde_json::Value> {
        let cache_key = format!("zakat_summary:{}", user_id);

        // Try cache first
        if let Ok(Some(cached_summary)) = self.cache.get::<serde_json::Value>(&cache_key).await {
            debug!("Returning cached user summary for: {}", user_id);
            return Ok(cached_summary);
        }

        // Fetch calculation history
        let calculations = self.repository.get_user_calculations(user_id).await?;
        let (total_this_year, total_zakat, most_common_type) =
            self.repository.get_calculation_stats(user_id).await?;

        // Calculate additional insights
        let calculation_types: std::collections::HashMap<String, usize> =
            calculations
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, calc| {
                    *acc.entry(calc.calculation_type.clone()).or_insert(0) += 1;
                    acc
                });

        let monthly_totals = calculate_monthly_totals(&calculations);
        let recent_activity = calculations.len() >= 5; // Considered active if 5+ calculations

        let summary = serde_json::json!({
            "user_id": user_id,
            "total_calculations": calculations.len(),
            "total_zakat_calculated": total_zakat,
            "calculations_this_year": total_this_year,
            "most_common_type": most_common_type,
            "calculation_types_breakdown": calculation_types,
            "monthly_totals": monthly_totals,
            "is_active_user": recent_activity,
            "average_zakat_amount": if !calculations.is_empty() {
                total_zakat / Decimal::from(calculations.len())
            } else {
                Decimal::ZERO
            },
            "last_calculation_date": calculations.first().map(|c| c.created_at),
            "generated_at": Utc::now()
        });

        // Cache for 1 hour
        if let Err(e) = self
            .cache
            .set(&cache_key, &summary, Some(Duration::from_secs(3600)))
            .await
        {
            warn!("Failed to cache user summary: {}", e);
        }

        Ok(summary)
    }
}

// Helper functions
fn parse_zakat_type(type_str: &str) -> Option<crate::models::ZakatType> {
    match type_str.to_lowercase().as_str() {
        "wealth" => Some(crate::models::ZakatType::Wealth),
        "gold" => Some(crate::models::ZakatType::Gold),
        "silver" => Some(crate::models::ZakatType::Silver),
        "business" => Some(crate::models::ZakatType::Business),
        "livestock" => Some(crate::models::ZakatType::Livestock),
        "crops" => Some(crate::models::ZakatType::Crops),
        _ => None,
    }
}

fn parse_currency(code: &str) -> Option<Currency> {
    match code.to_uppercase().as_str() {
        "USD" => Some(Currency::USD),
        "EUR" => Some(Currency::EUR),
        "GBP" => Some(Currency::GBP),
        "SAR" => Some(Currency::SAR),
        "AED" => Some(Currency::AED),
        "PKR" => Some(Currency::PKR),
        "INR" => Some(Currency::INR),
        "BDT" => Some(Currency::BDT),
        "MYR" => Some(Currency::MYR),
        "IDR" => Some(Currency::IDR),
        "TRY" => Some(Currency::TRY),
        "EGP" => Some(Currency::EGP),
        _ => None,
    }
}

fn calculate_monthly_totals(
    calculations: &[SavedCalculation],
) -> std::collections::HashMap<String, Decimal> {
    let mut monthly_totals = std::collections::HashMap::new();

    for calc in calculations {
        let month_key = calc.created_at.format("%Y-%m").to_string();
        let entry = monthly_totals.entry(month_key).or_insert(Decimal::ZERO);
        *entry += calc.zakat_amount;
    }

    monthly_totals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zakat_type() {
        assert!(matches!(
            parse_zakat_type("wealth"),
            Some(crate::models::ZakatType::Wealth)
        ));
        assert!(matches!(
            parse_zakat_type("GOLD"),
            Some(crate::models::ZakatType::Gold)
        ));
        assert!(matches!(
            parse_zakat_type("Business"),
            Some(crate::models::ZakatType::Business)
        ));
        assert!(parse_zakat_type("invalid").is_none());
    }

    #[test]
    fn test_parse_currency() {
        assert!(matches!(parse_currency("USD"), Some(Currency::USD)));
        assert!(matches!(parse_currency("eur"), Some(Currency::EUR)));
        assert!(matches!(parse_currency("PKR"), Some(Currency::PKR)));
        assert!(parse_currency("INVALID").is_none());
    }

    #[test]
    fn test_calculate_monthly_totals() {
        use chrono::{TimeZone, Utc};
        use rust_decimal_macros::dec;

        let calculations = vec![
            SavedCalculation {
                id: Uuid::new_v4(),
                user_id: "test".to_string(),
                calculation_type: "wealth".to_string(),
                input_data: serde_json::json!({}),
                result_data: serde_json::json!({}),
                zakat_amount: dec!(100.0),
                currency: "USD".to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
            },
            SavedCalculation {
                id: Uuid::new_v4(),
                user_id: "test".to_string(),
                calculation_type: "gold".to_string(),
                input_data: serde_json::json!({}),
                result_data: serde_json::json!({}),
                zakat_amount: dec!(50.0),
                currency: "USD".to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 1, 20, 10, 0, 0).unwrap(),
            },
            SavedCalculation {
                id: Uuid::new_v4(),
                user_id: "test".to_string(),
                calculation_type: "wealth".to_string(),
                input_data: serde_json::json!({}),
                result_data: serde_json::json!({}),
                zakat_amount: dec!(75.0),
                currency: "USD".to_string(),
                created_at: Utc.with_ymd_and_hms(2024, 2, 10, 10, 0, 0).unwrap(),
            },
        ];

        let totals = calculate_monthly_totals(&calculations);

        assert_eq!(totals.get("2024-01"), Some(&dec!(150.0)));
        assert_eq!(totals.get("2024-02"), Some(&dec!(75.0)));
        assert_eq!(totals.len(), 2);
    }
}
