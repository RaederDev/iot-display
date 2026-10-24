pub mod config;
pub mod data;
pub mod settings;
pub mod timer;
pub mod ui;

use crate::{
    config::Widget,
    integrations::{Data, Provider},
    ui::{UiAction, settings as app_settings},
    widgets::registry::{Registration, RenderContext, View as WidgetView},
};
use eframe::egui;

pub const REGISTRATION: Registration = Registration {
    kind: Widget::PiHole,
    name: "Pi-hole",
    key: timer::key,
    fonts: None,
    providers: &[Provider::PiHole],
    view: || Box::new(View),
    backend: Some(timer::backend),
    requests: timer::requests,
    refresh_seconds: timer::refresh_seconds,
    validate: config::validate,
    background: None,
    foreground: None,
    card: true,
    settings_opened: None,
};

pub struct View;

impl WidgetView for View {
    fn show(&mut self, ui: &mut egui::Ui, context: &RenderContext<'_>) {
        ui::show(ui, context.config, context.data);
    }

    fn settings(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut app_settings::State,
        _context: &app_settings::Context<'_>,
    ) -> Option<UiAction> {
        let result = match state.test_results.get(&Provider::PiHole) {
            Some(Ok(Data::PiHole(data))) => Some(data),
            _ => None,
        };
        settings::show(ui, &mut state.draft.pi_hole, result)
    }
}
