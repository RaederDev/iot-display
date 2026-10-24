use crate::{
    cache::Entry,
    config::Config,
    integrations::{Provider, Request},
    ui::data::LiveData,
};
use eframe::egui;

pub(crate) fn loading(ui: &mut egui::Ui, provider: Provider, config: &Config, data: &LiveData) {
    if let Some(error) = data.error(provider, config) {
        ui.label(format!("Unavailable: {}", error.message));
    } else {
        ui.label("Loading…");
    }
}

pub(crate) fn freshness(
    ui: &mut egui::Ui,
    provider: Provider,
    config: &Config,
    data: &LiveData,
    entry: &Entry,
) {
    let now = chrono::Utc::now().timestamp();
    let age = now.saturating_sub(entry.fetched_at).max(0);
    if let Some(error) = data.error(provider, config) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));
        ui.label(format!(
            "Cached data ({age}s old) · {} · retry in {}s",
            error.message,
            (error.retry_at - now).max(0)
        ));
    } else if !entry.fresh(now, Request::configured(provider, config).refresh_seconds()) {
        ui.label(format!("Stale data ({age}s old) · refreshing…"));
    }
}
