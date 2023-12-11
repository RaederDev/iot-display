use std::collections::HashMap;

use anyhow::{bail, Error, Ok};
use config::{Config, ConfigError};
use dirs::config_dir;
use serde::Deserialize;

use crate::widgets::widget::WidgetType;

const GRID_SIZE: u8 = 12;
const SETTINGS_FILE: &str = "Settings.toml";
const CONFIG_DIR: &str = "iot-display";

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum PositionConfig {
    Left,
    Center,
    Right,
}

#[derive(Deserialize, Debug)]
pub struct GridConfig {
    pub left: u8,
    pub center: u8,
    pub right: u8,
}

#[derive(Deserialize, Debug)]
pub struct WidgetConfig {
    pub position: PositionConfig,
    pub widget_type: WidgetType,
    pub config: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub grid: GridConfig,
    pub widget: Option<Vec<WidgetConfig>>,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let config_home = config_dir()
            .expect("Could not find config directory")
            .join(CONFIG_DIR)
            .join(SETTINGS_FILE);
        let settings = Config::builder()
            .add_source(config::File::with_name(SETTINGS_FILE).required(false))
            .add_source(config::File::from(config_home).required(false))
            .build()?;

        settings.try_deserialize::<Configuration>()
    }

    pub fn validate(&self) -> Result<(), Error> {
        let specified_grid_size = self
            .grid
            .left
            .saturating_add(self.grid.center)
            .saturating_add(self.grid.right);
        if specified_grid_size != GRID_SIZE {
            bail!(format!(
                "Grid size {} does not match required value {}",
                specified_grid_size, GRID_SIZE
            ));
        }

        Ok(())
    }
}
