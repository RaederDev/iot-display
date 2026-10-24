pub mod data;
pub mod settings;
pub mod timer;
pub mod ui;

use crate::{
    config::Widget,
    integrations::Provider,
    ui::{UiAction, settings as app_settings},
    widgets::registry::{Registration, RenderContext, View as WidgetView},
};
use eframe::egui;

pub const REGISTRATION: Registration = Registration {
    kind: Widget::Network,
    name: "Network",
    key: timer::key,
    fonts: None,
    providers: &[Provider::Network],
    view: || Box::new(View),
    backend: Some(timer::backend),
    requests: timer::requests,
    refresh_seconds: timer::refresh_seconds,
    validate: |_| Ok(()),
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
        context: &app_settings::Context<'_>,
    ) -> Option<UiAction> {
        settings::show(ui, &mut state.draft, context);
        None
    }

    fn status(
        &mut self,
        ui: &mut egui::Ui,
        config: &crate::config::Config,
        data: &crate::ui::data::LiveData,
    ) {
        ui::status(ui, config, data);
    }
}
