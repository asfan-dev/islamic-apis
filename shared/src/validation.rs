use chrono_tz::Tz;
use validator::{Validate, ValidationErrors};

use crate::error::ApiError;

pub trait ValidatedJson<T> {
    fn validate_json(self) -> Result<T, ApiError>;
}

impl<T> ValidatedJson<T> for Result<T, serde_json::Error>
where
    T: Validate,
{
    fn validate_json(self) -> Result<T, ApiError> {
        let data = self?;
        data.validate()
            .map_err(|e| ApiError::Validation(format_validation_errors(e)))?;
        Ok(data)
    }
}

pub fn validate_latitude(lat: f64) -> Result<(), ApiError> {
    if lat < -90.0 || lat > 90.0 {
        return Err(ApiError::Validation(
            "Latitude must be between -90 and 90 degrees".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_longitude(lng: f64) -> Result<(), ApiError> {
    if lng < -180.0 || lng > 180.0 {
        return Err(ApiError::Validation(
            "Longitude must be between -180 and 180 degrees".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_elevation(elevation: f64) -> Result<(), ApiError> {
    if elevation < -500.0 || elevation > 10000.0 {
        return Err(ApiError::Validation(
            "Elevation must be between -500 and 10000 meters".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_timezone(timezone: &str) -> Result<(), ApiError> {
    // Try to parse as offset first
    if timezone.starts_with('+') || timezone.starts_with('-') {
        return parse_timezone_offset(timezone).map(|_| ());
    }

    // Try to parse as timezone name
    timezone
        .parse::<Tz>()
        .map_err(|_| ApiError::TimezoneParsing(format!("Invalid timezone: {}", timezone)))?;

    Ok(())
}

fn parse_timezone_offset(timezone: &str) -> Result<chrono::FixedOffset, ApiError> {
    let mut parts = timezone[1..].split(':');
    let hours: i32 = parts
        .next()
        .ok_or_else(|| ApiError::TimezoneParsing("Missing hours in timezone offset".to_string()))?
        .parse()
        .map_err(|_| ApiError::TimezoneParsing("Invalid hours in timezone offset".to_string()))?;

    let minutes: i32 =
        parts.next().unwrap_or("0").parse().map_err(|_| {
            ApiError::TimezoneParsing("Invalid minutes in timezone offset".to_string())
        })?;

    if hours < -12 || hours > 14 {
        return Err(ApiError::TimezoneParsing(
            "Timezone offset hours must be between -12 and +14".to_string(),
        ));
    }

    if minutes < 0 || minutes > 59 {
        return Err(ApiError::TimezoneParsing(
            "Timezone offset minutes must be between 0 and 59".to_string(),
        ));
    }

    let total_seconds =
        (hours * 3600 + minutes * 60) * if timezone.starts_with('-') { -1 } else { 1 };

    chrono::FixedOffset::east_opt(total_seconds)
        .ok_or_else(|| ApiError::TimezoneParsing("Invalid timezone offset".to_string()))
}

fn format_validation_errors(errors: ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .map(|(field, errors)| {
            let messages: Vec<String> = errors
                .iter()
                .map(|error| {
                    error
                        .message
                        .as_ref()
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| format!("Invalid value for field '{}'", field))
                })
                .collect();
            format!("{}: {}", field, messages.join(", "))
        })
        .collect::<Vec<String>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_latitude() {
        assert!(validate_latitude(0.0).is_ok());
        assert!(validate_latitude(90.0).is_ok());
        assert!(validate_latitude(-90.0).is_ok());
        assert!(validate_latitude(45.5).is_ok());

        assert!(validate_latitude(91.0).is_err());
        assert!(validate_latitude(-91.0).is_err());
    }

    #[test]
    fn test_validate_longitude() {
        assert!(validate_longitude(0.0).is_ok());
        assert!(validate_longitude(180.0).is_ok());
        assert!(validate_longitude(-180.0).is_ok());
        assert!(validate_longitude(123.456).is_ok());

        assert!(validate_longitude(181.0).is_err());
        assert!(validate_longitude(-181.0).is_err());
    }

    #[test]
    fn test_validate_timezone() {
        assert!(validate_timezone("UTC").is_ok());
        assert!(validate_timezone("America/New_York").is_ok());
        assert!(validate_timezone("Europe/London").is_ok());
        assert!(validate_timezone("+05:00").is_ok());
        assert!(validate_timezone("-08:30").is_ok());

        assert!(validate_timezone("Invalid/Timezone").is_err());
        assert!(validate_timezone("+25:00").is_err());
        assert!(validate_timezone("-15:00").is_err());
    }
}
