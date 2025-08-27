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

/// Main handler for prayer times requests.
/// It retrieves data from cache or calculates it and then caches the result.
pub async fn prayer_times_handler(
    Extension(cache): Extension<Cache>,
    Extension(preferred): Extension<Arc<PreferredMethodMap>>,
    Json(request): Json<PrayerTimesRequest>,
) -> ApiResult<Json<PrayerTimesResponse>> {
    info!(
        "Processing prayer times request for coordinates: {:.4}, {:.4}",
        request.latitude, request.longitude
    );
    debug!("Incoming request details: {:?}", request);

    // Validate the request
    debug!("Starting request validation.");
    request
        .validate()
        .map_err(|e| shared::error::ApiError::Validation(format!("Validation failed: {}", e)))?;
    debug!("Request validation successful.");

    // Create cache key for this request
    let cache_key = create_cache_key(&request);
    debug!("Generated cache key: {}", cache_key);

    // Try to get from cache first
    debug!("Checking cache for key: {}", cache_key);
    if let Ok(Some(cached_response)) = cache.get::<PrayerTimesResponse>(&cache_key).await {
        debug!("Cache hit. Returning cached prayer times for key: {}", cache_key);
        return Ok(Json(cached_response));
    }
    debug!("Cache miss. Calculating new prayer times.");

    // Parse timezone
    debug!("Parsing timezone from request: '{}'", request.timezone);
    let timezone = TimezoneParsing::parse_timezone(&request.timezone)?;
    debug!("Parsed timezone: {:?}", timezone);

    // Create coordinates
    let coordinates = Coordinates {
        latitude: request.latitude,
        longitude: request.longitude,
        elevation: request.elevation.unwrap_or(0.0),
    };
    debug!("Using coordinates: {:?}", coordinates);

    // Determine calculation method
    debug!("Determining calculation method.");
    let (method_settings, standard_method) = determine_method(&request, &preferred)?;
    debug!("Determined method: {:?}, with settings: {:?}", standard_method, method_settings);

    // Get timespan - clone to avoid move
    let timespan = request.timespan.clone().unwrap_or_default();
    debug!("Parsing timespan: {:?}", timespan);
    let (start_date, day_count) = parse_timespan(timespan.clone(), timezone)?;
    debug!("Timespan parsed. Start date: {:?}, Day count: {}", start_date, day_count);

    // Validate day count
    if day_count > 366 {
        debug!("Day count exceeds limit: {}", day_count);
        return Err(shared::error::ApiError::InvalidInput(
            "Day count cannot exceed 366 days".to_string(),
        ));
    }

    // Get adjustments - clone to avoid move
    let adjustments = request.adjustments.clone().unwrap_or_default();
    debug!("Using adjustments: {:?}", adjustments);

    // Create calculator
    let calculator =
        PrayerCalculator::new(coordinates, method_settings.clone(), adjustments.clone());

    // Calculate prayer times for all requested days
    debug!("Starting calculation loop for {} days.", day_count);
    let mut prayers = Vec::new();
    for i in 0..day_count {
        let current_date = start_date + Duration::days(i as i64);
        debug!("Calculating prayer times for date: {:?}", current_date);
        let prayer_times = calculator.calculate_prayer_times(current_date)?;
        prayers.push(prayer_times);
    }
    debug!("Prayer times calculation loop finished. Calculated {} days.", prayers.len());

    // Calculate next prayer if applicable
    debug!("Calculating next prayer based on timespan: {:?}", timespan);
    let next_prayer = calculate_next_prayer(&timespan, start_date, &prayers);
    debug!("Next prayer result: {:?}", next_prayer);

    // Calculate qibla direction
    debug!("Calculating Qibla direction.");
    let qibla_direction = calculator.calculate_qibla_direction();
    debug!("Qibla direction: {}", qibla_direction);

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
        debug!("Timespan is DaysFromToday(1), removing the extra calculated day.");
        prayers.pop();
    }
    
    let response = PrayerTimesResponse {
        qibla_direction,
        next: next_prayer,
        prayers,
        meta,
    };

    // Cache the response for 1 hour
    debug!("Caching response for key: {} for 1 hour.", cache_key);
    if let Err(e) = cache
        .set(&cache_key, &response, Some(StdDuration::from_secs(3600)))
        .await
    {
        tracing::warn!("Failed to cache prayer times response: {}", e);
    }
    debug!("Response successfully cached.");

    info!(
        "Successfully calculated prayer times for {} days",
        day_count
    );
    Ok(Json(response))
}

/// Creates a unique cache key based on the request parameters.
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

    let key = format!("prayer_times:{:x}", hasher.finish());
    debug!("Generated cache key: {}", key);
    key
}

