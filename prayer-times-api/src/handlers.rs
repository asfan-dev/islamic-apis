use axum::{Extension, Json};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use shared::{cache::Cache, error::ApiResult};
use std::{sync::Arc, time::Duration as StdDuration};
use tracing::{debug, info};
use validator::Validate;

use crate::{
    calculations::PrayerCalculator,
    models::{
        Coordinates, MetaData, NextPrayer, PrayerTimesRequest, PrayerTimesResponse, StandardMethod,
        Timespan,
    },
    preferred::PreferredMethodMap,
    services::TimezoneParsing,
};

pub async fn prayer_times_handler(
    Extension(cache): Extension<Cache>,
    Extension(preferred): Extension<Arc<PreferredMethodMap>>,
    Json(request): Json<PrayerTimesRequest>,
) -> ApiResult<Json<PrayerTimesResponse>> {
    info!(
        "Processing prayer times request for coordinates: {:.4}, {:.4}",
        request.latitude, request.longitude
    );

    // Validate the request
    request
        .validate()
        .map_err(|e| shared::error::ApiError::Validation(format!("Validation failed: {}", e)))?;

    // Create cache key for this request
    let cache_key = create_cache_key(&request);

    // Try to get from cache first
    if let Ok(Some(cached_response)) = cache.get::<PrayerTimesResponse>(&cache_key).await {
        debug!("Returning cached prayer times for key: {}", cache_key);
        return Ok(Json(cached_response));
    }

    // Parse timezone
    let timezone = TimezoneParsing::parse_timezone(&request.timezone)?;

    // Create coordinates
    let coordinates = Coordinates {
        latitude: request.latitude,
        longitude: request.longitude,
        elevation: request.elevation.unwrap_or(0.0),
    };

    // Determine calculation method
    let (method_settings, standard_method) = determine_method(&request, &preferred)?;

    // Get timespan - clone to avoid move
    let timespan = request.timespan.clone().unwrap_or_default();
    let (start_date, day_count) = parse_timespan(timespan.clone(), timezone)?;

    // Validate day count
    if day_count > 366 {
        return Err(shared::error::ApiError::InvalidInput(
            "Day count cannot exceed 366 days".to_string(),
        ));
    }

    // Get adjustments - clone to avoid move
    let adjustments = request.adjustments.clone().unwrap_or_default();

    // Create calculator
    let calculator =
        PrayerCalculator::new(coordinates, method_settings.clone(), adjustments.clone());

    // Calculate prayer times for all requested days
    let mut prayers = Vec::new();
    for i in 0..day_count {
        let current_date = start_date + Duration::days(i as i64);
        let prayer_times = calculator.calculate_prayer_times(current_date)?;
        prayers.push(prayer_times);
    }

    // Calculate next prayer if applicable
    let next_prayer = calculate_next_prayer(&timespan, start_date, &prayers);

    // Calculate qibla direction
    let qibla_direction = calculator.calculate_qibla_direction();

    // Create metadata
    let meta = MetaData {
        method: standard_method,
        settings: method_settings,
        timezone: request.timezone.clone(),
        adjustments: Some(adjustments),
        coordinates,
        calculation_time: Utc::now().to_rfc3339(),
    };

    // Remove extra day for DaysFromToday(1) case
    if let Timespan::DaysFromToday(1) = timespan {
        prayers.pop();
    }

    let response = PrayerTimesResponse {
        qibla_direction,
        next: next_prayer,
        prayers,
        meta,
    };

    // Cache the response for 1 hour
    if let Err(e) = cache
        .set(&cache_key, &response, Some(StdDuration::from_secs(3600)))
        .await
    {
        tracing::warn!("Failed to cache prayer times response: {}", e);
    }

    info!(
        "Successfully calculated prayer times for {} days",
        day_count
    );
    Ok(Json(response))
}

