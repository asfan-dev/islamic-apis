use chrono::{DateTime, FixedOffset, NaiveDate};
use hijri_date::HijriDate;
use serde::{Deserialize, Serialize};
use shared::validation::{
    validate_elevation, validate_latitude, validate_longitude, validate_timezone,
};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StandardMethod {
    Jafari,
    Karachi,
    Isna,
    Mwl,
    Makkah,
    Egypt,
    Tehran,
    Gulf,
    Kuwait,
    Qatar,
    Singapore,
    France,
    Turkey,
    Russia,
    Moonsighting,
    Dubai,
    Uoif,    // Union of Islamic Organizations of France
    Diyanet, // Turkey Presidency of Religious Affairs
    Jakim,   // Malaysia
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum HighLatitudeRule {
    NightMiddle,
    AngleBased,
    OneSeventh,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum School {
    Standard,
    Hanafi,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Midnight {
    Standard,
    Jafari,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shafaq {
    General,
    Ahmer,
    Abyad,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomMethod {
    pub fajr: Option<f64>,
    pub isha: Option<String>,
    pub midnight: Option<Midnight>,
    pub maghrib: Option<String>,
    pub imsak: Option<String>,
    pub dhuhr: Option<f64>,
    pub shafaq: Option<Shafaq>,
    pub school: Option<School>,
    pub high_lat: Option<HighLatitudeRule>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Adjustments {
    pub imsak: i8,
    pub fajr: i8,
    pub sunrise: i8,
    pub dhuhr: i8,
    pub asr: i8,
    pub sunset: i8,
    pub maghrib: i8,
    pub isha: i8,
    pub midnight: i8,
    pub first_third: i8,
    pub last_third: i8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Timespan {
    DaysFromToday(u16),
    DaysFromDate(String, u16), // Date in DD/MM/YYYY format
    Month(String, i32),        // Month name and year
    GregorianYear(i32),
    HijriYear(i32),
}

#[derive(Debug, Deserialize, Validate)]
pub struct PrayerTimesRequest {
    #[validate(custom = "validate_latitude_field")]
    pub latitude: f64,

    #[validate(custom = "validate_longitude_field")]
    pub longitude: f64,

    pub method: Option<StandardMethod>,
    pub custom: Option<CustomMethod>,
    pub country: Option<String>,

    #[validate(custom = "validate_timezone_field")]
    pub timezone: String,

    pub timespan: Option<Timespan>,

    #[validate(custom = "validate_elevation_field")]
    pub elevation: Option<f64>,

    pub adjustments: Option<Adjustments>,
    pub high_lat: Option<HighLatitudeRule>,
    pub school: Option<School>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrayerTimesResponse {
    pub qibla_direction: f64,
    pub next: Option<NextPrayer>,
    pub prayers: Vec<PrayerTimes>,
    pub meta: MetaData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NextPrayer {
    pub name: String, // Changed from &'static str to String to avoid lifetime issues
    pub time: String, // Formatted as DD/MM/YYYY HH:MM
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrayerTimes {
    pub imsak: String,
    pub fajr: String,
    pub sunrise: String,
    pub dhuhr: String,
    pub asr: String,
    pub sunset: String,
    pub maghrib: String,
    pub isha: String,
    pub midnight: String,
    pub first_third: String,
    pub last_third: String,
    pub date: String,  // DD/MM/YYYY
    pub hijri: String, // DD/MM/YYYY (Hijri)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaData {
    pub method: Option<StandardMethod>,
    pub settings: MethodSettings,
    pub timezone: String,
    pub adjustments: Option<Adjustments>,
    pub coordinates: Coordinates,
    pub calculation_time: String, // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSettings {
    pub fajr: f64,
    pub isha: MinuteOrAngle,
    pub midnight: Midnight,
    pub maghrib: MinuteOrAngle,
    pub imsak: MinuteOrAngle,
    pub dhuhr: f64,
    pub shafaq: Option<Shafaq>,
    pub school: School,
    pub high_lat: Option<HighLatitudeRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinuteOrAngle {
    Minute { minute: f64 },
    Angle { angle: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
}

// Validation functions - corrected signatures to match validator expectations
fn validate_latitude_field(lat: f64) -> Result<(), ValidationError> {
    validate_latitude(lat).map_err(|_| ValidationError::new("invalid_latitude"))
}

fn validate_longitude_field(lng: f64) -> Result<(), ValidationError> {
    validate_longitude(lng).map_err(|_| ValidationError::new("invalid_longitude"))
}

fn validate_elevation_field(elevation: f64) -> Result<(), ValidationError> {
    validate_elevation(elevation).map_err(|_| ValidationError::new("invalid_elevation"))
}

fn validate_timezone_field(timezone: &str) -> Result<(), ValidationError> {
    validate_timezone(timezone).map_err(|_| ValidationError::new("invalid_timezone"))
}

impl Default for Timespan {
    fn default() -> Self {
        Timespan::DaysFromToday(1)
    }
}

impl Default for School {
    fn default() -> Self {
        School::Standard
    }
}

impl Default for Midnight {
    fn default() -> Self {
        Midnight::Standard
    }
}
