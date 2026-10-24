use super::{Completed, Mode, Query, Worker};
use crate::{
    cache::Cache,
    config::Config,
    http::Http,
    integrations::{Data, Provider, Request},
    widgets::{
        backend::{Backend, Fetch},
        widget_pihole::data::Stats,
    },
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
use tokio::time::Interval;

struct TestBackend {
    timer: Interval,
    calls: Arc<AtomicUsize>,
}

impl Backend for TestBackend {
    fn timer(&mut self) -> &mut Interval {
        &mut self.timer
    }

    fn requests(&self, config: &Config) -> Vec<Request> {
        vec![Request::PiHole(config.pi_hole.clone())]
    }

    fn fetch(&mut self, _request: Request, _http: Http, _testing: bool) -> Fetch {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Box::pin(async {
            Ok(Data::PiHole(Stats {
                queries: 10,
                blocked: 2,
                unique_domains: 4,
            }))
        })
    }
}

#[tokio::test]
async fn cache_loss_save_and_clear_do_not_reset_request_pacing() {
    let temp = tempfile::tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let config = Config::default();
    // A directory cannot be replaced by a cache file: successful networking must still be paced.
    let cache = Arc::new(Mutex::new(Cache::load(temp.path().to_path_buf())));
    let (sender, events) = mpsc::channel();
    let backend = TestBackend {
        timer: tokio::time::interval(Duration::from_secs(1)),
        calls: calls.clone(),
    };
    let mut worker = Worker::new(
        config.clone(),
        cache.clone(),
        Http::new(false).unwrap(),
        sender,
        eframe::egui::Context::default(),
        Box::new(backend),
    );
    let query = Query::new(Request::PiHole(config.pi_hole.clone()));
    assert!(worker.schedule(query.clone(), Mode::Cached));
    let completed = worker.tasks.join_next().await.unwrap().unwrap();
    worker.complete(completed);
    assert!(events.try_iter().any(|event| matches!(event, crate::runtime::Event::Notice(message) if message.contains("Cache could not be saved"))));
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    *cache.lock().unwrap() = Cache::load(temp.path().join("lost-cache.json"));
    for _ in 0..10 {
        assert!(worker.schedule(query.clone(), Mode::Cached));
    }
    worker
        .command(crate::runtime::Command::ApplyConfig(Box::new(config)))
        .await;
    assert!(worker.schedule(query.clone(), Mode::Cached));
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    worker.cancel_requests().await;
    cache.lock().unwrap().clear().unwrap();
    worker.enqueue(query.clone(), Mode::Lookup);
    worker.enqueue(query.clone(), Mode::Refresh);
    assert_eq!(worker.queued.len(), 1);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    worker
        .pacing
        .started(Provider::PiHole, Instant::now() - Duration::from_secs(5));
    assert!(worker.schedule(query, Mode::Refresh));
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn pending_requests_coalesce_and_tests_queue_without_overlap() {
    let temp = tempfile::tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let config = Config::default();
    let (sender, _) = mpsc::channel();
    let backend = TestBackend {
        timer: tokio::time::interval(Duration::from_secs(1)),
        calls: calls.clone(),
    };
    let mut worker = Worker::new(
        config.clone(),
        Arc::new(Mutex::new(Cache::load(temp.path().join("cache.json")))),
        Http::new(false).unwrap(),
        sender,
        eframe::egui::Context::default(),
        Box::new(backend),
    );
    let query = Query::new(Request::PiHole(config.pi_hole.clone()));
    worker.enqueue(query.clone(), Mode::Refresh);
    worker.enqueue(query.clone(), Mode::Refresh);
    assert!(worker.queued.is_empty());
    worker.enqueue(query.clone(), Mode::ConnectionTest);
    worker.enqueue(query.clone(), Mode::Refresh);
    assert_eq!(worker.queued[&Provider::PiHole].1, Mode::ConnectionTest);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    worker.complete(Completed {
        query,
        result: Err(crate::http::HttpError {
            message: "Rate limited".into(),
            status: Some(429),
            retry_after: Some(120),
        }),
        mode: Mode::Refresh,
    });
    assert!(
        !worker
            .pacing
            .allowed(Provider::PiHole, Instant::now() + Duration::from_secs(119))
    );
    worker.cancel_requests().await;
}

#[tokio::test]
async fn applying_an_older_config_does_not_overwrite_the_runtime_debug_setting() {
    let temp = tempfile::tempdir().unwrap();
    let config = Config::default();
    let http = Http::new(true).unwrap();
    let (sender, _) = mpsc::channel();
    let mut worker = Worker::new(
        config.clone(),
        Arc::new(Mutex::new(Cache::load(temp.path().join("cache.json")))),
        http.clone(),
        sender,
        eframe::egui::Context::default(),
        Box::new(TestBackend {
            timer: tokio::time::interval(Duration::from_secs(1)),
            calls: Arc::new(AtomicUsize::new(0)),
        }),
    );
    worker
        .command(crate::runtime::Command::ApplyConfig(Box::new(config)))
        .await;
    assert!(http.debug.load(Ordering::Relaxed));
}
