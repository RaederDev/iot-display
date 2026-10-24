use crate::{
    config::Config,
    integrations::{Data, Provider},
    ui::data::LiveData,
};
use eframe::egui::{self, Align, FontId, RichText};

pub fn show(ui: &mut egui::Ui, config: &Config, live: &LiveData) {
    if !config.pi_hole.enabled {
        ui.label("Pi-hole: open Settings to configure the API.");
        return;
    }
    ui.vertical_centered(|ui| {
        ui.add(
            egui::Image::new(egui::include_image!("assets/pi-hole.svg"))
                .fit_to_exact_size(egui::vec2(70.0, 70.0)),
        );
    });
    ui.add_space(20.0);
    if let Some(entry) = live.entry(Provider::PiHole, config) {
        if let Data::PiHole(data) = &entry.data {
            let stats = [
                ("DNS queries", data.queries),
                ("Blocked queries", data.blocked),
                ("Unique domains", data.unique_domains),
            ];
            pi_hole_stats(ui, &stats);
            if live.error(Provider::PiHole, config).is_some() {
                crate::widgets::common::freshness(ui, Provider::PiHole, config, live, entry);
            }
        }
    } else {
        crate::widgets::common::loading(ui, Provider::PiHole, config, live);
    }
}

fn pi_hole_stats(ui: &mut egui::Ui, stats: &[(&str, u64)]) {
    let width = ui.available_width();
    let value_width = stats
        .iter()
        .map(|(_, value)| {
            ui.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(
                        value.to_string(),
                        FontId::proportional(18.0),
                        ui.visuals().text_color(),
                    )
                    .size()
                    .x
            })
        })
        .fold(width * 0.25, f32::max)
        .min(width);
    let label_width = (width - value_width - ui.spacing().item_spacing.x).max(0.0);
    for (label, value) in stats {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(label_width, 24.0),
                egui::Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_min_size(egui::vec2(label_width, 24.0));
                    ui.add(egui::Label::new(RichText::new(*label).size(18.0)).truncate());
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(value_width, 24.0),
                egui::Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_min_size(egui::vec2(value_width, 24.0));
                    ui.label(RichText::new(value.to_string()).size(18.0));
                },
            );
        });
    }
}
