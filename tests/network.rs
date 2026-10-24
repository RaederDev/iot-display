use iot_display::{
    cache::{Cache, Entry},
    config::{Config, Widget},
    http::Http,
    integrations::{
        Data, Provider, Request,
        pi_hole::{PiHole, Stats},
    },
    runtime::{Command as WorkerCommand, Event, Runtime},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

struct Reply {
    status: u16,
    headers: &'static str,
    body: &'static str,
    delay_ms: u64,
}

impl Reply {
    fn ok(body: &'static str) -> Self {
        Self {
            status: 200,
            headers: "",
            body,
            delay_ms: 0,
        }
    }
}

const STATS: &str = r#"{"queries":{"total":100,"blocked":20,"unique_domains":30}}"#;

const AUTH: &str = r#"{"session":{"valid":true,"sid":"secret-session"}}"#;

fn server(replies: Vec<Reply>) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(vec![]));
    let recorded = requests.clone();
    let handle = thread::spawn(move || {
        for reply in replies {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 2048];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..count]);
                if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_lowercase()
                                .strip_prefix("content-length:")
                                .map(|value| value.trim().parse().unwrap())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            recorded
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&request).to_string());
            thread::sleep(Duration::from_millis(reply.delay_ms));
            let _ = write!(
                stream,
                "HTTP/1.1 {} Response\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                reply.status,
                reply.body.len(),
                reply.headers,
                reply.body
            );
        }
    });
    (url, requests, handle)
}

fn probe(case: &str, url: &str, cache: &std::path::Path) -> String {
    let executable = std::env::current_exe().unwrap();
    let mut command = if let Ok(runner) = std::env::var("IOT_DISPLAY_TEST_RUNNER") {
        let mut command = Command::new(runner);
        command.arg(executable);
        command
    } else {
        Command::new(executable)
    };
    let output = command
        .args(["--exact", "child_probe", "--nocapture"])
        .env("IOT_DISPLAY_TEST_CASE", case)
        .env("IOT_DISPLAY_TEST_URL", url)
        .env("IOT_DISPLAY_TEST_CACHE", cache)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn log_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.starts_with('['))
        .collect()
}

