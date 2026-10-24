use crate::ui::settings::controls::text;
use crate::{
    Result,
    config::{DeparturesConfig, StopConfig, StopKind},
    integrations::{
        Provider,
        departures::{self, Board, Stop},
    },
    ui::UiAction,
};
use eframe::egui::{self, RichText};
use std::collections::HashMap;

enum StopEdit {
    Move { from: usize, to: usize },
    Remove(usize),
}

pub struct Editor {
    pub query: String,
    pub manual_stop: u32,
    pub manual_kind: StopKind,
    pub line_inputs: HashMap<u32, String>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            query: String::new(),
            manual_stop: 0,
            manual_kind: StopKind::Diva,
            line_inputs: HashMap::new(),
        }
    }
}

impl Editor {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut DeparturesConfig,
        catalogue: &[Stop],
        discovered: &[Board],
    ) -> Option<UiAction> {
        let mut action = None;
        ui.heading("Wiener Linien departures");
        ui.checkbox(&mut config.enabled, "Enable departures");
        ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search stop name or DIVA number"),
        );
        let search = self.query.trim().to_lowercase();
        if !search.is_empty() {
            let matches: Vec<_> = catalogue
                .iter()
                .filter(|stop| {
                    stop.name.to_lowercase().contains(&search) || stop.diva.to_string() == search
                })
                .collect();
            ui.weak(format!("{} matches; showing up to 30.", matches.len()));
            for stop in matches.into_iter().take(30) {
                if ui
                    .button(format!("Add {} · DIVA {}", stop.name, stop.diva))
                    .clicked()
                    && let Err(error) =
                        add_stop(config, stop.diva, stop.name.clone(), StopKind::Diva)
                {
                    action = Some(UiAction::Notice(error));
                }
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Manual identifier");
            stop_kind(ui, "manual-stop-kind", &mut self.manual_kind);
            ui.add(egui::DragValue::new(&mut self.manual_stop).range(1..=u32::MAX));
            if ui
                .add_enabled(self.manual_stop > 0, egui::Button::new("Add stop"))
                .clicked()
                && let Err(error) =
                    add_stop(config, self.manual_stop, String::new(), self.manual_kind)
            {
                action = Some(UiAction::Notice(error));
            }
        });
        if ui.button("Test stops / discover lines").clicked() {
            action = Some(UiAction::TestConnection(Provider::Departures));
        }
        let count = config.stops.len();
        let mut change = None;
        for (index, stop) in config.stops.iter_mut().enumerate() {
            let edit = ui
                .push_id(index, |ui| {
                    self.stop_editor(ui, stop, index, count, discovered)
                })
                .inner;
            if edit.is_some() {
                change = edit;
            }
        }
        match change {
            Some(StopEdit::Move { from, to }) => config.stops.swap(from, to),
            Some(StopEdit::Remove(index)) => {
                config.stops.remove(index);
            }
            None => {}
        }
        ui.hyperlink_to("Datenquelle: Stadt Wien", "https://data.wien.gv.at");
        action
    }

    fn stop_editor(
        &mut self,
        ui: &mut egui::Ui,
        stop: &mut StopConfig,
        index: usize,
        count: usize,
        discovered: &[Board],
    ) -> Option<StopEdit> {
        let mut edit = None;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Stop {}", stop.id)).strong());
                if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                    edit = Some(StopEdit::Move {
                        from: index,
                        to: index - 1,
                    });
                }
                if ui
                    .add_enabled(index + 1 < count, egui::Button::new("Down"))
                    .clicked()
                {
                    edit = Some(StopEdit::Move {
                        from: index,
                        to: index + 1,
                    });
                }
                if ui.button("Remove").clicked() {
                    edit = Some(StopEdit::Remove(index));
                }
            });
            ui.horizontal(|ui| {
                stop_kind(ui, "stop-kind", &mut stop.kind);
                ui.add(egui::DragValue::new(&mut stop.id).range(1..=u32::MAX));
            });
            text(ui, "Display name", &mut stop.label);
            self.line_filters(ui, stop, discovered);
        });
        edit
    }

    fn line_filters(&mut self, ui: &mut egui::Ui, stop: &mut StopConfig, discovered: &[Board]) {
        let lines = self
            .line_inputs
            .entry(stop.id)
            .or_insert_with(|| stop.lines.join(", "));
        ui.label("Line filter (case insensitive; empty = all)");
        if ui
            .add(
                egui::TextEdit::singleline(lines)
                    .hint_text("12a, 14a")
                    .desired_width(300.0),
            )
            .changed()
        {
            stop.lines = lines
                .split(',')
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect();
        }
        if let Some(board) = departures::board_for_stop(discovered, stop) {
            ui.weak(&board.title);
            ui.horizontal_wrapped(|ui| {
                for line in &board.available_lines {
                    let mut selected = departures::line_selected(&stop.lines, line);
                    if ui.checkbox(&mut selected, line).changed() {
                        if selected {
                            stop.lines.push(line.clone());
                        } else {
                            stop.lines
                                .retain(|existing| !existing.eq_ignore_ascii_case(line));
                        }
                        self.line_inputs.insert(stop.id, stop.lines.join(", "));
                    }
                }
            });
            if ui.button("Show all lines").clicked() {
                stop.lines.clear();
                self.line_inputs.insert(stop.id, String::new());
            }
        } else {
            ui.weak("No discovery data for this stop yet. Manual line entry remains available.");
        }
    }
}

fn add_stop(config: &mut DeparturesConfig, id: u32, label: String, kind: StopKind) -> Result<()> {
    if config.stops.iter().any(|stop| stop.id == id) {
        return Err("This stop is already configured".into());
    }
    config.stops.push(StopConfig {
        id,
        kind,
        label,
        lines: vec![],
    });
    config.enabled = true;
    Ok(())
}

fn stop_kind(ui: &mut egui::Ui, id: &str, kind: &mut StopKind) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(match kind {
            StopKind::Diva => "DIVA",
            StopKind::StopId => "Stopping-point ID",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(kind, StopKind::Diva, "DIVA");
            ui.selectable_value(kind, StopKind::StopId, "Stopping-point ID");
        });
}