/// Determines the prayer calculation method from the request.
fn determine_method(
    request: &PrayerTimesRequest,
    preferred: &PreferredMethodMap,
) -> ApiResult<(crate::models::MethodSettings, Option<StandardMethod>)> {
    if let Some(ref custom) = request.custom {
        debug!("Using custom method from request.");
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
            debug!("Overriding high_lat with request value: {:?}", high_lat);
            settings.high_lat = Some(high_lat);
        }
        if let Some(school) = request.school {
            debug!("Overriding school with request value: {:?}", school);
            settings.school = school;
        }

        debug!("Custom method settings after overrides: {:?}", settings);
        return Ok((settings, None));
    }

    // Use standard method
    let standard_method = if let Some(method) = request.method {
        debug!("Using standard method from request: {:?}", method);
        method
    } else if let Some(ref country) = request.country {
        debug!("No method provided, looking up preferred method for country: {}", country);
        let preferred_method = preferred.get(country)?;
        debug!("Found preferred method for country: {:?}", preferred_method);
        preferred_method
    } else {
        debug!("No method, custom method, or country provided.");
        return Err(shared::error::ApiError::InvalidInput(
            "No method, custom method, or country provided".to_string(),
        ));
    };

    let mut settings = standard_method.to_method_settings();
    debug!("Base settings for standard method: {:?}", settings);

    // Apply request-level overrides
    if let Some(high_lat) = request.high_lat {
        debug!("Overriding high_lat with request value: {:?}", high_lat);
        settings.high_lat = Some(high_lat);
    }
    if let Some(school) = request.school {
        debug!("Overriding school with request value: {:?}", school);
        settings.school = school;
    }
    
    debug!("Final settings for standard method: {:?}", settings);
    Ok((settings, Some(standard_method)))
}

/// Parses a string into either a minute value or an angle value.
fn parse_minute_or_angle(value: &str) -> ApiResult<crate::models::MinuteOrAngle> {
    debug!("Parsing minute or angle from string: '{}'", value);
    if value.ends_with("min") {
        let minute_str = value.split_whitespace().next().unwrap_or("0");
        let minute: f64 = minute_str.parse().map_err(|_| {
            shared::error::ApiError::InvalidInput(format!("Invalid minute value: {}", value))
        })?;
        debug!("Parsed as Minute: {}", minute);
        Ok(crate::models::MinuteOrAngle::Minute { minute })
    } else {
        let angle: f64 = value.parse().map_err(|_| {
            shared::error::ApiError::InvalidInput(format!("Invalid angle value: {}", value))
        })?;
        debug!("Parsed as Angle: {}", angle);
        Ok(crate::models::MinuteOrAngle::Angle { angle })
    }
}

/// Parses the timespan from the request to get a start date and day count.
fn parse_timespan(
    timespan: Timespan,
    timezone: FixedOffset,
) -> ApiResult<(DateTime<FixedOffset>, u16)> {
    debug!("Parsing timespan: {:?}", timespan);
    match timespan {
        Timespan::DaysFromToday(days) => {
            debug!("Timespan is DaysFromToday({})", days);
            let start = Utc::now().with_timezone(&timezone);
            // For single day, we calculate 2 days to get next prayer correct, then remove the extra
            let day_count = if days == 1 { 2 } else { days };
            debug!("Calculated start date: {:?}, day count: {}", start, day_count);
            Ok((start, day_count))
        }
        Timespan::DaysFromDate(date_str, days) => {
            debug!("Timespan is DaysFromDate({}, {})", date_str, days);
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
            debug!("Calculated start date: {:?}, day count: {}", start, days);
            Ok((start, days))
        }
        Timespan::Month(month_name, year) => {
            debug!("Timespan is Month({}, {})", month_name, year);
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
            debug!("Calculated start date: {:?}, days in month: {}", start, days_in_month);
            Ok((start, days_in_month))
        }
        Timespan::GregorianYear(year) => {
            debug!("Timespan is GregorianYear({})", year);
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
            debug!("Calculated start date: {:?}, days in year: {}", start, days);
            Ok((start, days))
        }
        Timespan::HijriYear(hijri_year) => {
            debug!("Timespan is HijriYear({})", hijri_year);
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
            debug!("Calculated start date: {:?}, days in year: 355", start);
            Ok((start, 355))
        }
    }
}

/// Helper function to parse a month name string into its corresponding number.
fn parse_month_name(month_name: &str) -> ApiResult<u32> {
    debug!("Parsing month name: '{}'", month_name);
    let month_num = match month_name.to_lowercase().as_str() {
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
    }?;
    debug!("Parsed month name to number: {}", month_num);
    Ok(month_num)
}

/// Helper function to get the number of days in a given month and year.
fn days_in_month(year: i32, month: u32) -> ApiResult<u16> {
    debug!("Getting days in month for year: {}, month: {}", year, month);
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
    debug!("Days in month: {}", days);
    Ok(days)
}

/// Checks if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Calculates the next upcoming prayer time based on the current time.
fn calculate_next_prayer(
    timespan: &Timespan,
    start_date: DateTime<FixedOffset>,
    prayers: &[crate::models::PrayerTimes],
) -> Option<NextPrayer> {
    debug!("Attempting to calculate next prayer time.");
    match timespan {
        Timespan::DaysFromToday(_) => {
            let now = start_date;
            debug!("Current time for comparison: {:?}", now);

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
                            debug!("Found next prayer: {} at {}", name, time_str);
                            return Some(NextPrayer {
                                name: name.to_string(),
                                time: time_str.to_string(), // Convert &String to String
                            });
                        }
                    }
                }
            }
            debug!("No upcoming prayer found for today or tomorrow.");
            None
        }
        _ => {
            debug!("Not calculating next prayer for timespan type: {:?}", timespan);
            None
        }, // Only calculate next prayer for "days from today"
    }
}
