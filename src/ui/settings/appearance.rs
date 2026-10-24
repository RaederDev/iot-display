use crate::config::Config;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &mut Config) {
    ui.heading("Appearance");
    ui.checkbox(&mut config.dark, "Dark theme");
    ui.add(egui::Slider::new(&mut config.scale, 0.6..=3.0).text("UI scale"));
    ui.add(egui::Slider::new(&mut config.spacing, 4.0..=40.0).text("Spacing"));
}
