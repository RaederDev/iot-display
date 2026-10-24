use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StopConfig {
    pub id: u32,
    #[serde(default)]
    pub kind: StopKind,
    pub label: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    #[default]
    StopId,
    Diva,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeparturesConfig {
    pub enabled: bool,
    pub stops: Vec<StopConfig>,
}

pub fn validate(config: &crate::config::Config) -> crate::Result<()> {
    if config.departures.enabled && config.departures.stops.is_empty() {
        return Err("Add a departure stop or disable departures".into());
    }
    if config.departures.stops.iter().any(|stop| stop.id == 0) {
        return Err("Stop IDs must be positive".into());
    }
    let mut ids = std::collections::HashSet::new();
    if config
        .departures
        .stops
        .iter()
        .any(|stop| !ids.insert(stop.id))
    {
        return Err("Each stop ID can be configured only once".into());
    }
    Ok(())
}
