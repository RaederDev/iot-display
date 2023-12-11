use dioxus::prelude::*;
use dioxus::{
    core::{Element, Scope},
    core_macro::rsx,
    hooks::use_state,
};

use crate::{config::PositionConfig, widgets::widget::WidgetFactory, APP_CONFIG};

pub fn Dashboard(cx: Scope<'_>) -> Element<'_> {
    let widgets_left = use_state(cx, || {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Left)
    });
    let widgets_center = use_state(cx, || {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Center)
    });
    let widgets_right = use_state(cx, || {
        WidgetFactory::make_widgets(&APP_CONFIG.get().unwrap().widget, PositionConfig::Right)
    });
    let grid_config = use_state(cx, || {
        let grid = &APP_CONFIG.get().unwrap().grid;
        (
            format!("col-span-{}", grid.left),
            format!("col-span-{}", grid.center),
            format!("col-span-{}", grid.right),
        )
    });
    let dom = rsx! {
            div {
                class: "p-2 grid grid-cols-12",
                div {
                    class: "{grid_config.0}",
                    for (i, widget) in widgets_left.iter().enumerate() {
                        div {
                            key: "{i}",
                            widget.render(cx)
                        }
                    }
                },
                div {
                    class: "{grid_config.1}",
                    for (i, widget) in widgets_center.iter().enumerate() {
                        div {
                            key: "{i}",
                            widget.render(cx)
                        }
                    }
                },
                div {
                    class: "{grid_config.2}",
                    for (i, widget) in widgets_right.iter().enumerate() {
                        div {
                            key: "{i}",
                            widget.render(cx)
                        }
                    }
                },
            }
    };

    cx.render(dom)
}
