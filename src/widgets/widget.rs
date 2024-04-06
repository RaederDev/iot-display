use std::collections::HashMap;

use anyhow::Error;
use dioxus::dioxus_core::Element;
use serde::Deserialize;
use strum_macros::EnumIter;

use crate::config::{PositionConfig, WidgetConfig};

use super::{pihole_widget::PiholeWidget, text_widget::TextWidget};

#[derive(Debug, EnumIter, Deserialize)]
pub enum WidgetType {
    Text,
    Pihole,
}
pub struct WidgetFactory;
impl WidgetFactory {
    pub fn make_widget(config: &WidgetConfig) -> Result<Box<dyn Widget>, Error> {
        match config.widget_type {
            WidgetType::Text => TextWidget::build(&config.config),
            WidgetType::Pihole => PiholeWidget::build(&config.config),
        }
    }

    pub fn make_widgets(
        configs: &Option<Vec<WidgetConfig>>,
        pos: PositionConfig,
    ) -> Vec<Box<dyn Widget>> {
        if configs.is_none() {
            return vec![];
        }
        configs
            .as_ref()
            .unwrap()
            .iter()
            .filter(|cfg| cfg.position.eq(&pos))
            .filter_map(|cfg| WidgetFactory::make_widget(cfg).ok())
            .collect::<Vec<Box<dyn Widget>>>()
    }
}

pub trait Widget {
    fn render(&self) -> Element;
    fn build(config: &Option<HashMap<String, String>>) -> Result<Box<dyn Widget>, Error>
    where
        Self: Sized;
}
