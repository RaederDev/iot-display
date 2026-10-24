use super::data::PiHole;
use crate::{
    config::{Config, Widget},
    http::Http,
    integrations::{Data, Provider, Request},
    widgets::backend::{Backend, Cleanup, Fetch, Lifecycle},
};
use std::time::Duration;
use tokio::time::{Interval, MissedTickBehavior};

pub struct Timer {
    timer: Interval,
    client: PiHole,
    test_client: Option<PiHole>,
}

pub fn backend(config: &Config) -> Box<dyn Backend> {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    Box::new(Timer {
        timer,
        client: PiHole::new(config.pi_hole.clone()),
        test_client: None,
    })
}

impl Backend for Timer {
    fn timer(&mut self) -> &mut Interval {
        &mut self.timer
    }

    fn requests(&self, config: &Config) -> Vec<Request> {
        requests(config)
    }

    fn fetch(&mut self, request: Request, http: Http, testing: bool) -> Fetch {
        let client = if let Request::PiHole(config) = request
            && testing
        {
            let client = PiHole::new(config);
            self.test_client = Some(client.clone());
            client
        } else {
            self.client.clone()
        };
        Box::pin(async move {
            let result = client.fetch(&http).await.map(Data::PiHole);
            if testing {
                client.logout(&http).await;
            }
            result
        })
    }

    fn completed(&mut self, testing: bool) {
        if testing {
            self.test_client = None;
        }
    }

    fn reconfigure<'a>(&'a mut self, config: &'a Config, http: &'a Http) -> Lifecycle<'a> {
        Box::pin(async move {
            self.client
                .reconfigure(&config.pi_hole, http)
                .await
                .then_some(Provider::PiHole)
        })
    }

    fn cancel_tests<'a>(&'a mut self, http: &'a Http) -> Cleanup<'a> {
        Box::pin(async move {
            if let Some(client) = self.test_client.take() {
                client.logout(http).await;
            }
        })
    }

    fn shutdown<'a>(&'a mut self, http: &'a Http) -> Cleanup<'a> {
        Box::pin(async move {
            self.cancel_tests(http).await;
            self.client.logout(http).await;
        })
    }
}

pub fn requests(config: &Config) -> Vec<Request> {
    if config.pi_hole.enabled && config.uses(Widget::PiHole) {
        vec![Request::PiHole(config.pi_hole.clone())]
    } else {
        vec![]
    }
}

pub fn refresh_seconds(request: &Request) -> u64 {
    match request {
        Request::PiHole(_) => 60,
        _ => unreachable!(),
    }
}

pub fn key(request: &Request) -> String {
    match request {
        Request::PiHole(config) => format!("pi-hole:{}", config.api_url.trim_end_matches('/')),
        _ => unreachable!(),
    }
}
