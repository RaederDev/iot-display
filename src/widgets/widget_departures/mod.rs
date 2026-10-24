pub mod config;
pub mod data;
pub mod settings;
pub mod timer;
pub mod ui;

use crate::{
    config::Widget,
    integrations::{Data, Provider, Request},
    ui::{UiAction, settings as app_settings},
    widgets::registry::{Registration, RenderContext, View as WidgetView, WidgetId},
};
use eframe::egui;

pub const REGISTRATION: Registration = Registration {
    kind: Widget::Departures,
    name: "Departures",
    key: timer::key,
    fonts: Some(ui::install_fonts),
    providers: &[Provider::Departures, Provider::Catalogue],
    view: || Box::new(View::default()),
    backend: Some(timer::backend),
    requests: timer::requests,
    refresh_seconds: timer::refresh_seconds,
    validate: config::validate,
    background: Some(egui::Color32::from_rgb(48, 48, 48)),
    foreground: Some(ui::GREEN),
    card: true,
    settings_opened: Some(|| UiAction::Request(Request::Catalogue)),
};

#[derive(Default)]
pub struct View {
    pub editor: settings::Editor,
    pub details: std::collections::HashMap<WidgetId, data::Selection>,
}

impl WidgetView for View {
    fn show(&mut self, ui: &mut egui::Ui, context: &RenderContext<'_>) {
        if context.config.departures.enabled
            && context
                .data
                .entry(Provider::Departures, context.config)
                .is_some()
        {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
        }
        match ui::show(
            ui,
            context.config,
            context.data,
            self.details.get(&context.id),
            context.preview,
        ) {
            Some(ui::Navigation::Show(selection)) => {
                self.details.insert(context.id, selection);
            }
            Some(ui::Navigation::Back) => {
                self.details.remove(&context.id);
            }
            None => {}
        }
    }

    fn settings(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut app_settings::State,
        context: &app_settings::Context<'_>,
    ) -> Option<UiAction> {
        let boards = match state.test_results.get(&Provider::Departures) {
            Some(Ok(Data::Departures(data))) => data,
            _ => context.data.departures(context.saved),
        };
        self.editor.show(
            ui,
            &mut state.draft.departures,
            context.data.catalogue(context.saved),
            boards,
        )
    }

    fn reset(&mut self) {
        self.editor = settings::Editor::default();
        self.details.clear();
    }
}
