use eframe::egui;

pub(crate) fn text(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).desired_width(300.0))
        .changed()
}
