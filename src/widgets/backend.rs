use crate::{
    config::Config,
    http::{Http, HttpError},
    integrations::{Data, Provider, Request},
};
use std::{future::Future, pin::Pin};
use tokio::time::Interval;

pub type Fetch = Pin<Box<dyn Future<Output = Result<Data, HttpError>> + Send>>;
pub type Cleanup<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub type Lifecycle<'a> = Pin<Box<dyn Future<Output = Option<Provider>> + Send + 'a>>;

/// One backend per registered widget type, shared by all copies on the dashboard.
pub trait Backend: Send {
    fn timer(&mut self) -> &mut Interval;
    fn requests(&self, config: &Config) -> Vec<Request>;
    fn fetch(&mut self, request: Request, http: Http, testing: bool) -> Fetch;

    fn refresh_delay(
        &self,
        request: &Request,
        _data: &Data,
        _now: chrono::DateTime<chrono::Utc>,
    ) -> u64 {
        request.refresh_seconds()
    }

    fn refresh_requests(&self, config: &Config) -> Vec<Request> {
        self.requests(config)
    }

    fn requested(&mut self, _request: &Request) {}

    fn completed(&mut self, _testing: bool) {}

    fn reconfigure<'a>(&'a mut self, _config: &'a Config, _http: &'a Http) -> Lifecycle<'a> {
        Box::pin(async { None })
    }

    fn cancel_tests<'a>(&'a mut self, _http: &'a Http) -> Cleanup<'a> {
        Box::pin(async {})
    }

    fn shutdown<'a>(&'a mut self, http: &'a Http) -> Cleanup<'a> {
        self.cancel_tests(http)
    }
}
