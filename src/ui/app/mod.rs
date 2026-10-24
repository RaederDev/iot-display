mod events;

#[cfg(test)]
mod tests;

use super::{UiAction, dashboard, data::LiveData, settings, theme};
use crate::{
    Result,
    config::Config,
    integrations::{Provider, Request},
    runtime::{Command, Runtime},
    widgets::registry::Registry,
};
use eframe::egui;
use std::path::PathBuf;

pub struct Startup {
    pub config_path: PathBuf,
    pub cache_path: PathBuf,
    pub debug_flag: bool,
    pub windowed: bool,
    pub first_run: bool,
    pub load_error: Option<String>,
}

pub struct App {
    generation: u64,
    config: Config,
    config_path: PathBuf,
    runtime: Runtime,
    data: LiveData,
    settings: settings::State,
    widgets: Registry,
    notice: String,
    screen_power: crate::screen_power::Status,
    debug_flag: bool,
    windowed: bool,
}

impl App {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        config: Config,
        startup: Startup,
    ) -> Result<Self> {
        let Startup {
            config_path,
            cache_path,
            debug_flag,
            windowed,
            first_run,
            load_error,
        } = startup;
        theme::install(&context.egui_ctx);
        let runtime = Runtime::start(
            config.clone(),
            cache_path,
            debug_flag,
            context.egui_ctx.clone(),
        )?;
        let settings = settings::State::new(config.clone(), first_run, load_error.is_some());
        Ok(Self {
            generation: 0,
            config,
            config_path,
            runtime,
            settings,
            data: LiveData::default(),
            widgets: Registry::default(),
            notice: load_error.unwrap_or_default(),
            screen_power: crate::screen_power::Status::Checking,
            debug_flag,
            windowed,
        })
    }

    fn open_settings(&mut self) {
        self.settings.open(&self.config);
        self.widgets.reset();
    }

    fn handle_action(&mut self, action: UiAction, context: &egui::Context) {
        match action {
            UiAction::OpenSettings => self.open_settings(),
            UiAction::SaveSettings => self.save_settings(context),
            UiAction::CancelSettings => {
                self.widgets.reset();
                self.settings.draft = self.config.clone();
                self.settings.tab = None;
                self.settings.test_results.clear();
                self.settings.replace_invalid = false;
            }
            UiAction::Quit => context.send_viewport_cmd(egui::ViewportCommand::Close),
            UiAction::Refresh => self.runtime.send(Command::Refresh),
            UiAction::ClearCache => self.runtime.send(Command::ClearCache),
            UiAction::Request(request) => {
                self.notice = format!("Loading {}…", request.provider().name());
                self.runtime.send(Command::Request(request));
            }
            UiAction::TestConnection(provider) => self.test_connection(provider),
            UiAction::SettingsChanged(provider) => {
                self.settings.test_results.remove(&provider);
                self.notice.clear();
            }
            UiAction::Notice(message) => self.notice = message,
        }
        context.request_repaint();
    }

    fn save_settings(&mut self, context: &egui::Context) {
        if self.settings.invalid_file && !self.settings.replace_invalid {
            return;
        }
        if let Err(error) = self.settings.draft.save(&self.config_path) {
            self.notice = format!("Could not save: {error}");
            return;
        }
        self.config = self.settings.draft.clone();
        self.generation += 1;
        self.runtime
            .send(Command::ApplyConfig(Box::new(self.config.clone())));
        self.data.clear();
        self.widgets.reset();
        self.settings.invalid_file = false;
        self.settings.replace_invalid = false;
        self.settings.tab = None;
        self.notice.clear();
        context.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
            self.config.fullscreen && !self.windowed,
        ));
    }

    fn test_connection(&mut self, provider: Provider) {
        let request = Request::configured(provider, &self.settings.draft);
        if let Err(error) = request.validate_test() {
            self.notice = error;
            return;
        }
        self.settings.test_results.remove(&provider);
        self.notice = format!("Testing {}…", provider.name());
        self.runtime.send(Command::TestConnection(request));
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.receive();
        let appearance = if self.settings.tab.is_some() {
            &self.settings.draft
        } else {
            &self.config
        };
        theme::apply(ui.ctx(), appearance);
        ui.set_style(ui.ctx().global_style());
        if ui.input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::Comma)) {
            self.open_settings();
        }
        let action = egui::Frame::central_panel(ui.style())
            .inner_margin(0.0)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                if self.settings.tab.is_some() {
                    let context = settings::Context {
                        saved: &self.config,
                        data: &self.data,
                        notice: &self.notice,
                        screen_power: &self.screen_power,
                        config_path: &self.config_path,
                        windowed: self.windowed,
                        debug_flag: self.debug_flag,
                    };
                    egui::Frame::new()
                        .inner_margin(12.0)
                        .show(ui, |ui| self.settings.show(ui, &context, &mut self.widgets))
                        .inner
                } else {
                    dashboard::show(
                        ui,
                        &self.config,
                        &self.data,
                        &mut self.widgets,
                        &self.notice,
                    )
                }
            })
            .inner;
        if let Some(action) = action {
            self.handle_action(action, ui.ctx());
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}