#[test]
fn child_probe() {
    let Ok(case) = std::env::var("IOT_DISPLAY_TEST_CASE") else {
        return;
    };
    let url = std::env::var("IOT_DISPLAY_TEST_URL").unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let http = Http::new(true).unwrap();
    match case.as_str() {
        "redirect" => {
            runtime.block_on(async {
                let bytes = http
                    .get(
                        "test",
                        format!("{url}/start?apiKey=secret-key").parse().unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(bytes, b"ok");
            });
        }
        "auth" => {
            runtime.block_on(async {
                let pi_hole = PiHole::new(iot_display::config::PiHoleConfig {
                    api_url: format!("{url}/api"),
                    password: "secret-password".into(),
                    ..Default::default()
                });
                assert_eq!(pi_hole.fetch(&http).await.unwrap().blocked, 20);
                assert_eq!(pi_hole.fetch(&http).await.unwrap().queries, 100);
                pi_hole.logout(&http).await;
            });
        }
        "debug_config" | "debug_flag" | "debug_setting" | "debug_off" => {
            let mut config = Config {
                debug: case == "debug_setting",
                ..Default::default()
            };
            config.pi_hole.enabled = true;
            config.pi_hole.api_url = format!("{url}/api");
            config.pi_hole.password = "secret-password".into();
            let path = std::path::PathBuf::from(std::env::var("IOT_DISPLAY_TEST_CACHE").unwrap());
            let worker = Runtime::start(
                config.clone(),
                path,
                case == "debug_flag",
                eframe::egui::Context::default(),
            )
            .unwrap();
            wait_stats(&worker);
            config.debug = case == "debug_config";
            config.pi_hole.enabled = false;
            worker.send(WorkerCommand::ApplyConfig(Box::new(config)));
            drop(worker);
        }
        "failure" => {
            runtime.block_on(async {
                let http = Http::with_timeout(true, Duration::from_millis(200)).unwrap();
                let error = http.get("test", url.parse().unwrap()).await.unwrap_err();
                assert_eq!(error.status, Some(503));
                let error = http.get("test", url.parse().unwrap()).await.unwrap_err();
                assert_eq!(error.message, "Request timed out");
            });
        }
        "cache" => {
            let mut config = Config::default();
            config.pi_hole.enabled = true;
            config.pi_hole.api_url = format!("{url}/api");
            config.layout[0].columns[0].widgets = vec![Widget::PiHole; 3];
            let cache = std::path::PathBuf::from(std::env::var("IOT_DISPLAY_TEST_CACHE").unwrap());
            let worker = Runtime::start(
                config.clone(),
                cache.clone(),
                true,
                eframe::egui::Context::default(),
            )
            .unwrap();
            wait_stats(&worker);
            worker.send(WorkerCommand::Refresh);
            worker.send(WorkerCommand::Refresh);
            wait_stats(&worker);
            drop(worker);
            let worker =
                Runtime::start(config, cache, true, eframe::egui::Context::default()).unwrap();
            wait_stats(&worker);
            thread::sleep(Duration::from_millis(1100));
            drop(worker);
        }
        "draft" => {
            let mut config = Config::default();
            config.pi_hole.enabled = true;
            config.pi_hole.api_url = format!("{url}/api");
            let path = std::path::PathBuf::from(std::env::var("IOT_DISPLAY_TEST_CACHE").unwrap());
            let worker = Runtime::start(
                config.clone(),
                path.clone(),
                true,
                eframe::egui::Context::default(),
            )
            .unwrap();
            wait_stats(&worker);
            worker.send(WorkerCommand::TestConnection(Request::configured(
                Provider::PiHole,
                &config,
            )));
            loop {
                if let Event::ConnectionTestFinished { result, .. } =
                    worker.events.recv_timeout(Duration::from_secs(10)).unwrap()
                {
                    assert!(matches!(result.unwrap(), Data::PiHole(stats) if stats.queries == 200));
                    break;
                }
            }
            let cache = Cache::load(path);
            assert!(
                matches!(&cache.get(&iot_display::integrations::key(Provider::PiHole, &config)).unwrap().data, Data::PiHole(stats) if stats.queries == 100)
            );
            drop(worker);
        }
        _ => panic!("unknown test probe"),
    }
}

fn wait_stats(worker: &Runtime) {
    loop {
        if let Event::DataUpdated { entry, .. } =
            worker.events.recv_timeout(Duration::from_secs(10)).unwrap()
            && matches!(entry.data, Data::PiHole(_))
        {
            return;
        }
    }
}

#[test]
fn logs_redirects_and_redacts_url_secrets_on_stdout() {
    let (url, requests, handle) = server(vec![
        Reply {
            status: 302,
            headers: "Location: /final?sid=secret-session&stopId=399\r\n",
            body: "",
            delay_ms: 0,
        },
        Reply::ok("ok"),
    ]);
    let temp = tempfile::tempdir().unwrap();
    let output = probe("redirect", &url, temp.path());
    handle.join().unwrap();
    let logs = log_lines(&output);
    assert_eq!(logs.len(), 2);
    assert!(logs[0].contains("apiKey=REDACTED"));
    assert!(logs[1].contains("stopId=399"));
    assert!(!output.contains("secret-key"));
    assert!(!output.contains("secret-session"));
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn authenticates_reuses_and_renews_sessions_without_logging_secrets() {
    let unauthorized = || Reply {
        status: 401,
        headers: "",
        body: "{}",
        delay_ms: 0,
    };
    let (url, requests, handle) = server(vec![
        unauthorized(),
        Reply::ok(AUTH),
        Reply::ok(STATS),
        unauthorized(),
        Reply::ok(AUTH),
        Reply::ok(STATS),
        Reply::ok("{}"),
    ]);
    let temp = tempfile::tempdir().unwrap();
    let output = probe("auth", &url, temp.path());
    handle.join().unwrap();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 7);
    assert!(requests[1].contains("secret-password"));
    assert!(requests[3].contains("secret-session"));
    assert!(requests[6].starts_with("DELETE /api/auth"));
    assert_eq!(log_lines(&output).len(), 7);
    assert!(!output.contains("secret-password"));
    assert!(!output.contains("secret-session"));
}

#[test]
fn debug_settings_and_cli_override_apply_before_session_cleanup() {
    for (case, expected) in [
        ("debug_config", 1),
        ("debug_flag", 4),
        ("debug_setting", 3),
        ("debug_off", 0),
    ] {
        let (url, requests, handle) = server(vec![
            Reply {
                status: 401,
                headers: "",
                body: "{}",
                delay_ms: 0,
            },
            Reply::ok(AUTH),
            Reply::ok(STATS),
            Reply::ok("{}"),
        ]);
        let temp = tempfile::tempdir().unwrap();
        let output = probe(case, &url, &temp.path().join("cache.json"));
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 4, "{case}");
        assert!(requests[3].starts_with("DELETE /api/auth"));
        let logs = log_lines(&output);
        assert_eq!(logs.len(), expected, "{case}: {output}");
        if case == "debug_config" {
            assert!(logs[0].contains("pi-hole logout DELETE"));
        }
        assert!(!output.contains("secret-password"));
        assert!(!output.contains("secret-session"));
    }
}