fn create_cache_key(request: &PrayerTimesRequest) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash the main parameters that affect calculation
    request.latitude.to_bits().hash(&mut hasher);
    request.longitude.to_bits().hash(&mut hasher);
    request.elevation.unwrap_or(0.0).to_bits().hash(&mut hasher);
    request.method.hash(&mut hasher);
    request.country.hash(&mut hasher);
    request.timezone.hash(&mut hasher);
    request.high_lat.hash(&mut hasher);
    request.school.hash(&mut hasher);

    // For custom method, we need to hash the settings
    if let Some(ref custom) = request.custom {
        if let Ok(json_str) = serde_json::to_string(custom) {
            json_str.hash(&mut hasher);
        }
    }

    // Hash adjustments
    if let Some(ref adj) = request.adjustments {
        if let Ok(json_str) = serde_json::to_string(adj) {
            json_str.hash(&mut hasher);
        }
    }

    // Hash timespan
    if let Some(ref timespan) = request.timespan {
        if let Ok(json_str) = serde_json::to_string(timespan) {
            json_str.hash(&mut hasher);
        }
    }

    format!("prayer_times:{:x}", hasher.finish())
}

fn determine_method(
    request: &PrayerTimesRequest,
    preferred: &PreferredMethodMap,
) -> ApiResult<(crate::models::MethodSettings, Option<StandardMethod>)> {
    if let Some(ref custom) = request.custom {
        // Use custom method
        let mut settings = crate::models::MethodSettings {
            fajr: custom.fajr.unwrap_or(18.0),
            isha: parse_minute_or_angle(&custom.isha.as_ref().unwrap_or(&"18.0".to_string()))?,
            midnight: custom.midnight.unwrap_or_default(),
            maghrib: parse_minute_or_angle(
                &custom.maghrib.as_ref().unwrap_or(&"0 min".to_string()),
            )?,
            imsak: parse_minute_or_angle(&custom.imsak.as_ref().unwrap_or(&"10 min".to_string()))?,
            dhuhr: custom.dhuhr.unwrap_or(0.0),
            shafaq: custom.shafaq,
            school: custom.school.unwrap_or_default(),
            high_lat: custom.high_lat,
        };

        // Override with request-level settings
        if let Some(high_lat) = request.high_lat {
            settings.high_lat = Some(high_lat);
        }
        if let Some(school) = request.school {
            settings.school = school;
        }

        return Ok((settings, None));
    }

    // Use standard method
    let standard_method = if let Some(method) = request.method {
        method
    } else if let Some(ref country) = request.country {
        preferred.get(country)?
    } else {
        return Err(shared::error::ApiError::InvalidInput(
            "No method, custom method, or country provided".to_string(),
        ));
    };

    let mut settings = standard_method.to_method_settings();

    // Apply request-level overrides
    if let Some(high_lat) = request.high_lat {
        settings.high_lat = Some(high_lat);
    }
    if let Some(school) = request.school {
        settings.school = school;
    }

    Ok((settings, Some(standard_method)))
}

fn parse_minute_or_angle(value: &str) -> ApiResult<crate::models::MinuteOrAngle> {
    if value.ends_with("min") {
        let minute_str = value.split_whitespace().next().unwrap_or("0");
        let minute: f64 = minute_str.parse().map_err(|_| {
            shared::error::ApiError::InvalidInput(format!("Invalid minute value: {}", value))
        })?;
        Ok(crate::models::MinuteOrAngle::Minute { minute })
    } else {
        let angle: f64 = value.parse().map_err(|_| {
            shared::error::ApiError::InvalidInput(format!("Invalid angle value: {}", value))
        })?;
        Ok(crate::models::MinuteOrAngle::Angle { angle })
    }
}

