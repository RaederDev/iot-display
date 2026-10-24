pub mod config;
pub mod settings;
pub mod ui;

use crate::{
    config::Widget,
    ui::{UiAction, settings as app_settings},
    widgets::registry::{Registration, RenderContext, View as WidgetView},
};
use eframe::egui;

pub const REGISTRATION: Registration = Registration {
    kind: Widget::Clock,
    name: "Big clock",
    key: |_| String::new(),
    fonts: None,
    providers: &[],
    view: || Box::new(View),
    backend: None,
    requests: |_| vec![],
    refresh_seconds: |_| 0,
    validate: config::validate,
    background: None,
    foreground: None,
    card: true,
    settings_opened: None,
};

pub struct View;

impl WidgetView for View {
    fn show(&mut self, ui: &mut egui::Ui, context: &RenderContext<'_>) {
        ui::show(ui, context.config);
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
