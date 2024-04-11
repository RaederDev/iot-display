use dioxus::core_macro::rsx;
use dioxus::prelude::*;

use crate::{config::PositionConfig, top_bar::TopBar, widgets::widget::WidgetFactory, APP_CONFIG};

const COLUMN: &str = "relative py-4 px-4 rounded-sm shadow-md bg-white mt-5 first:mt-0";

pub fn Dashboard() -> Element {
    let widgets_left = use_signal(|| {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Left)
    });
    let widgets_center = use_signal(|| {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Center)
    });
    let widgets_right = use_signal(|| {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Right)
    });
    let grid_config = use_signal(|| {
        let grid = &APP_CONFIG.get().unwrap().grid;
        (
            format!("col-span-{}", grid.left),
            format!("col-span-{}", grid.center),
            format!("col-span-{}", grid.right),
        )
    });

    rsx! {
        div {
            class: "min-h-screen bg-lightgray",
            TopBar {}
            div {
                class: "p-5 grid grid-cols-12 gap-x-5 font-sans pt-16",
                div {
                    class: "{grid_config().0}",
                    for (i, widget) in widgets_left.iter().enumerate() {
                        div {
                            class: COLUMN,
                            key: "{i}",
                            { widget.render() }
                        }
                    }
                },
                div {
                    class: "{grid_config().1}",
                    for (i, widget) in widgets_center.iter().enumerate() {
                        div {
                            class: COLUMN,
                            key: "{i}",
                            { widget.render() }
                        }
                    }
                },
                div {
                    class: "{grid_config().2}",
                    for (i, widget) in widgets_right.iter().enumerate() {
                        div {
                            class: COLUMN,
                            key: "{i}",
                        { widget.render() }
                        }
                    }
                },
            }
        }
    }
}
