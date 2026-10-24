use super::{Provider, weather};
use crate::{
    Result,
    config::{Config, DeparturesConfig, PiHoleConfig, WeatherConfig},
};

/// Only the settings needed by this integration travel with a request.
#[derive(Clone)]
pub enum Request {
    Weather(WeatherConfig),
    Departures(DeparturesConfig),
    PiHole(PiHoleConfig),
    Network { interface: String },
    Catalogue,
    Geocoding(String),
}

impl Request {
    pub fn configured(provider: Provider, config: &Config) -> Self {
        match provider {
            Provider::Weather => Self::Weather(config.weather.clone()),
            Provider::Departures => Self::Departures(config.departures.clone()),
            Provider::PiHole => Self::PiHole(config.pi_hole.clone()),
            Provider::Network => Self::Network {
                interface: config.network_interface.clone(),
            },
            Provider::Catalogue => Self::Catalogue,
            Provider::Geocoding => Self::Geocoding(String::new()),
        }
    }

    pub fn search(text: &str) -> Option<Self> {
        let text = weather::normalize_search(text);
        (text.chars().count() >= 2).then_some(Self::Geocoding(text))
    }

    pub fn provider(&self) -> Provider {
        match self {
            Self::Weather(_) => Provider::Weather,
            Self::Departures(_) => Provider::Departures,
            Self::PiHole(_) => Provider::PiHole,
            Self::Network { .. } => Provider::Network,
            Self::Catalogue => Provider::Catalogue,
            Self::Geocoding(_) => Provider::Geocoding,
        }
    }

    pub fn key(&self) -> String {
        (crate::widgets::registry::for_provider(self.provider()).key)(self)
    }

    pub fn refresh_seconds(&self) -> u64 {
        (crate::widgets::registry::for_provider(self.provider()).refresh_seconds)(self)
    }

    pub fn validate_test(&self) -> Result<()> {
        let mut config = Config::default();
        match self {
            Self::Weather(weather) => config.weather = weather.clone(),
            Self::Departures(departures) => {
                config.departures = departures.clone();
                config.departures.enabled = true;
            }
            Self::PiHole(pi_hole) => {
                config.pi_hole = pi_hole.clone();
                config.pi_hole.enabled = true;
            }
            _ => return Err("This integration has no connection test".into()),
        }
        config.validate()
    }
}

pub fn key(provider: Provider, config: &Config) -> String {
    Request::configured(provider, config).key()
}

pub fn live_requests(config: &Config) -> Vec<Request> {
    crate::widgets::registry::REGISTERED
        .iter()
        .flat_map(|registration| (registration.requests)(config))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Widget;

    #[test]
    fn duplicate_widgets_share_queries_and_line_filters_do_not_change_key() {
        let mut config = Config::default();
        config.weather.enabled = true;
        config.layout[0].columns[0].widgets = vec![Widget::Weather; 3];
        assert_eq!(
            live_requests(&config)
                .iter()
                .filter(|query| query.provider() == Provider::Weather)
                .count(),
            1
        );
        config.departures.stops.push(crate::config::StopConfig {
            id: 399,
            kind: crate::config::StopKind::StopId,
            label: "stop".into(),
            lines: vec!["18".into()],
        });
        let original = key(Provider::Departures, &config);
        config.departures.stops[0].lines = vec!["6".into()];
        assert_eq!(key(Provider::Departures, &config), original);
        for id in [60200267, 60200268] {
            config.departures.stops.push(crate::config::StopConfig {
                id,
                kind: crate::config::StopKind::Diva,
                label: String::new(),
                lines: vec![],
            });
        }
        let combined = key(Provider::Departures, &config);
        assert_ne!(combined, original);
        config.departures.stops.reverse();
        assert_eq!(key(Provider::Departures, &config), combined);
        config.departures.stops[0].kind = crate::config::StopKind::StopId;
        assert_ne!(key(Provider::Departures, &config), combined);
        let weather_key = key(Provider::Weather, &config);
        config.weather.fahrenheit = true;
        assert_ne!(weather_key, key(Provider::Weather, &config));
    }
}
