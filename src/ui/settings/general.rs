use super::Context;
use crate::{config::Config, ui::UiAction};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &mut Config, context: &Context<'_>) -> Option<UiAction> {
    let mut action = None;
    ui.heading("General");
    ui.checkbox(&mut config.fullscreen, "Start fullscreen");
    if context.windowed {
        ui.weak("--windowed overrides fullscreen for this launch.");
    }
    ui.checkbox(&mut config.debug, "Print network requests to stdout");
    if context.debug_flag {
        ui.weak("--debug forces request logging on for this launch.");
    }
    ui.checkbox(&mut config.status_bar, "Show status bar");
    ui.separator();
    ui.checkbox(&mut config.screen_wake, "Automatic screen wake and sleep");
    ui.weak("Wake on presence or input; sleep after 10 minutes without either. Remove desktop screen blanking before enabling.");
    match context.screen_power {
        crate::screen_power::Status::Checking => {
            ui.weak("Checking screen power availability…");
        }
        crate::screen_power::Status::Available => {
            ui.weak("Sensor and Wayland screen power control are available.");
        }
        crate::screen_power::Status::Unavailable(message) => {
            ui.label(format!(
                "ℹ Automatic screen power is unavailable: {message}"
            ));
            ui.weak("Automatic sleep is suspended while unavailable. Availability is checked again automatically.");
        }
    }
    ui.separator();
    if ui.button("Clear cached data").clicked() {
        action = Some(UiAction::ClearCache);
    }
    ui.weak("Clears data and searches. Enabled widgets fetch again; credentials are kept.");
    ui.label(format!("Configuration: {}", context.config_path.display()));
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{screen_power::Status, ui::data::LiveData};
    use std::path::Path;

    #[test]
    fn availability_notice_is_visible_even_when_disabled_at_display_sizes() {
        let egui = egui::Context::default();
        let mut config = Config::default();
        let saved = config.clone();
        let data = LiveData::default();
        let context = Context {
            saved: &saved,
            data: &data,
            notice: "",
            screen_power: &Status::Unavailable("Enable I²C and check the i2c group.".into()),
            config_path: Path::new("config.json"),
            windowed: false,
            debug_flag: false,
        };
        for size in [egui::vec2(1024.0, 600.0), egui::vec2(1920.0, 1080.0)] {
            let mut output = egui.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| {
                    show(ui, &mut config, &context);
                },
            );
            output.textures_delta.clear();
            assert!(!config.screen_wake);
            assert!(output.shapes.iter().any(|shape| {
                match &shape.shape {
                    egui::Shape::Text(text) => text
                        .galley
                        .job
                        .text
                        .contains("Automatic screen power is unavailable: Enable I²C"),
                    _ => false,
                }
            }));
        }
    }
}
