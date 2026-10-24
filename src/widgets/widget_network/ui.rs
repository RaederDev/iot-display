use crate::{
    config::Config,
    integrations::{Data, Provider},
    ui::data::LiveData,
    widgets::widget_network::data::Network,
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &Config, data: &LiveData) {
    ui.heading("Network");
    address(ui, config, data);
}

pub fn status(ui: &mut egui::Ui, config: &Config, data: &LiveData) {
    if config.show_ip {
        address(ui, config, data);
    }
}

fn address(ui: &mut egui::Ui, config: &Config, data: &LiveData) {
    let address = data
        .network(config)
        .and_then(|network| network.selected.as_ref())
        .map(|interface| interface.address.as_str())
        .unwrap_or("unavailable");
    ui.label(format!("IP: {address}"));
}

impl LiveData {
    pub fn network(&self, config: &Config) -> Option<&Network> {
        match &self.entry(Provider::Network, config)?.data {
            Data::Network(network) => Some(network),
            _ => None,
        }
    }
}
