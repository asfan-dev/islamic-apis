use libm::{acos, asin, atan2, cos, sin, sqrt};
use shared::error::{ApiError, ApiResult};

use crate::models::{CoordinatesValidation, LocationInfo, QiblaDetailed, QiblaResponse};

const PI: f64 = std::f64::consts::PI;
const EARTH_RADIUS_KM: f64 = 6371.0;
const EARTH_RADIUS_MILES: f64 = 3959.0;

// Kaaba coordinates (most precise available)
const KAABA_LATITUDE: f64 = 21.4224779;
const KAABA_LONGITUDE: f64 = 39.8251832;
const KAABA_ELEVATION: f64 = 333.0; // meters above sea level

pub struct QiblaCalculator {
    latitude: f64,
    longitude: f64,
    elevation: f64,
}

impl QiblaCalculator {
    pub fn new(latitude: f64, longitude: f64, elevation: f64) -> Self {
        Self {
            latitude,
            longitude,
            elevation,
        }
    }

    pub fn calculate_qibla_direction(&self) -> ApiResult<QiblaResponse> {
        let qibla_direction = self.calculate_bearing_to_kaaba()?;
        let distance_km = self.calculate_distance_to_kaaba()?;
        
        let location = LocationInfo {
            latitude: self.latitude,
            longitude: self.longitude,
            elevation: self.elevation,
            description: self.get_location_description(),
        };

        let kaaba_location = LocationInfo {
            latitude: KAABA_LATITUDE,
            longitude: KAABA_LONGITUDE,
            elevation: KAABA_ELEVATION,
            description: Some("Holy Kaaba, Masjid al-Haram, Mecca, Saudi Arabia".to_string()),
        };

        Ok(QiblaResponse {
            qibla_direction,
            qibla_direction_compass: self.degrees_to_compass(qibla_direction),
            distance_km,
            location,
            kaaba_location,
            calculation_method: "Great Circle Method (Haversine Formula)".to_string(),
            calculation_time: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn calculate_detailed_qibla(&self) -> ApiResult<QiblaDetailed> {
        let qibla_direction = self.calculate_bearing_to_kaaba()?;
        let bearing_from_kaaba = self.calculate_bearing_from_kaaba()?;
        let distance_km = self.calculate_distance_to_kaaba()?;
        let distance_miles = distance_km * 0.621371;
        
        let location = LocationInfo {
            latitude: self.latitude,
            longitude: self.longitude,
            elevation: self.elevation,
            description: self.get_location_description(),
        };

        let kaaba_location = LocationInfo {
            latitude: KAABA_LATITUDE,
            longitude: KAABA_LONGITUDE,
            elevation: KAABA_ELEVATION,
            description: Some("Holy Kaaba, Masjid al-Haram, Mecca, Saudi Arabia".to_string()),
        };

        let validation = self.validate_coordinates();

        Ok(QiblaDetailed {
            qibla_direction,
            qibla_direction_compass: self.degrees_to_compass(qibla_direction),
            distance_km,
            distance_miles,
            bearing_from_kaaba,
            bearing_from_kaaba_compass: self.degrees_to_compass(bearing_from_kaaba),
            location,
            kaaba_location,
            calculation_method: "Great Circle Method (Haversine Formula)".to_string(),
            calculation_time: chrono::Utc::now().to_rfc3339(),
            coordinates_validation: validation,
        })
    }

    fn calculate_bearing_to_kaaba(&self) -> ApiResult<f64> {
        let lat1 = self.degrees_to_radians(self.latitude);
        let lon1 = self.degrees_to_radians(self.longitude);
        let lat2 = self.degrees_to_radians(KAABA_LATITUDE);
        let lon2 = self.degrees_to_radians(KAABA_LONGITUDE);

        let dlon = lon2 - lon1;

        let y = sin(dlon) * cos(lat2);
        let x = cos(lat1) * sin(lat2) - sin(lat1) * cos(lat2) * cos(dlon);

        let bearing = atan2(y, x);
        let bearing_degrees = self.radians_to_degrees(bearing);

        // Normalize to 0-360 degrees
        let normalized = if bearing_degrees < 0.0 {
            bearing_degrees + 360.0
        } else {
            bearing_degrees
        };

        Ok(self.round_to_precision(normalized, 6))
    }

    fn calculate_bearing_from_kaaba(&self) -> ApiResult<f64> {
        let lat1 = self.degrees_to_radians(KAABA_LATITUDE);
        let lon1 = self.degrees_to_radians(KAABA_LONGITUDE);
        let lat2 = self.degrees_to_radians(self.latitude);
        let lon2 = self.degrees_to_radians(self.longitude);

        let dlon = lon2 - lon1;

        let y = sin(dlon) * cos(lat2);
        let x = cos(lat1) * sin(lat2) - sin(lat1) * cos(lat2) * cos(dlon);

        let bearing = atan2(y, x);
        let bearing_degrees = self.radians_to_degrees(bearing);

        // Normalize to 0-360 degrees
        let normalized = if bearing_degrees < 0.0 {
            bearing_degrees + 360.0
        } else {
            bearing_degrees
        };

        Ok(self.round_to_precision(normalized, 6))
    }

    fn calculate_distance_to_kaaba(&self) -> ApiResult<f64> {
        let lat1 = self.degrees_to_radians(self.latitude);
        let lon1 = self.degrees_to_radians(self.longitude);
        let lat2 = self.degrees_to_radians(KAABA_LATITUDE);
        let lon2 = self.degrees_to_radians(KAABA_LONGITUDE);

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = sin(dlat / 2.0) * sin(dlat / 2.0)
            + cos(lat1) * cos(lat2) * sin(dlon / 2.0) * sin(dlon / 2.0);
        let c = 2.0 * asin(sqrt(a));

        let distance = EARTH_RADIUS_KM * c;
        Ok(self.round_to_precision(distance, 2))
    }

    fn degrees_to_radians(&self, degrees: f64) -> f64 {
        degrees * PI / 180.0
    }

    fn radians_to_degrees(&self, radians: f64) -> f64 {
        radians * 180.0 / PI
    }

    fn round_to_precision(&self, value: f64, decimal_places: u32) -> f64 {
        let multiplier = 10_f64.powi(decimal_places as i32);
        (value * multiplier).round() / multiplier
    }

    fn degrees_to_compass(&self, degrees: f64) -> String {
        let directions = [
            "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
            "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"
        ];
        
        let index = ((degrees + 11.25) / 22.5) as usize % 16;
        format!("{} ({}°)", directions[index], self.round_to_precision(degrees, 1))
    }

    fn get_location_description(&self) -> Option<String> {
        // Basic location description based on coordinates
        let lat_dir = if self.latitude >= 0.0 { "N" } else { "S" };
        let lon_dir = if self.longitude >= 0.0 { "E" } else { "W" };
        
        Some(format!(
            "{:.4}°{}, {:.4}°{} (Elevation: {:.0}m)",
            self.latitude.abs(),
            lat_dir,
            self.longitude.abs(),
            lon_dir,
            self.elevation
        ))
    }

    fn validate_coordinates(&self) -> CoordinatesValidation {
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut is_valid = true;

        // Check if coordinates are at exactly 0,0 (often indicates missing data)
        if self.latitude == 0.0 && self.longitude == 0.0 {
            warnings.push("Coordinates are at 0°N, 0°E (Gulf of Guinea). Please verify this is correct.".to_string());
            suggestions.push("Double-check your GPS coordinates if this location seems incorrect.".to_string());
        }

        // Check if coordinates are in the ocean (basic check)
        if self.is_likely_ocean_coordinates() {
            warnings.push("Coordinates appear to be in an ocean area.".to_string());
            suggestions.push("Verify coordinates if you expected a land location.".to_string());
        }

        // Check elevation reasonableness
        if self.elevation < -500.0 {
            warnings.push("Elevation is unusually low (below sea level).".to_string());
        } else if self.elevation > 5000.0 {
            warnings.push("Elevation is very high. Make sure this is accurate for better calculation precision.".to_string());
        }

        // Check if very close to Kaaba
        let distance = self.calculate_distance_to_kaaba().unwrap_or(0.0);
        if distance < 1.0 {
            warnings.push("You are very close to the Kaaba. Qibla direction may not be meaningful at this distance.".to_string());
            suggestions.push("If you are in Masjid al-Haram, face towards the center of the Kaaba.".to_string());
        }

        CoordinatesValidation {
            is_valid,
            warnings,
            suggestions,
        }
    }

    fn is_likely_ocean_coordinates(&self) -> bool {
        // Very basic ocean detection - in a real implementation, you'd use a more sophisticated method
        // This is just a simple heuristic
        
        // Atlantic Ocean areas
        if self.latitude > -60.0 && self.latitude < 70.0 
            && self.longitude > -70.0 && self.longitude < 20.0 
            && !(self.latitude > 35.0 && self.longitude > -10.0) // Exclude Europe/Africa
            && !(self.latitude > 25.0 && self.longitude > -100.0 && self.longitude < -70.0) // Exclude North America east coast
        {
            return true;
        }

        // Pacific Ocean areas
        if self.latitude > -60.0 && self.latitude < 70.0 
            && ((self.longitude > 120.0 && self.longitude <= 180.0) 
                || (self.longitude >= -180.0 && self.longitude < -100.0))
            && !(self.latitude > 30.0 && self.longitude > 120.0 && self.longitude < 150.0) // Exclude East Asia
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qibla_calculation_new_york() {
        let calculator = QiblaCalculator::new(40.7128, -74.0060, 10.0);
        let result = calculator.calculate_qibla_direction().unwrap();
        
        // New York to Mecca should be roughly northeast (around 58 degrees)
        assert!(result.qibla_direction > 50.0 && result.qibla_direction < 70.0);
        assert!(result.distance_km > 11000.0 && result.distance_km < 12000.0);
    }

    #[test]
    fn test_qibla_calculation_london() {
        let calculator = QiblaCalculator::new(51.5074, -0.1278, 25.0);
        let result = calculator.calculate_qibla_direction().unwrap();
        
        // London to Mecca should be roughly southeast (around 120 degrees)
        assert!(result.qibla_direction > 100.0 && result.qibla_direction < 140.0);
        assert!(result.distance_km > 4500.0 && result.distance_km < 5500.0);
    }

    #[test]
    fn test_qibla_calculation_karachi() {
        let calculator = QiblaCalculator::new(24.8607, 67.0011, 8.0);
        let result = calculator.calculate_qibla_direction().unwrap();
        
        // Karachi to Mecca should be roughly west (around 270 degrees)
        assert!(result.qibla_direction > 250.0 && result.qibla_direction < 290.0);
        assert!(result.distance_km > 1200.0 && result.distance_km < 1400.0);
    }

    #[test]
    fn test_distance_calculation() {
        let calculator = QiblaCalculator::new(0.0, 0.0, 0.0);
        let distance = calculator.calculate_distance_to_kaaba().unwrap();
        
        // Distance from equator/prime meridian to Mecca
        assert!(distance > 2400.0 && distance < 2600.0);
    }

    #[test]
    fn test_compass_directions() {
        let calculator = QiblaCalculator::new(0.0, 0.0, 0.0);
        
        assert_eq!(calculator.degrees_to_compass(0.0), "N (0°)");
        assert_eq!(calculator.degrees_to_compass(90.0), "E (90°)");
        assert_eq!(calculator.degrees_to_compass(180.0), "S (180°)");
        assert_eq!(calculator.degrees_to_compass(270.0), "W (270°)");
        assert_eq!(calculator.degrees_to_compass(45.0), "NE (45°)");
    }
}