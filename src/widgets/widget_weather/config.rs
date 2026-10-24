use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub fahrenheit: bool,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            label: "Vienna".into(),
            latitude: 48.2082,
            longitude: 16.3738,
            fahrenheit: false,
        }
    }
}

pub fn validate(config: &crate::config::Config) -> crate::Result<()> {
    if !config.weather.latitude.is_finite()
        || !(-90.0..=90.0).contains(&config.weather.latitude)
        || !config.weather.longitude.is_finite()
        || !(-180.0..=180.0).contains(&config.weather.longitude)
    {
        return Err("Weather coordinates are out of range".into());
    }
    Ok(())
}
