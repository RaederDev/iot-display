use crate::config::Config;
use eframe::egui::{self, RichText};

pub fn show(ui: &mut egui::Ui, config: &Config) {
    let greeting = if config.username.trim().is_empty() {
        "Hello, here's today's briefing:".into()
    } else {
        format!("Hello {}, here's today's briefing:", config.username)
    };
    ui.label(RichText::new(greeting).size(24.0));
}
