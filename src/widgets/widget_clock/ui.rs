use crate::config::Config;
use eframe::egui::{self, RichText};

pub fn show(ui: &mut egui::Ui, config: &Config) {
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_secs(1));
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(
                chrono::Local::now()
                    .format(&config.clock_format)
                    .to_string(),
            )
            .size(40.0),
        );
    });
}

pub fn status(ui: &mut egui::Ui, config: &Config, _data: &crate::ui::data::LiveData) {
    if config.show_clock {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));
        ui.label(
            chrono::Local::now()
                .format(&config.clock_format)
                .to_string(),
        );
    }
}
