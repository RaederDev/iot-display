use crate::ui::settings::controls::text;
use crate::{
    config::PiHoleConfig,
    integrations::{Provider, pi_hole::Stats},
    ui::UiAction,
};
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    config: &mut PiHoleConfig,
    test_result: Option<&Stats>,
) -> Option<UiAction> {
    let mut action = None;
    ui.heading("Pi-hole v6");
    ui.checkbox(&mut config.enabled, "Enable Pi-hole");
    text(ui, "API URL (include /api)", &mut config.api_url);
    ui.label("Application password");
    ui.add(egui::TextEdit::singleline(&mut config.password).password(true));
    ui.weak("Use an application password for unattended access, especially with 2FA. Stored in your private configuration file.");
    if ui.button("Test Pi-hole connection").clicked() {
        action = Some(UiAction::TestConnection(Provider::PiHole));
    }
    if let Some(stats) = test_result {
        ui.label(format!(
            "Queries: {} · Blocked: {} · Unique domains: {}",
            stats.queries, stats.blocked, stats.unique_domains
        ));
    }
    ui.hyperlink_to("Pi-hole", "https://pi-hole.net/");
    action
}
