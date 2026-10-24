use crate::{
    config::Config,
    integrations::{Data, Provider, weather},
    ui::{data::LiveData, theme::BLUE},
};
use eframe::egui::{self, Color32, RichText};

pub fn show(ui: &mut egui::Ui, config: &Config, live: &LiveData) {
    if !config.weather.enabled {
        ui.label("Weather: open Settings to choose a location.");
        return;
    }
    if let Some(entry) = live.entry(Provider::Weather, config) {
        if let Data::Weather(data) = &entry.data {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(
                    data.seconds_until_next_day(chrono::Utc::now()),
                ));
            let unit = if config.weather.fahrenheit {
                "°F"
            } else {
                "°C"
            };
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&config.weather.label).size(26.0));
                    ui.label(RichText::new(format!("{:.0}{unit}", data.temperature)).size(36.0));
                    ui.label(format!(
                        "{}. Feels like {:.0}{unit}",
                        weather::condition(data.code),
                        data.apparent
                    ));
                    if data.current_day(chrono::Utc::now()) {
                        if let Some(high) = data.high {
                            ui.label(format!("High: {high:.0}{unit}"));
                        }
                        if let Some(low) = data.low {
                            ui.label(format!("Low: {low:.0}{unit}"));
                        }
                    } else {
                        ui.weak("Daily forecast needs updating");
                    }
                });
                weather_icon(ui, data.code, data.is_day);
            });
            crate::widgets::common::freshness(ui, Provider::Weather, config, live, entry);
        }
    } else {
        crate::widgets::common::loading(ui, Provider::Weather, config, live);
    }
}

fn weather_icon(ui: &mut egui::Ui, code: u16, is_day: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(60.0, 70.0), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    if code <= 2 {
        painter.circle_filled(
            center + egui::vec2(10.0, -10.0),
            17.0,
            if is_day {
                Color32::from_rgb(248, 183, 57)
            } else {
                Color32::from_rgb(160, 180, 220)
            },
        );
    }
    if code > 0 {
        let cloud = Color32::from_rgb(150, 180, 200);
        for offset in [-15.0, 0.0, 15.0] {
            painter.circle_filled(center + egui::vec2(offset, 3.0), 15.0, cloud);
        }
    }
    if code >= 51 {
        for offset in [-12.0, 0.0, 12.0] {
            painter.line_segment(
                [
                    center + egui::vec2(offset, 25.0),
                    center + egui::vec2(offset - 4.0, 33.0),
                ],
                egui::Stroke::new(3.0, BLUE),
            );
        }
    }
}
