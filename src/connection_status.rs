use dioxus::core_macro::rsx;
use dioxus::prelude::*;
use dioxus_desktop::window;

use crate::{config::PositionConfig, widgets::widget::WidgetFactory, APP_CONFIG};

pub fn TopBar() -> Element {
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
            class: "mb-4"
        }
        div {
            class: "z-10 fixed top-0 w-full flex justify-between px-3 py-1 bg-blue text-white",
            div {
                ul {
                    class: "flex",
                    li {
                        class: "h-full flex items-center",
                        button {
                            onclick: |_| { window().close() },
                            class: "px-2",
                            img {
                                height: "15",
                                width: "15",
                                alt: "close",
                                src: "public/icons/close.svg",
                            }
                        }
                    }
                }
            }
        }
    }
}
