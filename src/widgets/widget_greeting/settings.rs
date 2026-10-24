use crate::{
    config::Config,
    ui::settings::{Context, controls::text},
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &mut Config, _context: &Context<'_>) {
    ui.heading("Greeting");
    text(ui, "Name", &mut config.username);
}
