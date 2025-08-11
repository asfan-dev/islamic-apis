use serde::{Deserialize, Serialize};
use shared::validation::{validate_elevation, validate_latitude, validate_longitude};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct QiblaRequest {
    #[validate(custom = "validate_latitude_field")]
    pub latitude: f64,

    #[validate(custom = "validate_longitude_field")]
    pub longitude: f64,

    #[validate(custom = "validate_elevation_field")]
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

// Validation functions
fn validate_latitude_field(lat: &f64) -> Result<(), ValidationError> {
    validate_latitude(*lat).map_err(|_| ValidationError::new("invalid_latitude"))
}

fn validate_longitude_field(lng: &f64) -> Result<(), ValidationError> {
    validate_longitude(*lng).map_err(|_| ValidationError::new("invalid_longitude"))
}

fn validate_elevation_field(elevation: &Option<f64>) -> Result<(), ValidationError> {
    if let Some(elev) = elevation {
        validate_elevation(*elev).map_err(|_| ValidationError::new("invalid_elevation"))?;
    }
    Ok(())
}

impl QiblaRequest {
    pub fn to_coordinates(&self) -> (f64, f64, f64) {
        (self.latitude, self.longitude, self.elevation.unwrap_or(0.0))
    }
}
