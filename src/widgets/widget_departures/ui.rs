use crate::{
    config::Config,
    integrations::{
        Data, Provider,
        departures::{self, Selection},
    },
    ui::data::LiveData,
};
use eframe::egui::{self, Align, FontFamily, FontId, RichText};

pub const GREEN: egui::Color32 = egui::Color32::from_rgb(89, 255, 89);

pub enum Navigation {
    Show(Selection),
    Back,
}

pub fn show(
    ui: &mut egui::Ui,
    config: &Config,
    data: &LiveData,
    selection: Option<&Selection>,
    preview: bool,
) -> Option<Navigation> {
    let mut action = None;
    if !config.departures.enabled {
        ui.label("Departures: open Settings to choose stops.");
        return None;
    }
    if let Some(entry) = data.entry(Provider::Departures, config) {
        if let Data::Departures(boards) = &entry.data {
            let detail = if preview { None } else { selection };
            if detail.is_some() && ui.button("Back to all departures").clicked() {
                action = Some(Navigation::Back);
            }
            let mut found = false;
            for stop in &config.departures.stops {
                if detail.is_some_and(|selected| selected.stop_id != stop.id) {
                    continue;
                }
                let Some(board) = departures::board_for_stop(boards, stop) else {
                    continue;
                };

                let departures =
                    departures::display_departures(&board, &stop.lines, chrono::Utc::now(), detail);
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Label::new(board_text(
                            if stop.label.is_empty() {
                                &board.title
                            } else {
                                &stop.label
                            },
                            14.0,
                        ))
                        .truncate(),
                    );
                });
                ui.add_space(8.0);
                for departure in departures {
                    found = true;
                    let time = departures::countdown(departure.at, chrono::Utc::now());
                    let time = if departure.planned {
                        format!("{time} (planned)")
                    } else {
                        time
                    };
                    if departure_row(ui, &departure.line, &departure.destination, &time).clicked()
                        && !preview
                    {
                        action = Some(Navigation::Show(Selection {
                            stop_id: stop.id,
                            line: departure.line.clone(),
                            direction: departure.direction.clone(),
                        }));
                    }
                }
                ui.add_space(12.0);
            }
            if !found {
                ui.vertical_centered(|ui| {
                    ui.label(board_text("*** NO DEPARTURES ***", 14.0));
                });
            }
            crate::widgets::common::freshness(ui, Provider::Departures, config, data, entry);
        }
    } else {
        crate::widgets::common::loading(ui, Provider::Departures, config, data);
    }
    action
}

fn board_text(text: &str, size: f32) -> RichText {
    RichText::new(text.to_uppercase())
        .font(FontId::new(size, FontFamily::Name("departures".into())))
        .color(GREEN)
}

fn departure_row(ui: &mut egui::Ui, line: &str, destination: &str, time: &str) -> egui::Response {
    let time_text: egui::WidgetText = board_text(time, 14.0).into();
    let time_width = time_text
        .clone()
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
        .size()
        .x;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
    // Reserve the countdown before truncating the destination.
    let time_rect = egui::Rect::from_min_max(
        egui::pos2((rect.right() - time_width).max(rect.left()), rect.top()),
        rect.right_bottom(),
    );
    let line_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2((rect.left() + 40.0).min(time_rect.left()), rect.bottom()),
    );
    let destination_left = (line_rect.right() + 12.0).min(time_rect.left());
    let destination_rect = egui::Rect::from_min_max(
        egui::pos2(destination_left, rect.top()),
        egui::pos2(
            (time_rect.left() - 12.0).max(destination_left),
            rect.bottom(),
        ),
    );
    for (cell, text, align) in [
        (line_rect, board_text(line, 14.0).into(), Align::Min),
        (
            destination_rect,
            board_text(destination, 14.0).into(),
            Align::Min,
        ),
        (time_rect, time_text, Align::Max),
    ] {
        let layout = if align == Align::Max {
            egui::Layout::right_to_left(Align::Center)
        } else {
            egui::Layout::left_to_right(Align::Center)
        };
        let mut cell_ui = ui.new_child(egui::UiBuilder::new().max_rect(cell).layout(layout));
        cell_ui.set_clip_rect(ui.clip_rect().intersect(cell));
        cell_ui.add(
            egui::Label::new(text)
                .truncate()
                .halign(align)
                .selectable(false),
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            ui.is_enabled(),
            format!("{line} {destination} {time}"),
        )
    });
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, GREEN),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_text(format!("{line} · {destination} · {time}"))
}

impl LiveData {
    pub fn catalogue(&self, config: &Config) -> &[crate::widgets::widget_departures::data::Stop] {
        match self
            .entry(Provider::Catalogue, config)
            .map(|entry| &entry.data)
        {
            Some(Data::Catalogue(stops)) => stops,
            _ => &[],
        }
    }

    pub fn departures(&self, config: &Config) -> &[crate::widgets::widget_departures::data::Board] {
        match self
            .entry(Provider::Departures, config)
            .map(|entry| &entry.data)
        {
            Some(Data::Departures(boards)) => boards,
            _ => &[],
        }
    }
}

pub fn install_fonts(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        "DSEG".into(),
        egui::FontData::from_static(include_bytes!("assets/DSEG14Classic-Regular.ttf")).into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("departures".into()),
        vec!["DSEG".into(), "Roboto".into()],
    );
}
