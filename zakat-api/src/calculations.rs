use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use shared::error::{ApiError, ApiResult};
use uuid::Uuid;

use crate::models::{
    BusinessZakatDetails, CropsZakatDetails, Currency, IrrigationMethod, IslamicReference,
    LivestockZakatDetails, MetalZakatDetails, WealthZakatDetails, ZakatCalculationRequest,
    ZakatCalculationResponse, ZakatDetails, ZakatType,
};

pub struct ZakatCalculator {
    // Current market rates (would normally be fetched from external API)
    gold_price_per_gram_usd: Decimal,
    silver_price_per_gram_usd: Decimal,
    currency_rates: std::collections::HashMap<Currency, Decimal>,
}

impl ZakatCalculator {
    pub fn new() -> Self {
        let mut currency_rates = std::collections::HashMap::new();

        // Sample exchange rates (in production, fetch from live API)
        currency_rates.insert(Currency::USD, dec!(1.0));
        currency_rates.insert(Currency::EUR, dec!(0.85));
        currency_rates.insert(Currency::GBP, dec!(0.73));
        currency_rates.insert(Currency::SAR, dec!(3.75));
        currency_rates.insert(Currency::AED, dec!(3.67));
        currency_rates.insert(Currency::PKR, dec!(280.0));
        currency_rates.insert(Currency::INR, dec!(83.0));
        currency_rates.insert(Currency::BDT, dec!(110.0));
        currency_rates.insert(Currency::MYR, dec!(4.7));
        currency_rates.insert(Currency::IDR, dec!(15500.0));
        currency_rates.insert(Currency::TRY, dec!(27.0));
        currency_rates.insert(Currency::EGP, dec!(31.0));

        Self {
            gold_price_per_gram_usd: dec!(65.0),   // ~$65 per gram
            silver_price_per_gram_usd: dec!(0.80), // ~$0.80 per gram
            currency_rates,
        }
    }

    pub async fn calculate_zakat(
        &self,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let calculation_id = Uuid::new_v4();

        match request.calculation_type {
            ZakatType::Wealth => self.calculate_wealth_zakat(calculation_id, request).await,
            ZakatType::Gold => self.calculate_gold_zakat(calculation_id, request).await,
            ZakatType::Silver => self.calculate_silver_zakat(calculation_id, request).await,
            ZakatType::Business => self.calculate_business_zakat(calculation_id, request).await,
            ZakatType::Livestock => {
                self.calculate_livestock_zakat(calculation_id, request)
                    .await
            }
            ZakatType::Crops => self.calculate_crops_zakat(calculation_id, request).await,
        }
    }

    async fn calculate_wealth_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let amount_in_usd = self.convert_to_usd(request.amount, request.currency)?;

        // Nisab for wealth is equivalent to 85g of gold or 595g of silver (whichever is lower)
        let gold_nisab_usd = dec!(85.0) * self.gold_price_per_gram_usd;
        let silver_nisab_usd = dec!(595.0) * self.silver_price_per_gram_usd;
        let nisab_threshold = gold_nisab_usd.min(silver_nisab_usd);

        let is_zakat_applicable = amount_in_usd >= nisab_threshold;
        let zakat_rate = dec!(2.5); // 2.5%
        let zakat_due = if is_zakat_applicable {
            amount_in_usd * zakat_rate / dec!(100.0)
        } else {
            dec!(0.0)
        };

        let details = WealthZakatDetails {
            cash_savings: request.amount,
            investments: dec!(0.0), // Could be separated in future
            total_wealth: request.amount,
            nisab_equivalent_gold: gold_nisab_usd,
            nisab_equivalent_silver: silver_nisab_usd,
            years_held: None, // Could be added as input
        };

