pub mod config;
pub mod data;
pub mod settings;
pub mod timer;
pub mod ui;

use crate::{
    config::Widget,
    integrations::{Data, Provider},
    runtime::Event,
    ui::{UiAction, settings as app_settings},
    widgets::registry::{Registration, RenderContext, View as WidgetView},
};
use eframe::egui;

pub const REGISTRATION: Registration = Registration {
    kind: Widget::Weather,
    name: "Weather",
    key: timer::key,
    fonts: None,
    providers: &[Provider::Weather, Provider::Geocoding],
    view: || Box::new(View::default()),
    backend: Some(timer::backend),
    requests: timer::requests,
    refresh_seconds: timer::refresh_seconds,
    validate: config::validate,
    background: None,
    foreground: None,
    card: true,
    settings_opened: None,
};

#[derive(Default)]
pub struct View {
    pub editor: settings::Editor,
}

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
        let result = match state.test_results.get(&Provider::Weather) {
            Some(Ok(Data::Weather(data))) => Some(data),
            _ => None,
        };
        self.editor.show(ui, &mut state.draft.weather, result)
    }

    fn event(&mut self, event: &Event) {
        match event {
            Event::DataUpdated { key, entry, .. } => {
                if let Data::Geocoding(results) = &entry.data {
                    self.editor.accept_results(key, results);
                }
            }
            Event::CacheCleared => {
                self.editor.results.clear();
                self.editor.status.clear();
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.editor = settings::Editor::default();
    }
}