#[test]
fn logs_http_failures_and_timed_out_attempts() {
    let mut delayed = Reply::ok("{}");
    delayed.delay_ms = 500;
    let (url, requests, handle) = server(vec![
        Reply {
            status: 503,
            headers: "",
            body: "{}",
            delay_ms: 0,
        },
        delayed,
    ]);
    let temp = tempfile::tempdir().unwrap();
    let output = probe("failure", &url, temp.path());
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 2);
    assert_eq!(log_lines(&output).len(), 2);
}

#[test]
fn duplicate_widgets_restart_and_force_refresh_do_not_repeat_requests() {
    let mut reply = Reply::ok(STATS);
    reply.delay_ms = 150;
    let (url, requests, handle) = server(vec![Reply::ok(STATS), reply]);
    let temp = tempfile::tempdir().unwrap();
    let output = probe("cache", &url, &temp.path().join("cache.json"));
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 2);
    assert_eq!(log_lines(&output).len(), 2);
}

#[test]
fn draft_connection_test_bypasses_statistics_cache_without_replacing_it() {
    let (url, requests, handle) = server(vec![
        Reply::ok(STATS),
        Reply::ok(r#"{"queries":{"total":200,"blocked":40,"unique_domains":50}}"#),
    ]);
    let temp = tempfile::tempdir().unwrap();
    let output = probe("draft", &url, &temp.path().join("cache.json"));
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 2);
    assert_eq!(log_lines(&output).len(), 2);
}

#[tokio::test]
async fn timeouts_http_errors_retry_after_and_malformed_json() {
    let mut reply = Reply::ok("{}");
    reply.delay_ms = 500;
    let (url, requests, handle) = server(vec![reply]);
    // Allow headers to reach the mock server under ARM emulation before timing out.
    let http = Http::with_timeout(false, Duration::from_millis(200)).unwrap();
    assert_eq!(
        http.get("test", url.parse().unwrap())
            .await
            .unwrap_err()
            .message,
        "Request timed out"
    );
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert!(requests.lock().unwrap()[0].starts_with("GET / HTTP/1.1"));
    let (url, _, handle) = server(vec![Reply {
        status: 429,
        headers: "Retry-After: 120\r\n",
        body: "{}",
        delay_ms: 0,
    }]);
    let error = Http::new(false)
        .unwrap()
        .get("test", url.parse().unwrap())
        .await
        .unwrap_err();
    assert_eq!(error.retry_after, Some(120));
    handle.join().unwrap();
    assert!(iot_display::integrations::weather::parse_forecast(b"not json").is_err());
}

#[test]
fn parses_planned_fallback_and_vehicle_destination_from_fixture() {
    let boards = iot_display::integrations::departures::parse_monitor(include_bytes!(
        "fixtures/wl-monitor.json"
    ))
    .unwrap();
    assert!(boards[0].departures[0].planned);
    assert!(!boards[0].departures[1].planned);
    assert_eq!(boards[0].departures[1].destination, "Short working");
    assert_eq!(boards[0].id, 399);
}