        let recommendations = self.get_wealth_recommendations(amount_in_usd, nisab_threshold);
        let islamic_references = self.get_wealth_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Wealth,
            input_amount: request.amount,
            currency: request.currency,
            nisab_threshold: self.convert_from_usd(nisab_threshold, request.currency)?,
            zakat_due: self.convert_from_usd(zakat_due, request.currency)?,
            zakat_percentage: zakat_rate,
            is_zakat_applicable,
            calculation_details: ZakatDetails::Wealth(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    async fn calculate_gold_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let weight_grams = request
            .gold_weight_grams
            .ok_or_else(|| ApiError::invalid_input("Gold weight in grams is required"))?;

        let purity_karats = request.gold_purity_karats.unwrap_or(24);
        let purity_percentage = self.karat_to_purity_percentage(purity_karats)?;
        let pure_weight_grams = weight_grams * purity_percentage / dec!(100.0);

        let nisab_weight_grams = dec!(85.0); // 85 grams of pure gold
        let is_zakat_applicable = pure_weight_grams >= nisab_weight_grams;

        let total_value_usd = pure_weight_grams * self.gold_price_per_gram_usd;
        let zakat_rate = dec!(2.5);
        let zakat_due_usd = if is_zakat_applicable {
            total_value_usd * zakat_rate / dec!(100.0)
        } else {
            dec!(0.0)
        };

        let details = MetalZakatDetails {
            weight_grams,
            purity_percentage,
            pure_weight_grams,
            current_price_per_gram: self.gold_price_per_gram_usd,
            total_value: total_value_usd,
            nisab_weight_grams,
        };

        let recommendations = self.get_gold_recommendations(pure_weight_grams, nisab_weight_grams);
        let islamic_references = self.get_gold_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Gold,
            input_amount: weight_grams,
            currency: Currency::USD, // Gold calculations in USD
            nisab_threshold: nisab_weight_grams,
            zakat_due: self.convert_from_usd(zakat_due_usd, request.currency)?,
            zakat_percentage: zakat_rate,
            is_zakat_applicable,
            calculation_details: ZakatDetails::Gold(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    async fn calculate_silver_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let weight_grams = request
            .silver_weight_grams
            .ok_or_else(|| ApiError::invalid_input("Silver weight in grams is required"))?;

        let purity_percentage = dec!(92.5); // Standard silver purity
        let pure_weight_grams = weight_grams * purity_percentage / dec!(100.0);

        let nisab_weight_grams = dec!(595.0); // 595 grams of pure silver
        let is_zakat_applicable = pure_weight_grams >= nisab_weight_grams;

        let total_value_usd = pure_weight_grams * self.silver_price_per_gram_usd;
        let zakat_rate = dec!(2.5);
        let zakat_due_usd = if is_zakat_applicable {
            total_value_usd * zakat_rate / dec!(100.0)
        } else {
            dec!(0.0)
        };

        let details = MetalZakatDetails {
            weight_grams,
            purity_percentage,
            pure_weight_grams,
            current_price_per_gram: self.silver_price_per_gram_usd,
            total_value: total_value_usd,
            nisab_weight_grams,
        };

        let recommendations =
            self.get_silver_recommendations(pure_weight_grams, nisab_weight_grams);
        let islamic_references = self.get_silver_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Silver,
            input_amount: weight_grams,
            currency: Currency::USD,
            nisab_threshold: nisab_weight_grams,
            zakat_due: self.convert_from_usd(zakat_due_usd, request.currency)?,
            zakat_percentage: zakat_rate,
            is_zakat_applicable,
            calculation_details: ZakatDetails::Silver(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    async fn calculate_business_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let assets = request.business_assets.unwrap_or(dec!(0.0));
        let liabilities = request.business_liabilities.unwrap_or(dec!(0.0));
        let inventory = request.inventory_value.unwrap_or(dec!(0.0));

        let net_assets = assets - liabilities;
        let zakatable_amount = net_assets + inventory + request.amount; // Cash + net assets + inventory

        let nisab_usd = dec!(85.0) * self.gold_price_per_gram_usd;
        let zakatable_amount_usd = self.convert_to_usd(zakatable_amount, request.currency)?;

        let is_zakat_applicable = zakatable_amount_usd >= nisab_usd;
        let zakat_rate = dec!(2.5);
        let zakat_due_usd = if is_zakat_applicable {
            zakatable_amount_usd * zakat_rate / dec!(100.0)
        } else {
            dec!(0.0)
        };

        let details = BusinessZakatDetails {
            total_assets: assets,
            total_liabilities: liabilities,
            net_assets,
            inventory_value: inventory,
            cash_equivalents: request.amount,
            zakatable_amount,
        };

        let recommendations = self.get_business_recommendations(&details);
        let islamic_references = self.get_business_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Business,
            input_amount: request.amount,
            currency: request.currency,
            nisab_threshold: self.convert_from_usd(nisab_usd, request.currency)?,
            zakat_due: self.convert_from_usd(zakat_due_usd, request.currency)?,
            zakat_percentage: zakat_rate,
            is_zakat_applicable,
            calculation_details: ZakatDetails::Business(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    async fn calculate_livestock_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let cattle = request.cattle_count.unwrap_or(0);
        let sheep_goats = request.sheep_goat_count.unwrap_or(0);
        let camels = request.camel_count.unwrap_or(0);

        let total_animals = cattle + sheep_goats + camels;

        // Simplified livestock zakat calculation
        let zakat_animals_due = self.calculate_livestock_zakat_animals(cattle, sheep_goats, camels);
        let is_zakat_applicable = zakat_animals_due > 0;

        // Estimate cash value (simplified)
        let estimated_value_per_animal = dec!(500.0); // USD
        let alternative_cash_value = if zakat_animals_due > 0 {
            Some(Decimal::from(zakat_animals_due) * estimated_value_per_animal)
        } else {
            None
        };

        let details = LivestockZakatDetails {
            cattle_count: cattle,
            sheep_goat_count: sheep_goats,
            camel_count: camels,
            total_animals,
            zakat_animals_due,
            alternative_cash_value,
        };

        let recommendations = self.get_livestock_recommendations(&details);
        let islamic_references = self.get_livestock_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Livestock,
            input_amount: Decimal::from(total_animals),
            currency: Currency::USD,
            nisab_threshold: dec!(40.0), // Minimum threshold varies by animal type
            zakat_due: alternative_cash_value.unwrap_or(dec!(0.0)),
            zakat_percentage: dec!(0.0), // Variable based on count
            is_zakat_applicable,
            calculation_details: ZakatDetails::Livestock(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    async fn calculate_crops_zakat(
        &self,
        calculation_id: Uuid,
        request: ZakatCalculationRequest,
    ) -> ApiResult<ZakatCalculationResponse> {
        let crop_type = request.crop_type.unwrap_or_else(|| "General".to_string());
        let irrigation = request
            .irrigation_method
            .unwrap_or(IrrigationMethod::Natural);

        let zakat_percentage = match irrigation {
            IrrigationMethod::Natural => dec!(10.0), // 10% for rain-fed crops
            IrrigationMethod::Manual => dec!(5.0),   // 5% for irrigated crops
        };

        let harvest_value_usd = self.convert_to_usd(request.amount, request.currency)?;
        let nisab_usd = dec!(653.0); // Approximately 5 Awsuq (about 653 kg of dates/wheat value)

        let is_zakat_applicable = harvest_value_usd >= nisab_usd;
        let zakat_due_usd = if is_zakat_applicable {
            harvest_value_usd * zakat_percentage / dec!(100.0)
        } else {
            dec!(0.0)
        };

        let details = CropsZakatDetails {
            crop_type,
            harvest_amount: request.amount,
            irrigation_method: irrigation,
            zakat_percentage,
            deductible_expenses: None,
            net_harvest_value: request.amount,
        };

        let recommendations = self.get_crops_recommendations(&details);
        let islamic_references = self.get_crops_references();

        Ok(ZakatCalculationResponse {
            calculation_id,
            calculation_type: ZakatType::Crops,
            input_amount: request.amount,
            currency: request.currency,
            nisab_threshold: self.convert_from_usd(nisab_usd, request.currency)?,
            zakat_due: self.convert_from_usd(zakat_due_usd, request.currency)?,
            zakat_percentage,
            is_zakat_applicable,
            calculation_details: ZakatDetails::Crops(details),
            recommendations,
            islamic_references,
            calculation_time: Utc::now(),
        })
    }

    // Helper methods
    fn convert_to_usd(&self, amount: Decimal, currency: Currency) -> ApiResult<Decimal> {
        let rate = self.currency_rates.get(&currency).ok_or_else(|| {
            ApiError::invalid_input(format!("Unsupported currency: {:?}", currency))
        })?;
        Ok(amount / rate)
    }

    fn convert_from_usd(&self, amount_usd: Decimal, currency: Currency) -> ApiResult<Decimal> {
        let rate = self.currency_rates.get(&currency).ok_or_else(|| {
            ApiError::invalid_input(format!("Unsupported currency: {:?}", currency))
        })?;
        Ok(amount_usd * rate)
    }

    fn karat_to_purity_percentage(&self, karats: u8) -> ApiResult<Decimal> {
        match karats {
            24 => Ok(dec!(100.0)),
            22 => Ok(dec!(91.7)),
            18 => Ok(dec!(75.0)),
            14 => Ok(dec!(58.3)),
            10 => Ok(dec!(41.7)),
            _ => Err(ApiError::calculation(
                "Unsupported gold purity. Supported: 10K, 14K, 18K, 22K, 24K",
            )),
        }
    }

    fn calculate_livestock_zakat_animals(&self, cattle: u32, sheep_goats: u32, camels: u32) -> u32 {
        let mut total_due = 0;

        // Simplified calculation - in reality, each type has specific thresholds
        if sheep_goats >= 40 {
            total_due += 1; // 1 sheep for 40-120 sheep
        }
        if cattle >= 30 {
            total_due += 1; // 1 calf for 30-39 cattle
        }
        if camels >= 5 {
            total_due += 1; // 1 sheep for 5-9 camels
        }

        total_due
    }

    // Recommendation methods
    fn get_wealth_recommendations(&self, amount_usd: Decimal, nisab_usd: Decimal) -> Vec<String> {
        let mut recommendations = Vec::new();

        if amount_usd < nisab_usd {
            recommendations.push(
                "Your wealth is below the nisab threshold, so Zakat is not required.".to_string(),
            );
        } else {
            recommendations.push("Zakat is due on your wealth. Pay 2.5% annually.".to_string());
            recommendations
                .push("Ensure you've held this wealth for a full lunar year (Hawl).".to_string());
        }

        recommendations.push(
            "Consider consulting with a qualified Islamic scholar for complex cases.".to_string(),
        );
        recommendations
    }

    fn get_gold_recommendations(&self, weight: Decimal, nisab: Decimal) -> Vec<String> {
        let mut recommendations = Vec::new();

        if weight < nisab {
            recommendations
                .push("Your gold is below the nisab threshold (85g of pure gold).".to_string());
        } else {
            recommendations.push(
                "Zakat is due on your gold. Consider the purity when calculating.".to_string(),
            );
        }

        recommendations.push(
            "Jewelry worn regularly may have different rulings - consult a scholar.".to_string(),
        );
        recommendations
    }

    fn get_silver_recommendations(&self, weight: Decimal, nisab: Decimal) -> Vec<String> {
        let mut recommendations = Vec::new();

        if weight < nisab {
            recommendations.push(
                "Your silver is below the nisab threshold (595g of pure silver).".to_string(),
            );
        } else {
            recommendations.push("Zakat is due on your silver holdings.".to_string());
        }

        recommendations
    }

    fn get_business_recommendations(&self, _details: &BusinessZakatDetails) -> Vec<String> {
        let mut recommendations = Vec::new();

        recommendations.push(
            "Include all business assets, inventory, and cash in Zakat calculation.".to_string(),
        );
        recommendations.push("Deduct legitimate business liabilities and debts.".to_string());
        recommendations
            .push("Calculate Zakat annually on your business lunar year-end.".to_string());

        recommendations
    }

    fn get_livestock_recommendations(&self, details: &LivestockZakatDetails) -> Vec<String> {
        let mut recommendations = Vec::new();

        if details.total_animals == 0 {
            recommendations.push("No livestock for Zakat calculation.".to_string());
        } else {
            recommendations
                .push("Livestock Zakat has specific thresholds for each animal type.".to_string());
            recommendations
                .push("Animals must be grazing freely for most of the year.".to_string());
            recommendations
                .push("Consult detailed Islamic texts for precise calculations.".to_string());
        }

        recommendations
    }

    fn get_crops_recommendations(&self, details: &CropsZakatDetails) -> Vec<String> {
        let mut recommendations = Vec::new();

        match details.irrigation_method {
            IrrigationMethod::Natural => {
                recommendations.push("Rain-fed crops: 10% Zakat rate applies.".to_string());
            }
            IrrigationMethod::Manual => {
                recommendations.push("Irrigated crops: 5% Zakat rate applies.".to_string());
            }
        }

        recommendations.push("Zakat is due at harvest time, not annually.".to_string());
        recommendations
            .push("Only staple crops like wheat, rice, dates typically require Zakat.".to_string());

        recommendations
    }

    // Islamic reference methods
    fn get_wealth_references(&self) -> Vec<IslamicReference> {
        vec![IslamicReference {
            source: "Quran".to_string(),
            reference: "Surah At-Tawbah 9:103".to_string(),
            arabic_text: Some("خُذْ مِنْ أَمْوَالِهِمْ صَدَقَةً تُطَهِّرُهُمْ وَتُزَكِّيهِمْ بِهَا".to_string()),
            translation:
                "Take from their wealth a charity by which you purify them and cause them increase"
                    .to_string(),
        }]
    }

    fn get_gold_references(&self) -> Vec<IslamicReference> {
        vec![IslamicReference {
            source: "Hadith".to_string(),
            reference: "Sunan Abu Dawood".to_string(),
            arabic_text: None,
            translation:
                "No Zakat is due on gold until it reaches 20 dinars (approximately 85 grams)"
                    .to_string(),
        }]
    }

    fn get_silver_references(&self) -> Vec<IslamicReference> {
        vec![IslamicReference {
            source: "Hadith".to_string(),
            reference: "Sahih Bukhari".to_string(),
            arabic_text: None,
            translation:
                "No Zakat is due on silver until it reaches 200 dirhams (approximately 595 grams)"
                    .to_string(),
        }]
    }

    fn get_business_references(&self) -> Vec<IslamicReference> {
        vec![IslamicReference {
            source: "Islamic Jurisprudence".to_string(),
            reference: "Fiqh al-Zakat".to_string(),
            arabic_text: None,
            translation: "Business assets and inventory are subject to Zakat if held for trade"
                .to_string(),
        }]
    }

    fn get_livestock_references(&self) -> Vec<IslamicReference> {
        vec![IslamicReference {
            source: "Hadith".to_string(),
            reference: "Sahih Bukhari".to_string(),
            arabic_text: None,
            translation:
                "On grazing livestock, specific Zakat rates apply based on numbers and type"
                    .to_string(),
        }]
    }

    fn get_crops_references(&self) -> Vec<IslamicReference> {
        vec![
            IslamicReference {
                source: "Hadith".to_string(),
                reference: "Sahih Bukhari".to_string(),
                arabic_text: None,
                translation: "On crops watered by rain or springs: one-tenth. On crops watered by irrigation: one-twentieth".to_string(),
            },
        ]
    }
}
