use std::{collections::HashMap, time::Duration};

use anyhow::{Error, Ok};
use dioxus::prelude::*;
use serde::Deserialize;
use tokio::time::sleep;

use super::widget::Widget;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct GravityLastUpdated {
    file_exists: bool,
    absolute: u64,
    relative: GravityRelative,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct GravityRelative {
    days: u64,
    hours: u64,
    minutes: u64,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct SummaryRawResponse {
    domains_being_blocked: u64,
    dns_queries_today: u64,
    ads_blocked_today: u64,
    ads_percentage_today: f64,
    unique_domains: u64,
    queries_forwarded: u64,
    queries_cached: u64,
    clients_ever_seen: u64,
    unique_clients: u64,
    dns_queries_all_types: u64,
    reply_UNKNOWN: u64,
    reply_NODATA: u64,
    reply_NXDOMAIN: u64,
    reply_CNAME: u64,
    reply_IP: u64,
    reply_DOMAIN: u64,
    reply_RRNAME: u64,
    reply_SERVFAIL: u64,
    reply_REFUSED: u64,
    reply_NOTIMP: u64,
    reply_OTHER: u64,
    reply_DNSSEC: u64,
    reply_NONE: u64,
    reply_BLOB: u64,
    dns_queries_all_replies: u64,
    privacy_level: u64,
    status: String,
    gravity_last_updated: GravityLastUpdated,
}

const FETCH_INTERVAL: Duration = Duration::from_secs(60);

#[derive(PartialEq, Props, Clone)]
pub struct PiholeWidget {
    apiKey: String,
    apiUrl: String,
}

impl PiholeWidget {
    async fn fetch(url: &str, key: &str) -> Result<SummaryRawResponse, reqwest::Error> {
        let api_url = format!("{}?summaryRaw&auth={}", url, key);
        println!("Fetching from {}", url);
        let response = reqwest::get(&api_url).await?;
        response.json::<SummaryRawResponse>().await
    }
}

impl Widget for PiholeWidget {
    fn render(&self) -> Element {
        let mut result: Signal<Option<SummaryRawResponse>> = use_signal(|| None);
        use_coroutine(|_rx: UnboundedReceiver<()>| {
            let url = self.apiUrl.clone();
            let key = self.apiKey.clone();
            println!("Starting coroutine for {}", url);
            async move {
                loop {
                    let response = PiholeWidget::fetch(&url, &key).await.ok();
                    result.set(response);
                    sleep(FETCH_INTERVAL).await;
                }
            }
        });

        rsx!(
            div {
                class: "flex justify-center w-full",
                img {
                    height: "50",
                    width: "50",
                    alt: "Pi-hole",
                    src: "public/icons/pi-hole.svg",
                }
            }
            table {
                class: "w-full text-xl mt-7",
                tbody {
                    tr {
                        td { "Ads blocked today" }
                        td {
                            {result.read().as_ref().map_or_else(|| "Loading...".to_string(), |r| r.ads_blocked_today.to_string())}
                        }
                    }
                    tr {
                        td { "DNS queries today" }
                        td {
                            {result.read().as_ref().map_or_else(|| "Loading...".to_string(), |r| r.dns_queries_today.to_string())}
                        }
                    }
                    tr {
                        td { "Blocked domains" }
                        td {
                            {result.read().as_ref().map_or_else(|| "Loading...".to_string(), |r| r.domains_being_blocked.to_string())}
                        }
                    }
                }
            }
        )
    }

    fn build(config: &Option<HashMap<String, String>>) -> Result<Box<dyn Widget>, Error> {
        let map = config
            .as_ref()
            .ok_or_else(|| Error::msg("Missing configuration for PiholeWidget"))?;
        let apiKey = map
            .get("apiKey")
            .ok_or_else(|| Error::msg("Missing config key apiKey for PiholeWidget"))?;
        let apiUrl = map
            .get("apiUrl")
            .ok_or_else(|| Error::msg("Missing config key apiUrl for PiholeWidget"))?;

        Ok(Box::from(PiholeWidget {
            apiKey: apiKey.to_string(),
            apiUrl: apiUrl.to_string(),
        }))
    }
}