#[test]
fn configuration_change_cancels_pending_results() {
    let mut delayed = Reply::ok(STATS);
    delayed.delay_ms = 250;
    let (url, requests, handle) = server(vec![delayed, Reply::ok(STATS)]);
    let temp = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.pi_hole.enabled = true;
    config.pi_hole.api_url = format!("{url}/old/api");
    let worker = Runtime::start(
        config.clone(),
        temp.path().join("cache.json"),
        false,
        eframe::egui::Context::default(),
    )
    .unwrap();
    for _ in 0..100 {
        if !requests.lock().unwrap().is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(requests.lock().unwrap().len(), 1);
    config.pi_hole.api_url = format!("{url}/new/api");
    worker.send(WorkerCommand::ApplyConfig(Box::new(config.clone())));
    loop {
        match worker.events.recv_timeout(Duration::from_secs(10)).unwrap() {
            Event::DataUpdated {
                generation,
                key,
                entry,
            } if matches!(entry.data, Data::PiHole(_)) => {
                assert_eq!(generation, 1);
                assert_eq!(
                    key,
                    iot_display::integrations::key(Provider::PiHole, &config)
                );
                break;
            }
            Event::RequestFailed { message, .. } => panic!("{message}"),
            _ => {}
        }
    }
    drop(worker);
    handle.join().unwrap();
    let requests = requests.lock().unwrap();
    assert!(requests[0].starts_with("GET /old/api/stats/summary"));
    assert!(requests[1].starts_with("GET /new/api/stats/summary"));
}

#[test]
fn stale_data_survives_failure_and_force_refresh_respects_cooldown() {
    let (url, requests, handle) = server(vec![
        Reply {
            status: 429,
            headers: "Retry-After: 2\r\n",
            body: "{}",
            delay_ms: 0,
        },
        Reply::ok(STATS),
    ]);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("cache.json");
    let mut config = Config::default();
    config.pi_hole.enabled = true;
    config.pi_hole.api_url = format!("{url}/api");
    let key = iot_display::integrations::key(Provider::PiHole, &config);
    let original_time = chrono::Utc::now().timestamp() - 120;
    Cache::load(path.clone())
        .insert(
            key.clone(),
            Entry {
                fetched_at: original_time,
                data: Data::PiHole(Stats {
                    queries: 40,
                    blocked: 10,
                    unique_domains: 20,
                }),
            },
        )
        .unwrap();
    let worker = Runtime::start(
        config,
        path.clone(),
        false,
        eframe::egui::Context::default(),
    )
    .unwrap();
    let mut saw_stale = false;
    loop {
        match worker.events.recv_timeout(Duration::from_secs(10)).unwrap() {
            Event::DataUpdated { entry, .. } if matches!(entry.data, Data::PiHole(_)) => {
                assert_eq!(entry.fetched_at, original_time);
                saw_stale = true;
            }
            Event::RequestFailed {
                provider: Provider::PiHole,
                ..
            } => break,
            _ => {}
        }
    }
    assert!(saw_stale);
    assert_eq!(
        Cache::load(path.clone()).get(&key).unwrap().fetched_at,
        original_time
    );
    worker.send(WorkerCommand::Refresh);
    thread::sleep(Duration::from_millis(100));
    assert_eq!(requests.lock().unwrap().len(), 1);
    wait_stats(&worker);
    let cache = Cache::load(path);
    assert!(cache.get(&key).unwrap().fetched_at > original_time);
    assert!(matches!(&cache.get(&key).unwrap().data, Data::PiHole(stats) if stats.queries == 100));
    drop(worker);
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[test]
fn save_and_clear_cache_preserve_pacing_and_refresh_the_shared_backend_once() {
    let (url, requests, handle) = server(vec![Reply::ok(STATS), Reply::ok(STATS)]);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("cache.json");
    let mut config = Config::default();
    config.pi_hole.enabled = true;
    config.pi_hole.api_url = format!("{url}/api");
    config.layout[0].columns[0].widgets = vec![Widget::PiHole; 3];
    let worker = Runtime::start(
        config.clone(),
        path.clone(),
        false,
        eframe::egui::Context::default(),
    )
    .unwrap();
    wait_stats(&worker);
    let key = iot_display::integrations::key(Provider::PiHole, &config);
    let fetched_at = Cache::load(path.clone()).get(&key).unwrap().fetched_at;

    worker.send(WorkerCommand::ApplyConfig(Box::new(config)));
    loop {
        if let Event::DataUpdated {
            generation, entry, ..
        } = worker.events.recv_timeout(Duration::from_secs(10)).unwrap()
            && matches!(entry.data, Data::PiHole(_))
        {
            assert_eq!(generation, 1);
            assert_eq!(entry.fetched_at, fetched_at);
            break;
        }
    }
    assert_eq!(requests.lock().unwrap().len(), 1);
    worker.send(WorkerCommand::ClearCache);
    loop {
        if matches!(
            worker.events.recv_timeout(Duration::from_secs(10)).unwrap(),
            Event::CacheCleared
        ) {
            break;
        }
    }
    assert!(Cache::load(path.clone()).get(&key).is_none());
    assert_eq!(requests.lock().unwrap().len(), 1);
    worker.send(WorkerCommand::Refresh);
    worker.send(WorkerCommand::Refresh);
    wait_stats(&worker);
    assert!(Cache::load(path).get(&key).unwrap().fetched_at > fetched_at);
    drop(worker);
    handle.join().unwrap();
    assert_eq!(requests.lock().unwrap().len(), 2);
}
