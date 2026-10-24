use crate::{
    Result,
    config::WeatherConfig,
    http::{Http, HttpError},
};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weather {
    pub temperature: f64,
    pub apparent: f64,
    pub code: u16,
    pub is_day: bool,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub date: NaiveDate,
    pub timezone: chrono_tz::Tz,
}

impl Weather {
    pub fn valid(&self) -> bool {
        self.temperature.is_finite()
            && self.apparent.is_finite()
            && self.high.is_none_or(f64::is_finite)
            && self.low.is_none_or(f64::is_finite)
    }

    pub fn seconds_until_next_day(&self, now: DateTime<Utc>) -> u64 {
        let local = now.with_timezone(&self.timezone);
        local
            .date_naive()
            .succ_opt()
            .and_then(|day| day.and_hms_opt(0, 0, 0))
            .and_then(|midnight| midnight.and_local_timezone(self.timezone).earliest())
            .map_or(3600, |midnight| {
                (midnight.with_timezone(&Utc) - now).num_seconds().max(1) as u64
            })
    }

    pub fn current_day(&self, now: DateTime<Utc>) -> bool {
        self.date == now.with_timezone(&self.timezone).date_naive()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub admin1: String,
}

impl Location {
    pub fn valid(&self) -> bool {
        self.latitude.is_finite()
            && self.longitude.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }

    pub fn label(&self) -> String {
        [
            self.name.as_str(),
            self.admin1.as_str(),
            self.country.as_str(),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
}

#[derive(Deserialize)]
struct Forecast {
    current: Current,
    daily: Daily,
    timezone: chrono_tz::Tz,
}

#[derive(Deserialize)]
struct Current {
    temperature_2m: f64,
    apparent_temperature: f64,
    weather_code: u16,
    is_day: u8,
}

#[derive(Deserialize)]
struct Daily {
    time: Vec<NaiveDate>,
    temperature_2m_max: Vec<Option<f64>>,
    temperature_2m_min: Vec<Option<f64>>,
}

pub fn forecast_url(config: &WeatherConfig) -> Url {
    let mut url = Url::parse("https://api.open-meteo.com/v1/forecast").unwrap();
    url.query_pairs_mut()
        .append_pair("latitude", &config.latitude.to_string())
        .append_pair("longitude", &config.longitude.to_string())
        .append_pair(
            "current",
            "temperature_2m,apparent_temperature,weather_code,is_day",
        )
        .append_pair("daily", "temperature_2m_max,temperature_2m_min")
        .append_pair("forecast_days", "1")
        .append_pair("timezone", "auto")
        .append_pair(
            "temperature_unit",
            if config.fahrenheit {
                "fahrenheit"
            } else {
                "celsius"
            },
        );
    url
}

pub fn parse_forecast(bytes: &[u8]) -> Result<Weather> {
    let data: Forecast = serde_json::from_slice(bytes).map_err(|_| "Invalid weather response")?;
    let date = *data
        .daily
        .time
        .first()
        .ok_or("Weather response has no daily date")?;
    let weather = Weather {
        temperature: data.current.temperature_2m,
        apparent: data.current.apparent_temperature,
        code: data.current.weather_code,
        is_day: data.current.is_day == 1,
        high: data.daily.temperature_2m_max.first().copied().flatten(),
        low: data.daily.temperature_2m_min.first().copied().flatten(),
        date,
        timezone: data.timezone,
    };
    if !weather.valid() {
        return Err("Invalid weather values".into());
    }
    Ok(weather)
}

pub async fn fetch(http: &Http, config: &WeatherConfig) -> std::result::Result<Weather, HttpError> {
    let bytes = http.get("weather", forecast_url(config)).await?;
    parse_forecast(&bytes).map_err(Into::into)
}

pub fn normalize_search(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub async fn search(http: &Http, query: &str) -> std::result::Result<Vec<Location>, HttpError> {
    let mut url = Url::parse("https://geocoding-api.open-meteo.com/v1/search").unwrap();
    url.query_pairs_mut()
        .append_pair("name", query)
        .append_pair("count", "10")
        .append_pair("language", "en");
    let bytes = http.get("location search", url).await?;

    #[derive(Deserialize)]
    struct Response {
        #[serde(default)]
        results: Vec<Location>,
    }
    let response: Response = serde_json::from_slice(&bytes)
        .map_err(|_| HttpError::from("Invalid location search response".to_owned()))?;
    if !response.results.iter().all(Location::valid) {
        return Err("Invalid location coordinates".to_owned().into());
    }
    Ok(response.results)
}

pub fn condition(code: u16) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95..=99 => "Thunderstorm",
        _ => "Unknown conditions",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_dates_and_missing_daily_values() {
        let bytes = br#"{"current":{"temperature_2m":12,"apparent_temperature":10,"weather_code":2,"is_day":1},"daily":{"time":["2026-10-11"],"temperature_2m_max":[null],"temperature_2m_min":[4]},"timezone":"Europe/Vienna"}"#;
        let data = parse_forecast(bytes).unwrap();
        assert_eq!(data.high, None);
        let before_midnight = DateTime::parse_from_rfc3339("2026-10-11T21:59:50Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(data.seconds_until_next_day(before_midnight), 10);
        // Old response dates do not make the independent deadline perpetually due.
        let next_day = before_midnight + chrono::Duration::seconds(10);
        assert_eq!(data.seconds_until_next_day(next_day), 86400);
        assert_eq!(data.low, Some(4.0));
        assert!(
            data.current_day(
                DateTime::parse_from_rfc3339("2026-10-11T21:59:59Z")
                    .unwrap()
                    .with_timezone(&Utc)
            )
        );
        assert!(
            !data.current_day(
                DateTime::parse_from_rfc3339("2026-10-11T22:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc)
            )
        );
        let config = WeatherConfig {
            fahrenheit: true,
            ..Default::default()
        };
        assert!(
            forecast_url(&config)
                .as_str()
                .contains("temperature_unit=fahrenheit")
        );
        assert!(parse_forecast(b"{}").is_err());
        assert_eq!(normalize_search("  New   YORK "), "new york");
    }
}
