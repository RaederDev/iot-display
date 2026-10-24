mod manager;
mod messages;
mod pacing;
mod worker;

pub use messages::{Command, Event};

use crate::{Result, cache::Cache, config::Config, http::Http};
use eframe::egui;
use std::{path::PathBuf, sync::mpsc, thread};
use tokio::sync::mpsc as async_mpsc;

/// Polls off the UI thread and wakes egui when results arrive.
pub struct Runtime {
    commands: async_mpsc::UnboundedSender<Command>,
    pub events: mpsc::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Runtime {
    pub fn start(
        config: Config,
        cache_path: PathBuf,
        debug_flag: bool,
        context: egui::Context,
    ) -> Result<Self> {
        let http = Http::new(config.debug || debug_flag)?;
        let (commands, receiver) = async_mpsc::unbounded_channel();
        let (sender, events) = mpsc::channel();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let thread = thread::spawn(move || {
            runtime.block_on(manager::run(
                config,
                Cache::load(cache_path),
                http,
                sender,
                context,
                debug_flag,
                receiver,
            ));
            // A stuck kernel I²C operation cannot be cancelled; never wait forever on exit.
            runtime.shutdown_timeout(std::time::Duration::from_secs(2));
        });
        Ok(Self {
            commands,
            events,
            thread: Some(thread),
        })
    }

    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
