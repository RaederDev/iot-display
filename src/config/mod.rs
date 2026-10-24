pub mod layout;

pub use crate::widgets::{
    widget_departures::config::{DeparturesConfig, StopConfig, StopKind},
    widget_pihole::config::PiHoleConfig,
    widget_weather::config::WeatherConfig,
};
use crate::{Result, storage};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Widget {
    Greeting,
    Weather,
    PiHole,
    Departures,
    Clock,
    Network,
}

impl Widget {
    pub fn name(self) -> &'static str {
        crate::widgets::registry::for_kind(self)
            .map_or("Unavailable widget", |registration| registration.name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Column {
    pub weight: f32,
    pub widgets: Vec<Widget>,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            weight: 1.0,
            widgets: vec![],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub columns: Vec<Column>,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            columns: vec![Column::default()],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub username: String,
    pub fullscreen: bool,
    pub screen_wake: bool,
    pub debug: bool,
    pub dark: bool,
    pub scale: f32,
    pub spacing: f32,
    pub status_bar: bool,
    pub show_ip: bool,
    pub show_clock: bool,
    pub clock_format: String,
    pub network_interface: String,
    pub weather: WeatherConfig,
    pub departures: DeparturesConfig,
    pub pi_hole: PiHoleConfig,
    pub layout: Vec<Row>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            username: "".into(),
            fullscreen: true,
            screen_wake: false,
            debug: false,
            dark: false,
            scale: 1.0,
            spacing: 20.0,
            status_bar: true,
            show_ip: true,
            show_clock: true,
            clock_format: "%d.%m.%Y %H:%M:%S".into(),
            network_interface: "".into(),
            weather: WeatherConfig::default(),
            departures: DeparturesConfig::default(),
            pi_hole: PiHoleConfig::default(),
            layout: vec![
                Row {
                    columns: vec![Column {
                        weight: 1.0,
                        widgets: vec![Widget::Greeting],
                    }],
                },
                Row {
                    columns: vec![
                        Column {
                            weight: 5.0,
                            widgets: vec![Widget::Weather, Widget::PiHole],
                        },
                        Column {
                            weight: 7.0,
                            widgets: vec![Widget::Departures],
                        },
                    ],
                },
            ],
        }
    }
}

impl Config {
    pub fn uses(&self, widget: Widget) -> bool {
        self.layout.iter().any(|row| {
            row.columns
                .iter()
                .any(|column| column.widgets.contains(&widget))
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported configuration version".into());
        }
        if !self.scale.is_finite() || !(0.6..=3.0).contains(&self.scale) {
            return Err("UI scale must be between 0.6 and 3".into());
        }
        if !self.spacing.is_finite() || !(4.0..=40.0).contains(&self.spacing) {
            return Err("Spacing must be between 4 and 40".into());
        }
        for registration in crate::widgets::registry::REGISTERED {
            (registration.validate)(self)?;
        }
        if self.layout.is_empty()
            || self.layout.iter().any(|row| {
                row.columns.is_empty()
                    || row.columns.iter().any(|column| {
                        !column.weight.is_finite() || column.weight <= 0.0 || column.weight > 100.0
                    })
            })
        {
            return Err(
                "Layout needs rows and columns with widths greater than zero and at most 100"
                    .into(),
            );
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let config: Self = storage::read_json(path)?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        storage::write_json(path, self)
    }
}

pub use crate::widgets::widget_pihole::config::validate_api_url;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_wake_is_opt_in_for_existing_and_new_configurations() {
        assert!(!Config::default().screen_wake);
        let config: Config = serde_json::from_str(r#"{"version":1}"#).unwrap();
        assert!(!config.screen_wake);
    }

    #[test]
    fn refresh_intervals_are_fixed_and_legacy_fields_are_not_saved() {
        let config: Config = serde_json::from_str(r#"{"network_refresh_seconds":1,"weather":{"refresh_seconds":1},"departures":{"refresh_seconds":1},"pi_hole":{"refresh_seconds":1}}"#).unwrap();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("refresh_seconds"));
        for (provider, seconds) in [
            (crate::integrations::Provider::Weather, 3600),
            (crate::integrations::Provider::Departures, 300),
            (crate::integrations::Provider::PiHole, 60),
            (crate::integrations::Provider::Network, 60),
        ] {
            assert_eq!(
                crate::integrations::Request::configured(provider, &config).refresh_seconds(),
                seconds
            );
        }
    }

    #[test]
    fn validates_boundaries() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        config.layout[0].columns[0].weight = 0.0;
        assert!(config.validate().is_err());
        config.layout[0].columns[0].weight = 1.0;
        config.weather.latitude = f64::NAN;
        assert!(config.validate().is_err());
        assert!(validate_api_url("http://user:secret@host/api").is_err());
        assert!(validate_api_url("https://host/proxy/api/").is_ok());
    }

    #[test]
    fn invalid_save_preserves_existing_config() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("app/config.json");
        let mut config = Config::default();
        config.save(&path).unwrap();
        config.clock_format = "%Q".into();
        assert!(config.save(&path).is_err());
        assert!(Config::load(&path).is_ok());
    }

    #[test]
    fn existing_stopping_point_configuration_keeps_its_identifier_type() {
        let config: Config = serde_json::from_str(r#"{"departures":{"enabled":true,"stops":[{"id":399,"label":"Eichenstraße","lines":["18"]}]}}"#).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.departures.stops[0].kind, StopKind::StopId);
        assert_eq!(config.departures.stops[0].id, 399);
    }
}
