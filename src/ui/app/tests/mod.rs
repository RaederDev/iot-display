use super::*;
use crate::integrations::Data;
use crate::{cache::Entry, config::StopKind, ui::settings::Tab};

mod departures;
mod settings;
mod weather;

pub(super) fn frame(
    app: &mut App,
    context: &egui::Context,
    size: egui::Vec2,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| app.draw(ui),
    );
    output.textures_delta.clear();
    output
}

pub(super) fn label_position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Missing label {label}"))
}

pub(super) fn click(app: &mut App, context: &egui::Context, label: &str, size: egui::Vec2) {
    let output = frame(app, context, size, vec![]);
    let position = label_position(&output, label);
    for pressed in [true, false] {
        frame(
            app,
            context,
            size,
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
}
