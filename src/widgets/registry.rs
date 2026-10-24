use super::{
    backend::Backend, widget_clock, widget_departures, widget_greeting, widget_network,
    widget_pihole, widget_weather,
};
use crate::{
    config::{Config, Widget},
    integrations::{Provider, Request},
    runtime::Event,
    ui::{UiAction, data::LiveData, settings},
};
use eframe::egui;
use std::{any::Any, collections::HashMap};

pub type WidgetId = (usize, usize, usize);

pub struct RenderContext<'a> {
    pub config: &'a Config,
    pub data: &'a LiveData,
    pub id: WidgetId,
    pub preview: bool,
}

pub trait View: Any {
    fn show(&mut self, ui: &mut egui::Ui, context: &RenderContext<'_>);
    fn settings(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut settings::State,
        context: &settings::Context<'_>,
    ) -> Option<UiAction>;

    fn status(&mut self, _ui: &mut egui::Ui, _config: &Config, _data: &LiveData) {}

    fn event(&mut self, _event: &Event) {}

    fn reset(&mut self) {}
}

pub type BackendFactory = fn(&Config) -> Box<dyn Backend>;

pub struct Registration {
    pub kind: Widget,
    pub name: &'static str,
    pub key: fn(&Request) -> String,
    pub fonts: Option<fn(&mut egui::FontDefinitions)>,
    pub providers: &'static [Provider],
    pub view: fn() -> Box<dyn View>,
    pub backend: Option<BackendFactory>,
    pub requests: fn(&Config) -> Vec<Request>,
    pub refresh_seconds: fn(&Request) -> u64,
    pub card: bool,
    pub background: Option<egui::Color32>,
    pub foreground: Option<egui::Color32>,
    pub validate: fn(&Config) -> crate::Result<()>,
    pub settings_opened: Option<fn() -> UiAction>,
}

// This is the only registration list. Rendering, settings, and background tasks all use it.
pub static REGISTERED: &[Registration] = &[
    widget_greeting::REGISTRATION,
    widget_weather::REGISTRATION,
    widget_pihole::REGISTRATION,
    widget_departures::REGISTRATION,
    widget_clock::REGISTRATION,
    widget_network::REGISTRATION,
];

pub fn for_kind(kind: Widget) -> Option<&'static Registration> {
    REGISTERED
        .iter()
        .find(|registration| registration.kind == kind)
}

pub fn for_provider(provider: Provider) -> &'static Registration {
    REGISTERED
        .iter()
        .find(|registration| registration.providers.contains(&provider))
        .expect("Every provider has a registered widget")
}

pub struct Registry {
    views: HashMap<Widget, Box<dyn View>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            views: REGISTERED
                .iter()
                .map(|registration| (registration.kind, (registration.view)()))
                .collect(),
        }
    }
}

impl Registry {
    pub fn show(&mut self, kind: Widget, ui: &mut egui::Ui, context: &RenderContext<'_>) {
        let Some(registration) = for_kind(kind) else {
            ui.weak("Widget unavailable");
            return;
        };
        let Some(view) = self.views.get_mut(&kind) else {
            return;
        };
        if !registration.card {
            view.show(ui, context);
            return;
        }
        let fill = registration.background.unwrap_or_else(|| {
            if context.config.dark {
                egui::Color32::from_rgb(50, 50, 50)
            } else {
                egui::Color32::WHITE
            }
        });
        egui::Frame::new()
            .fill(fill)
            .corner_radius(3)
            .inner_margin(16.0)
            .shadow(egui::epaint::Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: egui::Color32::from_black_alpha(35),
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_width().max(100.0));
                if let Some(color) = registration.foreground {
                    ui.visuals_mut().override_text_color = Some(color);
                }
                view.show(ui, context);
            });
    }

    pub fn settings(
        &mut self,
        kind: Widget,
        ui: &mut egui::Ui,
        state: &mut settings::State,
        context: &settings::Context<'_>,
    ) -> Option<UiAction> {
        self.views.get_mut(&kind)?.settings(ui, state, context)
    }

    pub fn status(&mut self, ui: &mut egui::Ui, config: &Config, data: &LiveData) {
        for registration in REGISTERED {
            if let Some(view) = self.views.get_mut(&registration.kind) {
                view.status(ui, config, data);
            }
        }
    }

    pub fn event(&mut self, event: &Event) {
        for view in self.views.values_mut() {
            view.event(event);
        }
    }

    pub fn reset(&mut self) {
        for view in self.views.values_mut() {
            view.reset();
        }
    }

    #[cfg(test)]
    pub(crate) fn view<T: View>(&mut self, kind: Widget) -> &mut T {
        let view: &mut dyn Any = self.views.get_mut(&kind).unwrap().as_mut();
        view.downcast_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registrations_have_unique_widgets_and_provider_owners() {
        let mut widgets = std::collections::HashSet::new();
        let mut providers = std::collections::HashSet::new();
        for registration in REGISTERED {
            assert!(widgets.insert(registration.kind));
            for provider in registration.providers {
                assert!(providers.insert(provider));
                assert!(registration.backend.is_some());
            }
        }
        for provider in [
            Provider::Weather,
            Provider::Departures,
            Provider::PiHole,
            Provider::Catalogue,
            Provider::Geocoding,
            Provider::Network,
        ] {
            assert!(providers.contains(&provider));
        }
    }
}
