use crate::{
    config::Config,
    ui::{UiAction, data::LiveData, theme::BLUE},
};
use eframe::egui::{self, Align, Color32};

pub fn show(
    ui: &mut egui::Ui,
    config: &Config,
    data: &LiveData,
    widgets: &mut crate::widgets::registry::Registry,
) -> Option<UiAction> {
    if !config.status_bar {
        return ui.horizontal(|ui| controls(ui, true)).inner;
    }
    egui::Frame::new()
        .fill(BLUE)
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::from_rgb(0, 80, 115);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::from_rgb(0, 65, 100);
            ui.visuals_mut().widgets.active.weak_bg_fill = Color32::from_rgb(0, 50, 85);
            ui.horizontal_wrapped(|ui| {
                let action = controls(ui, false);
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    widgets.status(ui, config, data);
                });
                action
            })
            .inner
        })
        .inner
}

fn controls(ui: &mut egui::Ui, small: bool) -> Option<UiAction> {
    let mut action = None;
    for (label, intent) in [
        ("Settings", UiAction::OpenSettings),
        ("Refresh", UiAction::Refresh),
        ("Quit", UiAction::Quit),
    ] {
        let button = if small {
            egui::Button::new(label).small()
        } else {
            egui::Button::new(label)
        };
        if ui.add(button).clicked() {
            action = Some(intent);
        }
    }
    action
}
