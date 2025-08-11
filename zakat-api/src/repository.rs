use rust_decimal::Decimal;
use shared::{database::Database, error::ApiResult};
use sqlx::{query, query_as, query_scalar, Row};
use tracing::{debug, info};

use crate::models::{NisabRate, NisabRateRow, SavedCalculation, SavedCalculationRow};

pub struct ZakatRepository {
    db: Database,
}

impl ZakatRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn save_calculation(
        &self,
        calculation: SavedCalculation,
    ) -> ApiResult<SavedCalculation> {
        debug!("Saving zakat calculation for user: {}", calculation.user_id);

        let result = sqlx::query_as::<_, SavedCalculationRow>(
            r#"
            INSERT INTO zakat_calculations (
                id, user_id, calculation_type, input_data, result_data, 
                zakat_amount, currency, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(calculation.id)
        .bind(calculation.user_id)
        .bind(calculation.calculation_type)
        .bind(calculation.input_data)
        .bind(calculation.result_data)
        .bind(calculation.zakat_amount.to_string()) // Convert to string
        .bind(calculation.currency)
        .bind(calculation.created_at)
        .fetch_one(&self.db.pool)
        .await?;

        let saved_calculation = SavedCalculation::from(result);
        info!("Saved zakat calculation with ID: {}", saved_calculation.id);
        Ok(saved_calculation)
    }

    pub async fn get_user_calculations(&self, user_id: &str) -> ApiResult<Vec<SavedCalculation>> {
        debug!("Fetching calculations for user: {}", user_id);

        let calculations = sqlx::query_as::<_, SavedCalculationRow>(
            "SELECT * FROM zakat_calculations WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.db.pool)
        .await?;

        let result: Vec<SavedCalculation> = calculations
            .into_iter()
            .map(SavedCalculation::from)
            .collect();
        debug!("Found {} calculations for user {}", result.len(), user_id);
        Ok(result)
    }

    pub async fn get_calculation_stats(
        &self,
        user_id: &str,
    ) -> ApiResult<(i64, Decimal, Option<String>)> {
        debug!("Fetching calculation stats for user: {}", user_id);

        // Total calculations this year
        let total_this_year = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zakat_calculations WHERE user_id = $1 AND created_at > DATE_TRUNC('year', NOW())",
        )
        .bind(user_id)
        .fetch_one(&self.db.pool)
        .await?;

        // Total zakat amount calculated - fetch as string and convert
        let total_zakat_string = sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(SUM(zakat_amount::numeric), '0') FROM zakat_calculations WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.db.pool)
        .await?;

        let total_zakat: Decimal = total_zakat_string.parse().unwrap_or(Decimal::ZERO);

        // Most common calculation type
        let most_common_type = sqlx::query(
            r#"
            SELECT calculation_type 
            FROM zakat_calculations 
            WHERE user_id = $1 
            GROUP BY calculation_type 
            ORDER BY COUNT(*) DESC 
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db.pool)
        .await?
        .map(|row| row.get::<String, _>("calculation_type"));

        Ok((total_this_year, total_zakat, most_common_type))
    }

    pub async fn get_nisab_rates(&self) -> ApiResult<Vec<NisabRate>> {
        debug!("Fetching current nisab rates");

        let rates =
            sqlx::query_as::<_, NisabRateRow>("SELECT * FROM nisab_rates ORDER BY metal_type")
                .fetch_all(&self.db.pool)
                .await?;

        let result: Vec<NisabRate> = rates.into_iter().map(NisabRate::from).collect();
        debug!("Found {} nisab rates", result.len());
        Ok(result)
    }

    pub async fn update_nisab_rate(
        &self,
        metal_type: &str,
        price_per_gram: Decimal,
    ) -> ApiResult<NisabRate> {
        debug!("Updating nisab rate for {}: {}", metal_type, price_per_gram);

        let result = sqlx::query_as::<_, NisabRateRow>(
            r#"
            UPDATE nisab_rates 
            SET price_per_gram_usd = $1, last_updated = NOW() 
            WHERE metal_type = $2 
            RETURNING *
            "#,
        )
        .bind(price_per_gram.to_string()) // Convert to string
        .bind(metal_type)
        .fetch_one(&self.db.pool)
        .await?;

        let updated_rate = NisabRate::from(result);
        info!("Updated nisab rate for {}", metal_type);
        Ok(updated_rate)
    }

    pub async fn get_currency_rates(
        &self,
    ) -> ApiResult<std::collections::HashMap<String, Decimal>> {
        debug!("Fetching currency exchange rates");

        let rates = sqlx::query("SELECT currency_code, rate_to_usd FROM currency_rates")
            .fetch_all(&self.db.pool)
            .await?;

        let mut currency_map = std::collections::HashMap::new();
        for rate in rates {
            let currency_code: String = rate.get("currency_code");
            let rate_to_usd_string: String = rate.get("rate_to_usd");
            let rate_to_usd: Decimal = rate_to_usd_string.parse().unwrap_or(Decimal::ZERO);
            currency_map.insert(currency_code, rate_to_usd);
        }

        debug!("Found {} currency rates", currency_map.len());
        Ok(currency_map)
    }

    pub async fn update_currency_rate(&self, currency: &str, rate: Decimal) -> ApiResult<()> {
        debug!("Updating currency rate for {}: {}", currency, rate);

        sqlx::query(
            r#"
            INSERT INTO currency_rates (currency_code, rate_to_usd, source)
            VALUES ($1, $2, 'API Update')
            ON CONFLICT (currency_code)
            DO UPDATE SET rate_to_usd = $2, last_updated = NOW()
            "#,
        )
        .bind(currency)
        .bind(rate.to_string()) // Convert to string
        .execute(&self.db.pool)
        .await?;

        info!("Updated currency rate for {}", currency);
        Ok(())
    }

    pub async fn get_global_stats(&self) -> ApiResult<(i64, Decimal, i64)> {
        debug!("Fetching global zakat calculation statistics");

        // Total calculations
        let total_calculations =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zakat_calculations")
                .fetch_one(&self.db.pool)
                .await?;

        // Total zakat calculated
        let total_zakat_string = sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(SUM(zakat_amount::numeric), '0') FROM zakat_calculations",
        )
        .fetch_one(&self.db.pool)
        .await?;

        let total_zakat: Decimal = total_zakat_string.parse().unwrap_or(Decimal::ZERO);

        // Unique users
        let unique_users =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT user_id) FROM zakat_calculations")
                .fetch_one(&self.db.pool)
                .await?;

        Ok((total_calculations, total_zakat, unique_users))
    }

    pub async fn delete_user_calculations(&self, user_id: &str) -> ApiResult<u64> {
        debug!("Deleting all calculations for user: {}", user_id);

        let rows_affected = sqlx::query("DELETE FROM zakat_calculations WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.db.pool)
            .await?
            .rows_affected();

        info!(
            "Deleted {} calculations for user {}",
            rows_affected, user_id
        );
        Ok(rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_save_and_retrieve_calculation() {
        if std::env::var("DATABASE_URL").is_err() {
            return;
        }

        let db = Database::new(&shared::config::DatabaseConfig {
            url: std::env::var("DATABASE_URL").unwrap(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
        })
        .await
        .unwrap();

        let repo = ZakatRepository::new(db);

        let test_calculation = SavedCalculation {
            id: Uuid::new_v4(),
            user_id: "test_user_123".to_string(),
            calculation_type: "wealth".to_string(),
            input_data: serde_json::json!({"amount": 10000, "currency": "USD"}),
            result_data: serde_json::json!({"zakat_due": 250, "nisab_threshold": 5525}),
            zakat_amount: Decimal::new(25000, 2), // 250.00
            currency: "USD".to_string(),
            created_at: Utc::now(),
        };

        // Save calculation
        let saved = repo.save_calculation(test_calculation.clone()).await;
        assert!(saved.is_ok());

        // Retrieve calculations
        let retrieved = repo.get_user_calculations("test_user_123").await;
        assert!(retrieved.is_ok());

        let calculations = retrieved.unwrap();
        assert!(!calculations.is_empty());
        assert_eq!(calculations[0].user_id, "test_user_123");

        // Clean up
        repo.delete_user_calculations("test_user_123")
            .await
            .unwrap();
    }
}
