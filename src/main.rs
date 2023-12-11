#![allow(non_snake_case)]
use std::sync::OnceLock;

use crate::config::Configuration;
use crate::dashboard::Dashboard;
use dioxus_desktop::Config;

mod config;
mod dashboard;
mod widgets;

static APP_CONFIG: OnceLock<Configuration> = OnceLock::new();

fn main() {
    let config = Configuration::new();
    match config {
        Ok(config) => {
            config.validate().unwrap();
            println!("{:?}", config);
            APP_CONFIG.set(config).unwrap();
            dioxus_desktop::launch_cfg(
                Dashboard,
                Config::new().with_custom_head(
                    r#"<link rel="stylesheet" href="public/tailwind.css">"#.to_string(),
                ),
            );
        }
        Err(e) => eprintln!("{}", e),
    }
}
