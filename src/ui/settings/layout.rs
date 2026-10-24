use crate::config::{
    Column, Row, Widget,
    layout::{self, Action},
};
use eframe::egui::{self, RichText};

pub fn show(ui: &mut egui::Ui, rows: &mut Vec<Row>) {
    ui.heading("Layout");
    ui.weak("Column widths are proportions. Widget settings and data are shared across copies.");
    let mut action = None;
    if ui.button("Add row").clicked() {
        action = Some(Action::AddRow);
    }
    let row_count = rows.len();
    let destinations: Vec<_> = rows
        .iter()
        .enumerate()
        .flat_map(|(row, value)| (0..value.columns.len()).map(move |column| (row, column)))
        .collect();
    for (index, row) in rows.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            row_editor(ui, row, index, row_count, &destinations, &mut action)
        });
    }
    // Apply edits after rendering so indices stay valid throughout this frame.
    if let Some(action) = action {
        layout::apply(rows, action);
    }
}

fn row_editor(
    ui: &mut egui::Ui,
    row: &mut Row,
    index: usize,
    count: usize,
    destinations: &[(usize, usize)],
    action: &mut Option<Action>,
) {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("Row {}", index + 1)).strong());
            if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
                *action = Some(Action::SwapRows(index, index - 1));
            }
            if ui
                .add_enabled(index + 1 < count, egui::Button::new("Down"))
                .clicked()
            {
                *action = Some(Action::SwapRows(index, index + 1));
            }
            if ui
                .add_enabled(count > 1, egui::Button::new("Remove row"))
                .clicked()
            {
                *action = Some(Action::RemoveRow(index));
            }
            if ui.button("Add column").clicked() {
                *action = Some(Action::AddColumn(index));
            }
        });
        let column_count = row.columns.len();
        for (column_index, column) in row.columns.iter_mut().enumerate() {
            ui.push_id(column_index, |ui| {
                column_editor(
                    ui,
                    column,
                    (index, column_index),
                    column_count,
                    destinations,
                    action,
                )
            });
        }
    });
}

fn column_editor(
    ui: &mut egui::Ui,
    column: &mut Column,
    position: (usize, usize),
    count: usize,
    destinations: &[(usize, usize)],
    action: &mut Option<Action>,
) {
    let (row_index, index) = position;
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Column {}", index + 1));
            ui.label("Width");
            ui.add(
                egui::DragValue::new(&mut column.weight)
                    .speed(0.1)
                    .range(0.1..=100.0),
            );
            if ui
                .add_enabled(index > 0, egui::Button::new("Left"))
                .clicked()
            {
                *action = Some(Action::SwapColumns(row_index, index, index - 1));
            }
            if ui
                .add_enabled(index + 1 < count, egui::Button::new("Right"))
                .clicked()
            {
                *action = Some(Action::SwapColumns(row_index, index, index + 1));
            }
            if ui
                .add_enabled(count > 1, egui::Button::new("Remove column"))
                .clicked()
            {
                *action = Some(Action::RemoveColumn(row_index, index));
            }
        });
        let widget_count = column.widgets.len();
        for (widget_index, widget) in column.widgets.iter_mut().enumerate() {
            ui.push_id(widget_index, |ui| {
                widget_editor(
                    ui,
                    widget,
                    (row_index, index, widget_index),
                    widget_count,
                    destinations,
                    action,
                )
            });
        }
        ui.menu_button("Add widget", |ui| {
            for registration in crate::widgets::registry::REGISTERED {
                let widget = registration.kind;
                if ui.button(widget.name()).clicked() {
                    *action = Some(Action::AddWidget(row_index, index, widget));
                    ui.close();
                }
            }
        });
    });
}

fn widget_editor(
    ui: &mut egui::Ui,
    widget: &mut Widget,
    position: (usize, usize, usize),
    count: usize,
    destinations: &[(usize, usize)],
    action: &mut Option<Action>,
) {
    let (row, column, index) = position;
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_id_salt("widget-type")
            .selected_text(widget.name())
            .show_ui(ui, |ui| {
                for registration in crate::widgets::registry::REGISTERED {
                    let option = registration.kind;
                    ui.selectable_value(widget, option, option.name());
                }
            });
        if ui.add_enabled(index > 0, egui::Button::new("Up")).clicked() {
            *action = Some(Action::SwapWidgets(row, column, index, index - 1));
        }
        if ui
            .add_enabled(index + 1 < count, egui::Button::new("Down"))
            .clicked()
        {
            *action = Some(Action::SwapWidgets(row, column, index, index + 1));
        }
        if ui.button("Remove").clicked() {
            *action = Some(Action::RemoveWidget(row, column, index));
        }
        ui.menu_button("Move to", |ui| {
            for &(target_row, target_column) in destinations {
                if (target_row, target_column) != (row, column)
                    && ui
                        .button(format!(
                            "Row {}, column {}",
                            target_row + 1,
                            target_column + 1
                        ))
                        .clicked()
                {
                    *action = Some(Action::MoveWidget {
                        row,
                        column,
                        widget: index,
                        target_row,
                        target_column,
                    });
                    ui.close();
                }
            }
        });
    });
}
