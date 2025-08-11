use csv::Reader;
use serde::Deserialize;
use shared::error::{ApiError, ApiResult};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::models::StandardMethod;

#[derive(Debug, Deserialize)]
struct PreferredRecord {
    country: String,
    alternative: Option<String>,
    method: StandardMethod,
}

pub struct PreferredMethodMap {
    map: HashMap<String, StandardMethod>,
}

impl PreferredMethodMap {
    pub fn load(path: &str) -> ApiResult<Self> {
        let mut map = HashMap::new();

        match Reader::from_path(path) {
            Ok(mut reader) => {
                let mut count = 0;
                for result in reader.deserialize() {
                    match result {
                        Ok(record) => {
                            let record: PreferredRecord = record;
                            map.insert(record.country.to_lowercase(), record.method);
                            if let Some(alternative) = record.alternative {
                                map.insert(alternative.to_lowercase(), record.method);
                            }
                            count += 1;
                        }
                        Err(e) => {
                            warn!("Failed to parse preferred method record: {}", e);
                        }
                    }
                }
                info!("Loaded {} preferred method mappings from {}", count, path);
            }
            Err(e) => {
                warn!("Failed to load preferred methods file {}: {}", path, e);
                // Use default mappings if file is not found
                Self::load_default_mappings(&mut map);
            }
        }

        Ok(Self { map })
    }

    pub fn get(&self, country: &str) -> ApiResult<StandardMethod> {
        let country_lower = country.to_lowercase();
        self.map.get(&country_lower).copied().ok_or_else(|| {
            ApiError::NotFound(format!(
                "No preferred method found for country: {}. Please specify a method explicitly.",
                country
            ))
        })
    }

    fn load_default_mappings(map: &mut HashMap<String, StandardMethod>) {
        // Default country mappings based on common usage
        let defaults = [
            // Middle East
            ("saudi arabia", StandardMethod::Makkah),
            ("uae", StandardMethod::Dubai),
            ("united arab emirates", StandardMethod::Dubai),
            ("kuwait", StandardMethod::Kuwait),
            ("qatar", StandardMethod::Qatar),
            ("iran", StandardMethod::Tehran),
            ("iraq", StandardMethod::Karachi),
            ("syria", StandardMethod::Karachi),
            ("lebanon", StandardMethod::Karachi),
            ("jordan", StandardMethod::Karachi),
            ("palestine", StandardMethod::Karachi),
            ("israel", StandardMethod::Karachi),
            // South Asia
            ("pakistan", StandardMethod::Karachi),
            ("india", StandardMethod::Karachi),
            ("bangladesh", StandardMethod::Karachi),
            ("afghanistan", StandardMethod::Karachi),
            ("sri lanka", StandardMethod::Karachi),
            ("maldives", StandardMethod::Karachi),
            // Southeast Asia
            ("malaysia", StandardMethod::Jakim),
            ("singapore", StandardMethod::Singapore),
            ("indonesia", StandardMethod::Karachi),
            ("brunei", StandardMethod::Singapore),
            ("thailand", StandardMethod::Singapore),
            // Africa
            ("egypt", StandardMethod::Egypt),
            ("libya", StandardMethod::Egypt),
            ("tunisia", StandardMethod::Egypt),
            ("algeria", StandardMethod::Egypt),
            ("morocco", StandardMethod::Egypt),
            ("sudan", StandardMethod::Karachi),
            ("south africa", StandardMethod::Karachi),
            ("nigeria", StandardMethod::Karachi),
            // Europe
            ("turkey", StandardMethod::Diyanet),
            ("france", StandardMethod::Uoif),
            ("germany", StandardMethod::Mwl),
            ("uk", StandardMethod::Mwl),
            ("united kingdom", StandardMethod::Mwl),
            ("netherlands", StandardMethod::Mwl),
            ("belgium", StandardMethod::Mwl),
            ("sweden", StandardMethod::Mwl),
            ("norway", StandardMethod::Mwl),
            ("denmark", StandardMethod::Mwl),
            ("russia", StandardMethod::Russia),
            // North America
            ("usa", StandardMethod::Isna),
            ("united states", StandardMethod::Isna),
            ("canada", StandardMethod::Isna),
            ("mexico", StandardMethod::Isna),
        ];

        for (country, method) in &defaults {
            map.insert(country.to_string(), *method);
        }
    }

    pub fn list_supported_countries(&self) -> Vec<String> {
        let mut countries: Vec<String> = self.map.keys().cloned().collect();
        countries.sort();
        countries
    }

    pub fn get_method_for_countries(&self, method: StandardMethod) -> Vec<String> {
        self.map
            .iter()
            .filter(|(_, &m)| m == method)
            .map(|(country, _)| country.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preferred_method_map() {
        let map = PreferredMethodMap::load("nonexistent.csv").unwrap();

        // Test some default mappings
        assert_eq!(map.get("pakistan").unwrap(), StandardMethod::Karachi);
        assert_eq!(map.get("usa").unwrap(), StandardMethod::Isna);
        assert_eq!(map.get("egypt").unwrap(), StandardMethod::Egypt);
        assert_eq!(map.get("turkey").unwrap(), StandardMethod::Diyanet);

        // Test case insensitivity
        assert_eq!(map.get("PAKISTAN").unwrap(), StandardMethod::Karachi);
        assert_eq!(map.get("UsA").unwrap(), StandardMethod::Isna);

        // Test unknown country
        assert!(map.get("unknown_country").is_err());
    }

    #[test]
    fn test_list_supported_countries() {
        let map = PreferredMethodMap::load("nonexistent.csv").unwrap();
        let countries = map.list_supported_countries();

        assert!(!countries.is_empty());
        assert!(countries.contains(&"pakistan".to_string()));
        assert!(countries.contains(&"usa".to_string()));
    }

    #[test]
    fn test_get_method_for_countries() {
        let map = PreferredMethodMap::load("nonexistent.csv").unwrap();
        let karachi_countries = map.get_method_for_countries(StandardMethod::Karachi);

        assert!(karachi_countries.contains(&"pakistan".to_string()));
        assert!(karachi_countries.contains(&"india".to_string()));
    }
}
