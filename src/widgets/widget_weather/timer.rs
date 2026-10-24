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
}

pub fn backend(_config: &Config) -> Box<dyn Backend> {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    Box::new(Timer { timer })
}

impl Backend for Timer {
    fn timer(&mut self) -> &mut Interval {
        &mut self.timer
    }

    fn requests(&self, config: &Config) -> Vec<Request> {
        requests(config)
    }

    fn refresh_delay(
        &self,
        request: &Request,
        data: &Data,
        now: chrono::DateTime<chrono::Utc>,
    ) -> u64 {
        match data {
            Data::Weather(weather) => request
                .refresh_seconds()
                .min(weather.seconds_until_next_day(now)),
            _ => request.refresh_seconds(),
        }
    }

    fn fetch(&mut self, request: Request, http: Http, _testing: bool) -> Fetch {
        Box::pin(async move {
            match request {
                Request::Weather(config) => data::fetch(&http, &config).await.map(Data::Weather),
                Request::Geocoding(query) => data::search(&http, &query).await.map(Data::Geocoding),
                _ => Err("Request sent to the wrong widget".to_owned().into()),
            }
        })
    }
}

pub fn requests(config: &Config) -> Vec<Request> {
    if config.weather.enabled && config.uses(Widget::Weather) {
        vec![Request::Weather(config.weather.clone())]
    } else {
        vec![]
    }
}

pub fn refresh_seconds(request: &Request) -> u64 {
    match request {
        Request::Weather(_) => 3600,
        Request::Geocoding(_) => 30 * 86400,
        _ => unreachable!(),
    }
}

pub fn key(request: &Request) -> String {
    match request {
        Request::Weather(config) => data::forecast_url(config).to_string(),
        Request::Geocoding(text) => format!("geocoding:{}", data::normalize_search(text)),
        _ => unreachable!(),
    }
}
