mod appearance;
pub(crate) mod controls;
mod general;
mod layout;

use super::{UiAction, dashboard, data::LiveData};
use crate::{
    Result,
    config::{Config, Widget},
    integrations::{Data, Provider},
};
use eframe::egui;
use std::{collections::HashMap, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    General,
    Appearance,
    Widget(Widget),
    Layout,
}

impl Tab {
    pub fn all() -> Vec<Self> {
        let mut tabs = vec![Self::General, Self::Appearance];
        tabs.extend(
            crate::widgets::registry::REGISTERED
                .iter()
                .map(|registration| Self::Widget(registration.kind)),
        );
        tabs.push(Self::Layout);
        tabs
    }

    fn name(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Widget(kind) => kind.name(),
            Self::Layout => "Layout",
        }
    }
}

pub struct State {
    pub draft: Config,
    pub tab: Option<Tab>,
    pub invalid_file: bool,
    pub replace_invalid: bool,
    pub test_results: HashMap<Provider, Result<Data>>,
}

pub struct Context<'a> {
    pub saved: &'a Config,
    pub data: &'a LiveData,
    pub notice: &'a str,
    pub screen_power: &'a crate::screen_power::Status,
    pub config_path: &'a Path,
    pub windowed: bool,
    pub debug_flag: bool,
}

impl State {
    pub fn new(config: Config, first_run: bool, invalid_file: bool) -> Self {
        Self {
            draft: config,
            tab: first_run.then_some(Tab::General),
            invalid_file,
            replace_invalid: false,
            test_results: HashMap::new(),
        }
    }

    pub fn open(&mut self, config: &Config) {
        self.draft = config.clone();
        self.tab = Some(Tab::General);
        self.test_results.clear();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        context: &Context<'_>,
        widgets: &mut crate::widgets::registry::Registry,
    ) -> Option<UiAction> {
        let mut action = None;
        ui.heading("Dashboard settings");
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.invalid_file || self.replace_invalid,
                    egui::Button::new("Save"),
                )
                .clicked()
            {
                action = Some(UiAction::SaveSettings);
            }
            if ui.button("Cancel").clicked() {
                action = Some(UiAction::CancelSettings);
            }
            if ui.button("Quit").clicked() {
                action = Some(UiAction::Quit);
            }
            ui.weak("Changes are applied after Save.");
        });
        if self.invalid_file {
            ui.label("The existing configuration is invalid and has been left untouched.");
            ui.checkbox(
                &mut self.replace_invalid,
                "Replace the invalid configuration when I click Save",
            );
        }
        if !context.notice.is_empty() {
            ui.label(context.notice);
        }
        ui.horizontal_wrapped(|ui| {
            for tab in Tab::all() {
                if ui
                    .selectable_label(self.tab == Some(tab), tab.name())
                    .clicked()
                {
                    self.tab = Some(tab);
                    if let Tab::Widget(kind) = tab
                        && let Some(opened) = crate::widgets::registry::for_kind(kind)
                            .and_then(|registration| registration.settings_opened)
                    {
                        action = Some(opened());
                    }
                }
            }
        });
        ui.separator();
        let tab = self.tab?;
        let available = ui.available_size();
        let form_action = if available.x >= 1100.0 {
            ui.columns(2, |columns| {
                let result = egui::ScrollArea::vertical()
                    .id_salt("settings-form")
                    .max_height(available.y)
                    .show(&mut columns[0], |ui| {
                        self.show_tab(ui, tab, context, widgets)
                    })
                    .inner;
                columns[1].heading("Live preview");
                egui::ScrollArea::both()
                    .id_salt("settings-preview")
                    .max_height((available.y - 30.0).max(50.0))
                    .show(&mut columns[1], |ui| {
                        dashboard::render_layout(ui, &self.draft, context.data, widgets, true)
                    });
                result
            })
        } else {
            egui::ScrollArea::both()
                .id_salt("settings-form")
                .show(ui, |ui| {
                    let result = self.show_tab(ui, tab, context, widgets);
                    ui.separator();
                    egui::CollapsingHeader::new("Live preview")
                        .default_open(tab == Tab::Layout || tab == Tab::Appearance)
                        .show(ui, |ui| {
                            dashboard::render_layout(ui, &self.draft, context.data, widgets, true)
                        });
                    result
                })
                .inner
        };
        action.or(form_action)
    }

    fn show_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: Tab,
        context: &Context<'_>,
        widgets: &mut crate::widgets::registry::Registry,
    ) -> Option<UiAction> {
        match tab {
            Tab::General => general::show(ui, &mut self.draft, context),
            Tab::Appearance => {
                appearance::show(ui, &mut self.draft);
                None
            }
            Tab::Widget(kind) => widgets.settings(kind, ui, self, context),
            Tab::Layout => {
                layout::show(ui, &mut self.draft.layout);
                None
            }
        }
    }
}
