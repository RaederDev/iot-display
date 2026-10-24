use super::{
    Command, Event,
    worker::{Control, Worker},
};
use crate::{
    cache::Cache,
    config::Config,
    http::Http,
    integrations::Provider,
    widgets::registry::{self, Registration},
};
use eframe::egui;
use std::sync::{Arc, Mutex, mpsc};
use tokio::{
    sync::{mpsc as async_mpsc, oneshot, watch},
    task::JoinHandle,
};

struct Running {
    registration: &'static Registration,
    commands: async_mpsc::UnboundedSender<Control>,
    task: JoinHandle<()>,
}

pub(super) async fn run(
    config: Config,
    cache: Cache,
    http: Http,
    sender: mpsc::Sender<Event>,
    context: egui::Context,
    debug_flag: bool,
    mut commands: async_mpsc::UnboundedReceiver<Command>,
) {
    let (power_config, power_receiver) = watch::channel(Some(config.screen_wake));
    let power = tokio::spawn(crate::screen_power::run(
        power_receiver,
        sender.clone(),
        context.clone(),
    ));
    let cache = Arc::new(Mutex::new(cache));
    let mut running = vec![];
    for registration in registry::REGISTERED {
        let Some(factory) = registration.backend else {
            continue;
        };
        let (commands, receiver) = async_mpsc::unbounded_channel();
        let worker = Worker::new(
            config.clone(),
            cache.clone(),
            http.clone(),
            sender.clone(),
            context.clone(),
            factory(&config),
        );
        running.push(Running {
            registration,
            commands,
            task: tokio::spawn(worker.run(receiver)),
        });
    }
    while let Some(command) = commands.recv().await {
        match command {
            Command::ApplyConfig(config) => {
                // Update logging before workers cancel requests or release old sessions.
                http.debug.store(
                    debug_flag || config.debug,
                    std::sync::atomic::Ordering::Relaxed,
                );
                let _ = power_config.send(Some(config.screen_wake));
                for widget in &running {
                    let _ = widget
                        .commands
                        .send(Control::Command(Command::ApplyConfig(config.clone())));
                }
            }
            Command::Refresh => {
                for widget in &running {
                    let _ = widget.commands.send(Control::Command(Command::Refresh));
                }
            }
            Command::ClearCache => {
                // Stop every writer before clearing; resume keeps independent pacing deadlines.
                for widget in &running {
                    let (acknowledge, stopped) = oneshot::channel();
                    let _ = widget.commands.send(Control::Pause(acknowledge));
                    let _ = stopped.await;
                }
                let event = match cache.lock().unwrap().clear() {
                    Ok(()) => Event::CacheCleared,
                    Err(error) => Event::Notice(error),
                };
                let _ = sender.send(event);
                context.request_repaint();
                for widget in &running {
                    let _ = widget.commands.send(Control::Resume);
                }
            }
            Command::Request(request) => {
                route(&running, request.provider(), Command::Request(request))
            }
            Command::TestConnection(request) => route(
                &running,
                request.provider(),
                Command::TestConnection(request),
            ),
            Command::Shutdown => break,
        }
    }
    let _ = power_config.send(None);
    for widget in &running {
        let _ = widget.commands.send(Control::Command(Command::Shutdown));
    }
    for widget in running {
        let _ = widget.task.await;
    }
    let _ = power.await;
}

fn route(running: &[Running], provider: Provider, command: Command) {
    if let Some(widget) = running
        .iter()
        .find(|widget| widget.registration.providers.contains(&provider))
    {
        let _ = widget.commands.send(Control::Command(command));
    }
}
