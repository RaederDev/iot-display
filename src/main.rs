#![allow(non_snake_case)]
use std::sync::OnceLock;

use crate::config::Configuration;
use crate::dashboard::Dashboard;
use dioxus::prelude::*;
use dioxus_desktop::{tao::window::Fullscreen, Config, LogicalSize, WindowBuilder};

mod cache;
mod config;
mod dashboard;
mod top_bar;
mod widgets;

static APP_CONFIG: OnceLock<Configuration> = OnceLock::new();

fn main() {
    let config = Configuration::new();
    match config {
        Ok(config) => {
            config.validate().unwrap();
            APP_CONFIG.set(config).unwrap();
            let window_config = &APP_CONFIG.get().unwrap().window;
            let mut window = WindowBuilder::new()
                .with_title("IOT Display")
                .with_inner_size(LogicalSize::new(window_config.width, window_config.height));
            if window_config.fullscreen {
                window = window.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }

            LaunchBuilder::desktop()
                .with_cfg(
                    Config::new()
                        .with_custom_head(
                            r#"<link rel="stylesheet" href="/public/tailwind.css">"#.to_string(),
                        )
                        .with_window(window),
                )
                .launch(Dashboard);
        }
        Err(e) => eprintln!("{}", e),
    }
}
