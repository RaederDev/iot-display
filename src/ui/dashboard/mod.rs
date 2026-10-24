mod toolbar;

use super::{UiAction, data::LiveData};
use crate::config::Config;
use crate::widgets::registry::Registry;
use eframe::egui::{self, Align};

pub fn show(
    ui: &mut egui::Ui,
    config: &Config,
    data: &LiveData,
    state: &mut Registry,
    notice: &str,
) -> Option<UiAction> {
    let action = toolbar::show(ui, config, data, state);
    if !notice.is_empty() {
        ui.label(notice);
    }
    egui::ScrollArea::both()
        .id_salt("dashboard-scroll")
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(config.spacing)
                .show(ui, |ui| render_layout(ui, config, data, state, false));
        });
    action
}

pub fn render_layout(
    ui: &mut egui::Ui,
    config: &Config,
    data: &LiveData,
    state: &mut Registry,
    preview: bool,
) {
    for (row_index, row) in config.layout.iter().enumerate() {
        let spacing = config.spacing;
        let width = ui.available_width().max(
            row.columns.len() as f32 * 160.0 + spacing * row.columns.len().saturating_sub(1) as f32,
        );
        let weights: f32 = row.columns.iter().map(|column| column.weight).sum();
        ui.spacing_mut().item_spacing.x = spacing;
        ui.horizontal_top(|ui| {
            for (column_index, column) in row.columns.iter().enumerate() {
                let column_width = ((width - spacing * row.columns.len().saturating_sub(1) as f32)
                    * column.weight
                    / weights)
                    .max(1.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(column_width, 0.0),
                    egui::Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(column_width);
                        for (widget_index, widget) in column.widgets.iter().enumerate() {
                            let id = (row_index, column_index, widget_index);
                            ui.push_id((preview, id), |ui| {
                                state.show(
                                    *widget,
                                    ui,
                                    &crate::widgets::registry::RenderContext {
                                        config,
                                        data,
                                        id,
                                        preview,
                                    },
                                )
                            });
                            ui.add_space(spacing);
                        }
                        if preview && column.widgets.is_empty() {
                            ui.weak("Empty column");
                        }
                    },
                );
            }
        });
    }
}
