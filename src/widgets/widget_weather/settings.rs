use crate::ui::settings::controls::text;
use crate::{
    config::WeatherConfig,
    integrations::{
        Provider, Request,
        weather::{Location, Weather},
    },
    ui::UiAction,
};
use eframe::egui;

#[derive(Default)]
pub struct Editor {
    pub query: String,
    pub results: Vec<Location>,
    pub status: String,
}

impl Editor {
    pub fn accept_results(&mut self, key: &str, results: &[Location]) -> bool {
        if Request::search(&self.query)
            .map(|request| request.key())
            .as_deref()
            != Some(key)
        {
            return false;
        }
        self.status = if results.is_empty() {
            "No matching locations".into()
        } else {
            String::new()
        };
        self.results = results.to_vec();
        true
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut WeatherConfig,
        test_result: Option<&Weather>,
    ) -> Option<UiAction> {
        let mut action = None;
        ui.heading("Weather");
        ui.checkbox(&mut config.enabled, "Enable weather");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("City or postal code")
                    .desired_width(210.0),
            );
            if ui
                .add_enabled(
                    self.query.trim().chars().count() >= 2,
                    egui::Button::new("Search"),
                )
                .clicked()
            {
                self.results.clear();
                self.status.clear();
                action = Request::search(&self.query).map(UiAction::Request);
            }
        });
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
        for location in self.results.clone() {
            if ui.button(location.label()).clicked() {
                config.label = location.name.clone();
                config.latitude = location.latitude;
                config.longitude = location.longitude;
                config.enabled = true;
                self.query.clear();
                self.results.clear();
                self.status.clear();
                action = Some(UiAction::SettingsChanged(Provider::Weather));
            }
        }
        if config.enabled || !config.label.trim().is_empty() {
            ui.group(|ui| {
                let name = if config.label.trim().is_empty() {
                    "Manual location"
                } else {
                    &config.label
                };
                ui.label(format!("Selected location: {name}"));
                ui.label(format!(
                    "Latitude: {:.5} · Longitude: {:.5}",
                    config.latitude, config.longitude
                ));
                if ui.button("Clear location").clicked() {
                    config.enabled = false;
                    config.label.clear();
                    config.latitude = 0.0;
                    config.longitude = 0.0;
                    self.query.clear();
                    self.results.clear();
                    self.status.clear();
                    action = Some(UiAction::SettingsChanged(Provider::Weather));
                }
            });
        } else {
            ui.label("No location selected.");
        }
        text(ui, "Display name", &mut config.label);
        ui.horizontal(|ui| {
            ui.label("Latitude");
            ui.add(
                egui::DragValue::new(&mut config.latitude)
                    .speed(0.001)
                    .range(-90.0..=90.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Longitude");
            ui.add(
                egui::DragValue::new(&mut config.longitude)
                    .speed(0.001)
                    .range(-180.0..=180.0),
            );
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut config.fahrenheit, false, "Celsius");
            ui.selectable_value(&mut config.fahrenheit, true, "Fahrenheit");
        });
        if ui.button("Test weather connection").clicked() {
            action = Some(UiAction::TestConnection(Provider::Weather));
        }
        if let Some(weather) = test_result {
            ui.label(format!(
                "Temperature: {:.1} · {}",
                weather.temperature,
                crate::integrations::weather::condition(weather.code)
            ));
        }
        ui.hyperlink_to(
            "Open-Meteo · free API for non-commercial use",
            "https://open-meteo.com/en/pricing",
        );
        action
    }
}
