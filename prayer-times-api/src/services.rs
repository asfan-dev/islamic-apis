use chrono::Offset;
use chrono::{FixedOffset, TimeZone, Utc};
use chrono_tz::Tz;
use shared::error::{ApiError, ApiResult};
pub struct TimezoneParsing;

impl TimezoneParsing {
    pub fn parse_timezone(timezone: &str) -> ApiResult<FixedOffset> {
        let timezone = timezone.trim();

        if timezone.starts_with('+') || timezone.starts_with('-') {
            Self::parse_timezone_offset(timezone)
        } else {
            Self::parse_timezone_name(timezone)
        }
    }

    fn parse_timezone_name(timezone: &str) -> ApiResult<FixedOffset> {
        let tz: Tz = timezone.parse().map_err(|_| {
            ApiError::TimezoneParsing(format!("Invalid timezone name: {}", timezone))
        })?;

        let offset = tz.offset_from_utc_datetime(&Utc::now().naive_utc());
        Ok(offset.fix())
    }

    fn parse_timezone_offset(timezone: &str) -> ApiResult<FixedOffset> {
        let mut parts = timezone[1..].split(':');

        let hours: i32 = parts
            .next()
            .ok_or_else(|| {
                ApiError::TimezoneParsing("Missing hours in timezone offset".to_string())
            })?
            .parse()
            .map_err(|_| {
                ApiError::TimezoneParsing("Invalid hours in timezone offset".to_string())
            })?;

        let minutes: i32 = parts.next().unwrap_or("0").parse().map_err(|_| {
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

        FixedOffset::east_opt(total_seconds)
            .ok_or_else(|| ApiError::TimezoneParsing("Invalid timezone offset".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timezone_offset() {
        assert!(TimezoneParsing::parse_timezone("+05:00").is_ok());
        assert!(TimezoneParsing::parse_timezone("-08:30").is_ok());
        assert!(TimezoneParsing::parse_timezone("+00:00").is_ok());

        assert!(TimezoneParsing::parse_timezone("+25:00").is_err());
        assert!(TimezoneParsing::parse_timezone("-15:00").is_err());
        assert!(TimezoneParsing::parse_timezone("+05:60").is_err());
    }

    #[test]
    fn test_parse_timezone_name() {
        assert!(TimezoneParsing::parse_timezone("UTC").is_ok());
        assert!(TimezoneParsing::parse_timezone("America/New_York").is_ok());
        assert!(TimezoneParsing::parse_timezone("Europe/London").is_ok());
        assert!(TimezoneParsing::parse_timezone("Asia/Karachi").is_ok());

        assert!(TimezoneParsing::parse_timezone("Invalid/Timezone").is_err());
    }
}
