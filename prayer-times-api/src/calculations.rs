use chrono::{DateTime, Offset, Datelike, FixedOffset, NaiveDate};
use hijri_date::HijriDate;
use libm::{acos, asin, atan, atan2, cos, floor, sin, sqrt, tan};
use shared::error::{ApiError, ApiResult};
use tracing::{debug, info, warn};

use crate::models::{
    Adjustments, Coordinates, HighLatitudeRule, MethodSettings, Midnight, MinuteOrAngle, School,
    StandardMethod,
};

const PI: f64 = std::f64::consts::PI;

// Math utility functions
/// Converts degrees to radians.
fn dtr(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// Converts radians to degrees.
fn rtd(radians: f64) -> f64 {
    radians * 180.0 / PI
}

/// Normalizes an angle to be within [0, 360).
fn fix_angle(angle: f64) -> f64 {
    fix(angle, 360.0)
}

/// Normalizes a time value to be within [0, 24).
fn fix_hour(hour: f64) -> f64 {
    fix(hour, 24.0)
}

/// Generic normalization function.
fn fix(a: f64, b: f64) -> f64 {
    let a = a - b * floor(a / b);
    if a < 0.0 {
        a + b
    } else {
        a
    }
}

pub struct PrayerCalculator {
    coordinates: Coordinates,
    method_settings: MethodSettings,
    adjustments: Adjustments,
}

impl PrayerCalculator {
    pub fn new(
        coordinates: Coordinates,
        method_settings: MethodSettings,
        adjustments: Adjustments,
    ) -> Self {
        Self {
            coordinates,
            method_settings,
            adjustments,
        }
    }

    pub fn calculate_prayer_times(
        &self,
        date: DateTime<FixedOffset>,
    ) -> ApiResult<super::models::PrayerTimes> {
        info!("Starting prayer time calculation for date: {}", date);
        
        // Compute the raw floating-point prayer times
        let times = self.compute_times(date)?;
        
        debug!("Raw prayer times (floating point):");
        debug!("  Imsak: {}", times.imsak);
        debug!("  Fajr: {}", times.fajr);
        debug!("  Sunrise: {}", times.sunrise);
        debug!("  Dhuhr: {}", times.dhuhr);
        debug!("  Asr: {}", times.asr);
        debug!("  Sunset: {}", times.sunset);
        debug!("  Maghrib: {}", times.maghrib);
        debug!("  Isha: {}", times.isha);
        debug!("  Midnight: {}", times.midnight);
        debug!("  First Third: {}", times.first_third);
        debug!("  Last Third: {}", times.last_third);
        
        let hijri = self.calculate_hijri_date(date.date_naive())?;

        Ok(super::models::PrayerTimes {
            imsak: self.format_time(times.imsak, date, self.adjustments.imsak),
            fajr: self.format_time(times.fajr, date, self.adjustments.fajr),
            sunrise: self.format_time(times.sunrise, date, self.adjustments.sunrise),
            dhuhr: self.format_time(times.dhuhr, date, self.adjustments.dhuhr),
            asr: self.format_time(times.asr, date, self.adjustments.asr),
            sunset: self.format_time(times.sunset, date, self.adjustments.sunset),
            maghrib: self.format_time(times.maghrib, date, self.adjustments.maghrib),
            isha: self.format_time(times.isha, date, self.adjustments.isha),
            midnight: self.format_time(times.midnight, date, self.adjustments.midnight),
            first_third: self.format_time(times.first_third, date, self.adjustments.first_third),
            last_third: self.format_time(times.last_third, date, self.adjustments.last_third),
            date: date.format("%d/%m/%Y").to_string(),
            hijri: hijri.format("%d/%m/%Y").to_string(),
        })
    }

    pub fn calculate_qibla_direction(&self) -> f64 {
        let kaaba_lat = 21.4224779_f64;
        let kaaba_lng = 39.8251832_f64;

        let lat1 = dtr(self.coordinates.latitude);
        let lng1 = dtr(self.coordinates.longitude);
        let lat2 = dtr(kaaba_lat);
        let lng2 = dtr(kaaba_lng);

        let dlng = lng2 - lng1;
        let y = sin(dlng);
        let x = cos(lat1) * tan(lat2) - sin(lat1) * cos(dlng);

        let qibla = rtd(atan2(y, x));
        fix_angle(qibla)
    }

    fn compute_times(&self, date: DateTime<FixedOffset>) -> ApiResult<RawPrayerTimes> {
        let jd = self.julian_date(date)?;
        debug!("Julian Date: {}", jd);

        let (eqt, decl) = self.sun_position(jd);
        debug!("Equation of Time (eqt): {}, Declination (decl): {}", eqt, decl);

        let mut times = RawPrayerTimes::default();

        // Dhuhr calculation
        times.dhuhr = self.mid_day(eqt);

        // Sunrise and Sunset calculation
        let rise_set_angle = self.rise_set_angle();
        debug!("Rise/Set Angle: {}", rise_set_angle);
        times.sunrise = self.sun_angle_time(rise_set_angle, eqt, decl, -1.0)?;
        times.sunset = self.sun_angle_time(rise_set_angle, eqt, decl, 1.0)?;

        // Fajr calculation
        let fajr_angle = match self.method_settings.imsak {
            MinuteOrAngle::Angle { angle } => angle,
            _ => self.method_settings.fajr,
        };
        times.fajr = self.sun_angle_time(self.method_settings.fajr, eqt, decl, -1.0)?;

        // Asr calculation
        let asr_factor = match self.method_settings.school {
            School::Standard => 1.0,
            School::Hanafi => 2.0,
        };
        debug!("Asr shadow factor: {}, Asr Angle: {}", asr_factor, 
            rtd(atan(1.0 / (asr_factor + tan((dtr(self.coordinates.latitude) - dtr(decl)).abs())))));
        times.asr = self.asr_time(asr_factor, eqt, decl)?;

        // Maghrib calculation
        match &self.method_settings.maghrib {
            MinuteOrAngle::Angle { angle } => {
                times.maghrib = self.sun_angle_time(*angle, eqt, decl, 1.0)?;
            }
            MinuteOrAngle::Minute { minute } => {
                times.maghrib = times.sunset + minute / 60.0;
            }
        }

        // Isha calculation
        match &self.method_settings.isha {
            MinuteOrAngle::Angle { angle } => {
                times.isha = self.sun_angle_time(*angle, eqt, decl, 1.0)?;
            }
            MinuteOrAngle::Minute { minute } => {
                times.isha = times.maghrib + minute / 60.0;
            }
        }

        // Imsak calculation
        match &self.method_settings.imsak {
            MinuteOrAngle::Angle { angle } => {
                times.imsak = self.sun_angle_time(*angle, eqt, decl, -1.0)?;
            }
            MinuteOrAngle::Minute { minute } => {
                times.imsak = times.fajr - minute / 60.0;
            }
        }

        // Apply high latitude adjustments
        if let Some(rule) = self.method_settings.high_lat {
            debug!("Applying high latitude rule: {:?}", rule);
            times = self.adjust_high_latitudes(times, rule)?;
        }

        // Calculate night portions for Midnight, First Third, Last Third
        let night_length = fix_hour(times.sunrise - times.sunset);
        debug!("Night length: {} hours", night_length);
        match self.method_settings.midnight {
            Midnight::Standard => {
                times.midnight = times.sunset + night_length / 2.0;
            }
            Midnight::Jafari => {
                times.midnight = times.sunset + fix_hour(times.fajr - times.sunset) / 2.0;
            }
        }

        times.first_third = times.sunset + night_length / 3.0;
        times.last_third = times.sunset + (night_length * 2.0) / 3.0;

        // Apply longitude adjustment
        let lng_diff = self.coordinates.longitude / 15.0;
        debug!("Longitude difference adjustment: {} hours", lng_diff);
        times.imsak = fix_hour(times.imsak - lng_diff);
        times.fajr = fix_hour(times.fajr - lng_diff);
        times.sunrise = fix_hour(times.sunrise - lng_diff);
        times.dhuhr = fix_hour(times.dhuhr - lng_diff + self.method_settings.dhuhr / 60.0);
        times.asr = fix_hour(times.asr - lng_diff);
        times.sunset = fix_hour(times.sunset - lng_diff);
        times.maghrib = fix_hour(times.maghrib - lng_diff);
        times.isha = fix_hour(times.isha - lng_diff);
        times.midnight = fix_hour(times.midnight - lng_diff);
        times.first_third = fix_hour(times.first_third - lng_diff);
        times.last_third = fix_hour(times.last_third - lng_diff);

        Ok(times)
    }

    fn julian_date(&self, date: DateTime<FixedOffset>) -> ApiResult<f64> {
        let mut year = date.year();
        let mut month = date.month();
        let day = date.day();

        if month <= 2 {
            year -= 1;
            month += 12;
        }

        let a = floor(year as f64 / 100.0);
        let b = 2.0 - a + floor(a / 4.0);

        let jd = floor(365.25 * (year as f64 + 4716.0))
            + floor(30.6001 * (month as f64 + 1.0))
            + day as f64
            + b
            - 1524.5;
        
        Ok(jd)
    }

    fn sun_position(&self, jd: f64) -> (f64, f64) {
        let d = jd - 2451545.0;
        let g = fix_angle(357.529 + 0.98560028 * d);
        let q = fix_angle(280.459 + 0.98564736 * d);
        let l = fix_angle(q + 1.915 * sin(dtr(g)) + 0.020 * sin(dtr(2.0 * g)));
        let e = 23.439 - 0.00000036 * d;

        let ra = rtd(atan2(cos(dtr(e)) * sin(dtr(l)), cos(dtr(l)))) / 15.0;
        let eqt = q / 15.0 - fix_hour(ra);
        let decl = rtd(asin(sin(dtr(e)) * sin(dtr(l))));

        (eqt, decl)
    }

    fn mid_day(&self, eqt: f64) -> f64 {
        fix_hour(12.0 - eqt)
    }

    fn sun_angle_time(&self, angle: f64, eqt: f64, decl: f64, direction: f64) -> ApiResult<f64> {
        let lat = dtr(self.coordinates.latitude);
        let noon = self.mid_day(eqt);

        let p1 = -sin(dtr(angle)) - sin(dtr(decl)) * sin(lat);
        let p2 = cos(dtr(decl)) * cos(lat);

        if p2 == 0.0 {
            // This happens at the poles where cos(lat) is 0
            // and `cos(decl)` can be 0 depending on the time of year
            warn!("Division by zero in sun angle calculation (p2 is 0)");
            return Err(ApiError::Calculation(
                "Division by zero in sun angle calculation".to_string(),
            ));
        }

        let cos_range = (p1 / p2).clamp(-1.0, 1.0);
        let t = rtd(acos(cos_range)) / 15.0;

        Ok(noon + direction * t)
    }

    fn asr_time(&self, factor: f64, eqt: f64, decl: f64) -> ApiResult<f64> {
        let lat = dtr(self.coordinates.latitude);
        let decl_rad = dtr(decl);
        let angle = rtd(atan(1.0 / (factor + tan((lat - decl_rad).abs()))));
        debug!("Asr shadow factor: {}, Asr Angle: {}", factor, angle);

        self.sun_angle_time(angle, eqt, decl, 1.0)
    }

    fn rise_set_angle(&self) -> f64 {
        0.833 + 0.0347 * sqrt(self.coordinates.elevation.abs())
    }

    fn adjust_high_latitudes(
        &self,
        mut times: RawPrayerTimes,
        rule: HighLatitudeRule,
    ) -> ApiResult<RawPrayerTimes> {
        debug!("Starting high latitude adjustments");
        let fajr_angle = self.method_settings.fajr;
        let isha_angle = match self.method_settings.isha {
            MinuteOrAngle::Angle { angle } => angle,
            _ => 18.0, // Default to a standard angle if it's a minute-based method
        };

        let night_time = fix_hour(times.sunrise - times.sunset);
        debug!("Night time length for adjustments: {} hours", night_time);

        match rule {
            HighLatitudeRule::NightMiddle => {
                let portion = night_time / 2.0;
                debug!("Night Middle adjustment portion: {} hours", portion);
                if times.fajr.is_nan() || fix_hour(times.sunrise - times.fajr) > portion {
                    times.fajr = times.sunrise - portion;
                    debug!("Fajr adjusted to: {}", times.fajr);
                }
                if times.isha.is_nan() || fix_hour(times.isha - times.sunset) > portion {
                    times.isha = times.sunset + portion;
                    debug!("Isha adjusted to: {}", times.isha);
                }
            }
            HighLatitudeRule::OneSeventh => {
                let portion = night_time / 7.0;
                debug!("One Seventh adjustment portion: {} hours", portion);
                if times.fajr.is_nan() || fix_hour(times.sunrise - times.fajr) > portion {
                    times.fajr = times.sunrise - portion;
                    debug!("Fajr adjusted to: {}", times.fajr);
                }
                if times.isha.is_nan() || fix_hour(times.isha - times.sunset) > portion {
                    times.isha = times.sunset + portion;
                    debug!("Isha adjusted to: {}", times.isha);
                }
            }
            HighLatitudeRule::AngleBased => {
                let fajr_portion = fajr_angle / 60.0 * night_time;
                let isha_portion = isha_angle / 60.0 * night_time;
                debug!("Angle Based Fajr portion: {} hours, Isha portion: {} hours", fajr_portion, isha_portion);

                if times.fajr.is_nan() || fix_hour(times.sunrise - times.fajr) > fajr_portion {
                    times.fajr = times.sunrise - fajr_portion;
                    debug!("Fajr adjusted to: {}", times.fajr);
                }
                if times.isha.is_nan() || fix_hour(times.isha - times.sunset) > isha_portion {
                    times.isha = times.sunset + isha_portion;
                    debug!("Isha adjusted to: {}", times.isha);
                }
            }
        }
        Ok(times)
    }

    fn calculate_hijri_date(&self, date: NaiveDate) -> ApiResult<HijriDate> {
        let year: i16 = date.year().try_into().map_err(|e| {
            ApiError::Calculation(format!("Failed to convert year to i16: {}", e))
        })?;
        let month: u8 = date.month().try_into().map_err(|e| {
            ApiError::Calculation(format!("Failed to convert month to u8: {}", e))
        })?;
        let day: u8 = date.day().try_into().map_err(|e| {
            ApiError::Calculation(format!("Failed to convert day to u8: {}", e))
        })?;

        HijriDate::from_gr(year.try_into().unwrap(), month.into(), day.into())
            .map_err(|e| ApiError::Calculation(format!("Failed to calculate Hijri date: {}", e)))
    }

    fn format_time(&self, time: f64, date: DateTime<FixedOffset>, adjustment: i8) -> String {
        debug!("Entering format_time for raw time: {}", time);
        debug!("Adjustment: {}", adjustment);

        if time.is_nan() {
            warn!("Time is NaN, returning 'Invalid Time'");
            return "Invalid Time".to_string();
        }

        let offset = date.offset();
        debug!("Timezone offset: {} seconds", offset.local_minus_utc());

        let total_minutes = (time * 60.0).round() as i32 + adjustment as i32;
        debug!("Total minutes (raw time * 60 + adjustment): {}", total_minutes);

        // Convert total minutes to a duration and add to a naive date
        let dt_with_mins = date.date_naive().and_hms_opt(0, 0, 0).unwrap()
            + chrono::Duration::minutes(total_minutes as i64);

        // Apply the timezone offset
        let dt = dt_with_mins.and_local_timezone(*offset)
            .single()
            .unwrap_or_else(|| {
                // If there's an ambiguity (like DST change), handle it gracefully
                warn!("Ambiguous or non-existent time, using the first valid option.");
                dt_with_mins.and_local_timezone(*offset).earliest().unwrap_or_default()
            });

        let formatted_time = dt.format("%d/%m/%Y %H:%M").to_string();
        debug!("Formatted time string: {}", formatted_time);
        
        formatted_time
    }
}

#[derive(Debug, Default)]
struct RawPrayerTimes {
    imsak: f64,
    fajr: f64,
    sunrise: f64,
    dhuhr: f64,
    asr: f64,
    sunset: f64,
    maghrib: f64,
    isha: f64,
    midnight: f64,
    first_third: f64,
    last_third: f64,
}

impl StandardMethod {
    pub fn to_method_settings(self) -> MethodSettings {
        match self {
            Self::Mwl => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Angle { angle: 17.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Isna => MethodSettings {
                fajr: 15.0,
                isha: MinuteOrAngle::Angle { angle: 15.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Egypt => MethodSettings {
                fajr: 19.5,
                isha: MinuteOrAngle::Angle { angle: 17.5 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Makkah => MethodSettings {
                fajr: 18.5,
                isha: MinuteOrAngle::Minute { minute: 90.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Karachi => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Angle { angle: 18.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Tehran => MethodSettings {
                fajr: 17.7,
                isha: MinuteOrAngle::Angle { angle: 14.0 },
                midnight: Midnight::Jafari,
                maghrib: MinuteOrAngle::Angle { angle: 4.5 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Jafari => MethodSettings {
                fajr: 16.0,
                isha: MinuteOrAngle::Angle { angle: 14.0 },
                midnight: Midnight::Jafari,
                maghrib: MinuteOrAngle::Angle { angle: 4.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Gulf => MethodSettings {
                fajr: 19.5,
                isha: MinuteOrAngle::Minute { minute: 90.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Kuwait => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Angle { angle: 17.5 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Qatar => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Minute { minute: 90.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Singapore => MethodSettings {
                fajr: 20.0,
                isha: MinuteOrAngle::Angle { angle: 18.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::France | Self::Uoif => MethodSettings {
                fajr: 12.0,
                isha: MinuteOrAngle::Angle { angle: 12.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Turkey | Self::Diyanet => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Angle { angle: 17.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Russia => MethodSettings {
                fajr: 16.0,
                isha: MinuteOrAngle::Angle { angle: 15.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Moonsighting => MethodSettings {
                fajr: 18.0,
                isha: MinuteOrAngle::Angle { angle: 18.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: Some(crate::models::Shafaq::General),
                school: School::Standard,
                high_lat: None,
            },
            Self::Dubai => MethodSettings {
                fajr: 18.2,
                isha: MinuteOrAngle::Angle { angle: 18.2 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
            Self::Jakim => MethodSettings {
                fajr: 20.0,
                isha: MinuteOrAngle::Angle { angle: 18.0 },
                midnight: Midnight::Standard,
                maghrib: MinuteOrAngle::Minute { minute: 0.0 },
                imsak: MinuteOrAngle::Minute { minute: 10.0 },
                dhuhr: 0.0,
                shafaq: None,
                school: School::Standard,
                high_lat: None,
            },
        }
    }
}