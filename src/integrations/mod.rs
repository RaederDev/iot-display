pub use crate::widgets::widget_departures::data as departures;
pub use crate::widgets::widget_network::data as network;
pub use crate::widgets::widget_pihole::data as pi_hole;
mod requests;
pub use crate::widgets::widget_weather::data as weather;

pub use requests::{Request, key, live_requests};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Weather,
    Departures,
    PiHole,
    Catalogue,
    Geocoding,
    Network,
}

impl Provider {
    pub fn persistent_cache(self) -> bool {
        self != Self::Network
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Weather => "weather",
            Self::Departures => "departures",
            Self::PiHole => "pi-hole",
            Self::Catalogue => "stop catalogue",
            Self::Geocoding => "location search",
            Self::Network => "network",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Data {
    Weather(weather::Weather),
    Departures(Vec<departures::Board>),
    PiHole(pi_hole::Stats),
    Catalogue(Vec<departures::Stop>),
    Geocoding(Vec<weather::Location>),
    Network(network::Network),
}

impl Data {
    pub fn provider(&self) -> Provider {
        match self {
            Self::Weather(_) => Provider::Weather,
            Self::Departures(_) => Provider::Departures,
            Self::PiHole(_) => Provider::PiHole,
            Self::Catalogue(_) => Provider::Catalogue,
            Self::Geocoding(_) => Provider::Geocoding,
            Self::Network(_) => Provider::Network,
        }
    }

    pub fn valid(&self) -> bool {
        match self {
            Self::Weather(value) => value.valid(),
            Self::Catalogue(stops) => !stops.is_empty() && stops.iter().all(|stop| stop.diva > 0),
            Self::Geocoding(locations) => locations.iter().all(|location| location.valid()),
            Self::Departures(boards) => boards.iter().all(|board| board.id > 0),
            Self::PiHole(_) | Self::Network(_) => true,
        }
    }
}
