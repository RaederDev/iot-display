#[cfg(not(target_os = "linux"))]
compile_error!("iot-display supports Linux only");
pub mod cache;
pub mod config;
pub mod http;
pub mod integrations;
pub mod runtime;
pub mod screen_power;
pub mod storage;
pub mod ui;
pub mod widgets;

pub type Result<T> = std::result::Result<T, String>;
