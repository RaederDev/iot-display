use crate::{
    config::Config,
    ui::settings::{Context, controls::text},
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &mut Config, _context: &Context<'_>) {
    ui.heading("Clock");
    ui.checkbox(&mut config.show_clock, "Show clock in status bar");
    text(ui, "Clock format (strftime)", &mut config.clock_format);
    ui.weak("Example: %d.%m.%Y %H:%M:%S. Uses the system timezone.");
}
