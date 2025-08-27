use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct QiblaRequest {
    #[validate(range(min = -90.0, max = 90.0))]
    pub latitude: f64,

    #[validate(range(min = -180.0, max = 180.0))]
    pub longitude: f64,

    pub elevation: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct QiblaResponse {
    pub qibla_direction: f64,
    pub qibla_direction_compass: String,
    pub distance_km: f64,
    pub location: LocationInfo,
    pub kaaba_location: LocationInfo,
    pub calculation_method: String,
    pub calculation_time: String,
}

#[derive(Debug, Serialize)]
pub struct LocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QiblaDetailed {
    pub qibla_direction: f64,
    pub qibla_direction_compass: String,
    pub distance_km: f64,
    pub distance_miles: f64,
    pub bearing_from_kaaba: f64,
    pub bearing_from_kaaba_compass: String,
    pub location: LocationInfo,
    pub kaaba_location: LocationInfo,
    pub calculation_method: String,
    pub calculation_time: String,
    pub coordinates_validation: CoordinatesValidation,
}

#[derive(Debug, Serialize)]
pub struct CoordinatesValidation {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl QiblaRequest {
    pub fn to_coordinates(&self) -> (f64, f64, f64) {
        (self.latitude, self.longitude, self.elevation.unwrap_or(0.0))
    }
}