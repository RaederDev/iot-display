use super::{Command, Event, pacing::Pacing};
use crate::{
    cache::{Cache, Entry},
    config::Config,
    http::{Http, HttpError},
    integrations::{Data, Provider, Request},
    widgets::backend::Backend,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
    time::Instant,
};
use tokio::{
    sync::{mpsc as async_mpsc, oneshot},
    task::JoinSet,
};

pub(super) enum Control {
    Command(Command),
    Pause(oneshot::Sender<()>),
    Resume,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Cached,
    Lookup,
    Refresh,
    ConnectionTest,
}

#[derive(Clone)]
struct Query {
    request: Request,
    provider: Provider,
    key: String,
    ttl: u64,
}

impl Query {
    fn new(request: Request) -> Self {
        Self {
            provider: request.provider(),
            key: request.key(),
            ttl: request.refresh_seconds(),
            request,
        }
    }
}

struct Completed {
    query: Query,
    result: std::result::Result<Data, HttpError>,
    mode: Mode,
}

struct Retry {
    attempts: u32,
}

pub(super) struct Worker {
    generation: u64,
    config: Config,
    cache: Arc<Mutex<Cache>>,
    http: Http,
    backend: Box<dyn Backend>,
    pacing: Pacing,
    queued: HashMap<Provider, (Query, Mode)>,
    sender: mpsc::Sender<Event>,
    context: eframe::egui::Context,
    tasks: JoinSet<Completed>,
    pending: HashMap<Provider, (String, Mode)>,
    retries: HashMap<Provider, Retry>,
    delivered: HashMap<Provider, (String, i64)>,
    memory_data: Option<(String, Entry)>,
}

impl Worker {
    pub(super) fn new(
        config: Config,
        cache: Arc<Mutex<Cache>>,
        http: Http,
        sender: mpsc::Sender<Event>,
        context: eframe::egui::Context,
        backend: Box<dyn Backend>,
    ) -> Self {
        Self {
            generation: 0,
            config,
            cache,
            http,
            backend,
            pacing: Pacing::default(),
            queued: HashMap::new(),
            sender,
            context,
            tasks: JoinSet::new(),
            pending: HashMap::new(),
            retries: HashMap::new(),
            delivered: HashMap::new(),
            memory_data: None,
        }
    }

    fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
        self.context.request_repaint();
    }

    fn schedule(&mut self, query: Query, mode: Mode) -> bool {
        let test = mode == Mode::ConnectionTest;
        let now = chrono::Utc::now().timestamp();

        if let Some((key, active_mode)) = self.pending.get(&query.provider) {
            return mode == Mode::Cached
                || (mode != Mode::ConnectionTest
                    && *active_mode != Mode::ConnectionTest
                    && key == &query.key);
        }

        // Publish stale data before refreshing so offline startup still shows the last good response.
        if !test {
            let cache = self.cache.clone();
            let cache = cache.lock().unwrap();
            let entry = if !query.provider.persistent_cache() {
                self.memory_data
                    .as_ref()
                    .filter(|(key, _)| key == &query.key)
                    .map(|(_, entry)| entry)
            } else {
                cache.get(&query.key)
            };
            if let Some(entry) = entry {
                if !self
                    .delivered
                    .get(&query.provider)
                    .is_some_and(|(key, time)| key == &query.key && *time == entry.fetched_at)
                {
                    self.emit(Event::DataUpdated {
                        generation: self.generation,
                        key: query.key.clone(),
                        entry: entry.clone(),
                    });
                    self.delivered
                        .insert(query.provider, (query.key.clone(), entry.fetched_at));
                }
                if matches!(mode, Mode::Cached | Mode::Lookup) && entry.fresh(now, query.ttl) {
                    let remaining = query
                        .ttl
                        .saturating_sub(now.saturating_sub(entry.fetched_at) as u64);
                    let delay = self
                        .backend
                        .refresh_delay(&query.request, &entry.data, chrono::Utc::now())
                        .min(remaining);
                    self.pacing
                        .seed(query.provider, query.key.clone(), Instant::now(), delay);
                    return true;
                }
            }
        }

        let monotonic = Instant::now();
        if mode == Mode::Cached
            && !self
                .pacing
                .automatic_due(query.provider, &query.key, monotonic)
        {
            return true;
        }
        if !self.pacing.allowed(query.provider, monotonic) {
            return false;
        }

        self.pacing.started(query.provider, monotonic);
        self.pending
            .insert(query.provider, (query.key.clone(), mode));
        let http = self.http.clone();
        let fetch = self.backend.fetch(query.request.clone(), http, test);
        self.tasks.spawn(async move {
            let result = fetch.await.and_then(|data| {
                if data.valid() && data.provider() == query.provider {
                    Ok(data)
                } else {
                    Err("Invalid widget response".to_owned().into())
                }
            });
            Completed {
                query,
                result,
                mode,
            }
        });
        true
    }

    fn enqueue(&mut self, query: Query, mode: Mode) {
        if !self.schedule(query.clone(), mode)
            && !self
                .queued
                .get(&query.provider)
                .is_some_and(|(_, queued_mode)| {
                    *queued_mode == Mode::ConnectionTest && mode != Mode::ConnectionTest
                })
        {
            self.queued.insert(query.provider, (query, mode));
        }
    }

    fn complete(&mut self, completed: Completed) {
        let Completed {
            query,
            result,
            mode,
        } = completed;
        let test = mode == Mode::ConnectionTest;
        self.pending.remove(&query.provider);
        self.backend.completed(test);
        let now = chrono::Utc::now().timestamp();
        match result {
            Ok(data) => {
                self.retries.remove(&query.provider);
                if test {
                    self.emit(Event::ConnectionTestFinished {
                        generation: self.generation,
                        provider: query.provider,
                        result: Ok(data),
                    });
                    return;
                }
                let delay = self
                    .backend
                    .refresh_delay(&query.request, &data, chrono::Utc::now());
                self.pacing
                    .succeeded(query.provider, query.key.clone(), Instant::now(), delay);
                let entry = Entry {
                    fetched_at: now,
                    data,
                };
                if !query.provider.persistent_cache() {
                    self.memory_data = Some((query.key.clone(), entry.clone()));
                } else if let Err(error) = self
                    .cache
                    .lock()
                    .unwrap()
                    .insert(query.key.clone(), entry.clone())
                {
                    self.emit(Event::Notice(format!("Cache could not be saved: {error}")));
                }
                self.delivered
                    .insert(query.provider, (query.key.clone(), now));
                self.emit(Event::DataUpdated {
                    generation: self.generation,
                    key: query.key,
                    entry,
                });
            }
            Err(error) => {
                let attempts = self
                    .retries
                    .get(&query.provider)
                    .map_or(1, |retry| retry.attempts.saturating_add(1));
                let delay = error.retry_after.unwrap_or(
                    15_u64
                        .saturating_mul(2_u64.saturating_pow(attempts.min(5)))
                        .min(300),
                );
                self.pacing.failed(query.provider, Instant::now(), delay);
                let retry_at = now.saturating_add(delay.max(5).min(i64::MAX as u64) as i64);
                self.retries.insert(query.provider, Retry { attempts });
                if test {
                    self.emit(Event::ConnectionTestFinished {
                        generation: self.generation,
                        provider: query.provider,
                        result: Err(error.message),
                    });
                } else {
                    self.emit(Event::RequestFailed {
                        generation: self.generation,
                        key: query.key,
                        provider: query.provider,
                        message: error.message,
                        retry_at,
                    });
                }
            }
        }
    }

    async fn command(&mut self, command: Command) -> bool {
        match command {
            Command::ApplyConfig(config) => {
                // Drain old requests before changing credentials or accepting the next generation.
                self.cancel_requests().await;
                self.generation += 1;
                if let Some(provider) = self.backend.reconfigure(&config, &self.http).await {
                    self.pacing.invalidate(provider);
                    if let Err(error) = self.cache.lock().unwrap().remove_provider(provider) {
                        self.emit(Event::Notice(error));
                    }
                }
                self.config = *config;
                self.delivered.clear();
            }
            Command::Refresh => {
                for query in self.backend.refresh_requests(&self.config) {
                    self.enqueue(Query::new(query), Mode::Refresh);
                }
            }
            Command::ClearCache => unreachable!("Cache clearing is coordinated by the runtime"),
            Command::Request(request) => {
                self.backend.requested(&request);
                let query = Query::new(request);
                self.delivered.remove(&query.provider);
                self.enqueue(query, Mode::Lookup);
            }
            Command::TestConnection(request) => {
                self.enqueue(Query::new(request), Mode::ConnectionTest)
            }
            Command::Shutdown => return false,
        }
        true
    }

    async fn cancel_requests(&mut self) {
        self.tasks.abort_all();
        while self.tasks.join_next().await.is_some() {}
        self.pending.clear();
        self.queued.clear();
        self.backend.cancel_tests(&self.http).await;
    }

    pub(super) async fn run(mut self, mut commands: async_mpsc::UnboundedReceiver<Control>) {
        let mut paused = false;
        loop {
            tokio::select! {
                command = commands.recv() => {
                    let Some(command) = command else { break };
                    match command {
                        Control::Command(command) => { if !self.command(command).await { break; } },
                        Control::Pause(acknowledge) => {
                            self.cancel_requests().await;
                            self.delivered.clear();
                            self.memory_data = None;
                            paused = true;
                            let _ = acknowledge.send(());
                        },
                        Control::Resume => {
                            paused = false;
                            for request in self.backend.requests(&self.config) { self.enqueue(Query::new(request), Mode::Lookup); }
                        },
                    }
                }
                Some(completed) = self.tasks.join_next(), if !self.tasks.is_empty() => {
                    match completed {
                        Ok(completed) => self.complete(completed),
                        Err(error) if error.is_cancelled() => {},
                        Err(_) => {
                            let providers: Vec<_> = self.pending.keys().copied().collect();
                            self.cancel_requests().await;
                            for provider in providers { self.pacing.failed(provider, Instant::now(), 30); }
                            self.emit(Event::Notice("A background request failed unexpectedly; retrying after cooldown".into()));
                        }
                    }
                }
                _ = self.backend.timer().tick(), if !paused => {
                    let queued = std::mem::take(&mut self.queued);
                    for (_, (query, mode)) in queued { self.enqueue(query, mode); }
                    for query in self.backend.requests(&self.config) { self.schedule(Query::new(query), Mode::Cached); }
                }
            }
        }
        self.cancel_requests().await;
        self.backend.shutdown(&self.http).await;
    }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod tests;
