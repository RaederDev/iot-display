use crate::integrations::{Provider, Request};

/// Views describe intent; the application performs I/O after rendering.
pub enum UiAction {
    OpenSettings,
    SaveSettings,
    CancelSettings,
    Quit,
    Refresh,
    ClearCache,
    Request(Request),
    TestConnection(Provider),
    SettingsChanged(Provider),
    Notice(String),
}
