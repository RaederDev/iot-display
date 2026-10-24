mod actions;
mod app;
mod dashboard;
pub(crate) mod data;
pub(crate) mod settings;
pub(crate) mod theme;

pub(crate) use actions::UiAction;
pub use app::{App, Startup};
