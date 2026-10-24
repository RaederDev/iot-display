use super::data;
use crate::{
    config::Config,
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

    fn fetch(&mut self, request: Request, _http: Http, _testing: bool) -> Fetch {
        Box::pin(async move {
            match request {
                Request::Network { interface } => data::read(&interface)
                    .map(Data::Network)
                    .map_err(Into::into),
                _ => Err("Request sent to the wrong widget".to_owned().into()),
            }
        })
    }
}

pub fn requests(config: &Config) -> Vec<Request> {
    vec![Request::Network {
        interface: config.network_interface.clone(),
    }]
}

pub fn refresh_seconds(request: &Request) -> u64 {
    match request {
        Request::Network { .. } => 60,
        _ => unreachable!(),
    }
}

pub fn key(request: &Request) -> String {
    match request {
        Request::Network { interface } => format!("interface:{interface}"),
        _ => unreachable!(),
    }
}
