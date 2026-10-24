use crate::{
    config::Config,
    ui::settings::{Context, controls::text},
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, config: &mut Config, context: &Context<'_>) {
    ui.heading("Network");
    ui.checkbox(&mut config.show_ip, "Show IP address");
    let interfaces = context
        .data
        .network(context.saved)
        .map(|network| network.interfaces.as_slice())
        .unwrap_or_default();
    egui::ComboBox::from_id_salt("network-interface")
        .selected_text(if config.network_interface.is_empty() {
            "Automatic"
        } else {
            &config.network_interface
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut config.network_interface, String::new(), "Automatic");
            let mut names = std::collections::HashSet::new();
            for interface in interfaces {
                if names.insert(interface.name.clone()) {
                    ui.selectable_value(
                        &mut config.network_interface,
                        interface.name.clone(),
                        format!("{} ({})", interface.name, interface.address),
                    );
                }
            }
        });
    text(
        ui,
        "Interface name (manual fallback)",
        &mut config.network_interface,
    );
}