fn parse_timespan(
    timespan: Timespan,
    timezone: FixedOffset,
) -> ApiResult<(DateTime<FixedOffset>, u16)> {
    match timespan {
        Timespan::DaysFromToday(days) => {
            let start = Utc::now().with_timezone(&timezone);
            // For single day, we calculate 2 days to get next prayer correct, then remove the extra
            let day_count = if days == 1 { 2 } else { days };
            Ok((start, day_count))
        }
        Timespan::DaysFromDate(date_str, days) => {
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%d/%m/%Y").map_err(|_| {
                shared::error::ApiError::DateParsing(format!(
                    "Invalid date format, expected DD/MM/YYYY: {}",
                    date_str
                ))
            })?;
            let start = date
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_local_timezone(timezone)
                .single()
                .ok_or_else(|| {
                    shared::error::ApiError::DateParsing(
                        "Failed to create datetime with timezone".to_string(),
                    )
                })?;
            Ok((start, days))
        }
        Timespan::Month(month_name, year) => {
            let month = parse_month_name(&month_name)?;
            let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or_else(|| {
                    shared::error::ApiError::DateParsing(format!(
                        "Invalid month/year: {}/{}",
                        month_name, year
                    ))
                })?
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_local_timezone(timezone)
                .single()
                .ok_or_else(|| {
                    shared::error::ApiError::DateParsing(
                        "Failed to create datetime with timezone".to_string(),
                    )
                })?;

            let days_in_month = days_in_month(year, month)?;
            Ok((start, days_in_month))
        }
        Timespan::GregorianYear(year) => {
            let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .ok_or_else(|| {
                    shared::error::ApiError::DateParsing(format!("Invalid year: {}", year))
                })?
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_local_timezone(timezone)
                .single()
                .ok_or_else(|| {
                    shared::error::ApiError::DateParsing(
                        "Failed to create datetime with timezone".to_string(),
                    )
                })?;

            let days = if is_leap_year(year) { 366 } else { 365 };
            Ok((start, days))
        }
        Timespan::HijriYear(hijri_year) => {
            let hijri_start = hijri_date::HijriDate::from_hijri(hijri_year as usize, 1, 1)
                .map_err(|e| {
                    shared::error::ApiError::DateParsing(format!("Invalid Hijri year: {}", e))
                })?;

            let start = chrono::NaiveDate::from_ymd_opt(
                hijri_start.year_gr() as i32,
                hijri_start.month_gr() as u32,
                hijri_start.day_gr() as u32,
            )
            .ok_or_else(|| {
                shared::error::ApiError::DateParsing(
                    "Failed to convert Hijri date to Gregorian".to_string(),
                )
            })?
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_local_timezone(timezone)
            .single()
            .ok_or_else(|| {
                shared::error::ApiError::DateParsing(
                    "Failed to create datetime with timezone".to_string(),
                )
            })?;

            // Hijri years are typically 354 or 355 days
            Ok((start, 355))
        }
    }
}

fn parse_month_name(month_name: &str) -> ApiResult<u32> {
    match month_name.to_lowercase().as_str() {
        "january" | "jan" => Ok(1),
        "february" | "feb" => Ok(2),
        "march" | "mar" => Ok(3),
        "april" | "apr" => Ok(4),
        "may" => Ok(5),
        "june" | "jun" => Ok(6),
        "july" | "jul" => Ok(7),
        "august" | "aug" => Ok(8),
        "september" | "sep" => Ok(9),
        "october" | "oct" => Ok(10),
        "november" | "nov" => Ok(11),
        "december" | "dec" => Ok(12),
        _ => Err(shared::error::ApiError::InvalidInput(format!(
            "Invalid month name: {}",
            month_name
        ))),
    }
}

fn days_in_month(year: i32, month: u32) -> ApiResult<u16> {
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => {
            return Err(shared::error::ApiError::InvalidInput(format!(
                "Invalid month: {}",
                month
            )))
        }
    };
    Ok(days)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn calculate_next_prayer(
    timespan: &Timespan,
    start_date: DateTime<FixedOffset>,
    prayers: &[crate::models::PrayerTimes],
) -> Option<NextPrayer> {
    match timespan {
        Timespan::DaysFromToday(_) => {
            let now = start_date;

            for prayer in prayers {
                let prayer_times = [
                    ("Imsak", &prayer.imsak),
                    ("Fajr", &prayer.fajr),
                    ("Dhuhr", &prayer.dhuhr),
                    ("Asr", &prayer.asr),
                    ("Maghrib", &prayer.maghrib),
                    ("Isha", &prayer.isha),
                ];

                for (name, time_str) in &prayer_times {
                    if let Ok(prayer_time) =
                        chrono::DateTime::parse_from_str(time_str, "%d/%m/%Y %H:%M")
                    {
                        if prayer_time > now {
                            return Some(NextPrayer {
                                name: name.to_string(),
                                time: time_str.to_string(), // Convert &String to String
                            });
                        }
                    }
                }
            }
            None
        }
        _ => None, // Only calculate next prayer for "days from today"
    }
}
