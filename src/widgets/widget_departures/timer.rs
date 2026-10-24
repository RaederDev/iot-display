use super::data;
use crate::{
    config::{Config, Widget},
    http::Http,
    integrations::{Data, Request},
    widgets::backend::{Backend, Fetch},
};
use std::time::Duration;
use tokio::time::{Interval, MissedTickBehavior};

pub struct Timer {
    timer: Interval,
    catalogue_requested: bool,
}

pub fn backend(_config: &Config) -> Box<dyn Backend> {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    Box::new(Timer {
        timer,
        catalogue_requested: false,
    })
}

impl Backend for Timer {
    fn timer(&mut self) -> &mut Interval {
        &mut self.timer
    }

    fn requests(&self, config: &Config) -> Vec<Request> {
        let mut requests = requests(config);
        if self.catalogue_requested {
            requests.push(Request::Catalogue);
        }
        requests
    }

    fn refresh_requests(&self, config: &Config) -> Vec<Request> {
        requests(config)
    }

    fn requested(&mut self, request: &Request) {
        if matches!(request, Request::Catalogue) {
            self.catalogue_requested = true;
        }
    }

    fn fetch(&mut self, request: Request, http: Http, _testing: bool) -> Fetch {
        Box::pin(async move {
            match request {
                Request::Departures(config) => {
                    data::fetch(&http, &config).await.map(Data::Departures)
                }
                Request::Catalogue => data::catalogue(&http).await.map(Data::Catalogue),
                _ => Err("Request sent to the wrong widget".to_owned().into()),
            }
        })
    }
}

pub fn requests(config: &Config) -> Vec<Request> {
    if config.departures.enabled && config.uses(Widget::Departures) {
        vec![Request::Departures(config.departures.clone())]
    } else {
        vec![]
    }
}

pub fn refresh_seconds(request: &Request) -> u64 {
    match request {
        Request::Departures(_) => 300,
        Request::Catalogue => 7 * 86400,
        _ => unreachable!(),
    }
}

pub fn key(request: &Request) -> String {
    match request {
        Request::Departures(config) => data::monitor_urls(config)
            .iter()
            .map(reqwest::Url::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        Request::Catalogue => data::CATALOGUE_URL.into(),
        _ => unreachable!(),
    }
}
