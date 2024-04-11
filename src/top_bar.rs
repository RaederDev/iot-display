use std::net::IpAddr;

use dioxus::core_macro::rsx;
use dioxus::prelude::*;
use dioxus_desktop::window;
use local_ip_address::list_afinet_netifas;

use crate::APP_CONFIG;

fn get_local_ip(interface: &String) -> String {
    let interfaces = list_afinet_netifas().unwrap_or_default();

    // Prefer IPv4 if available, otherwise return any IP address
    interfaces
        .iter()
        .find(|(name, ip)| name == interface && matches!(ip, IpAddr::V4(_)))
        .or_else(|| interfaces.iter().find(|(name, _)| name == interface))
        .map_or_else(|| "No IP".to_string(), |(_, ip)| ip.to_string())
}

pub fn TopBar() -> Element {
    let ip = use_signal(|| get_local_ip(&APP_CONFIG.get().unwrap().ip.interface));

    rsx! {
        div {
            class: "z-10 fixed top-0 w-full flex justify-between px-3 py-1 bg-blue text-white",
            div {
                "{ip}"
            },
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
