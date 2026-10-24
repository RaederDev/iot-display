mod controller;
mod policy;
mod sensor;
mod wayland;

#[cfg(test)]
mod tests;

use crate::runtime::Event;
use controller::Controller;
use eframe::egui;
use std::{sync::mpsc, time::Duration};
use tokio::sync::watch;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Checking,
    Available,
    Unavailable(String),
}

pub(crate) async fn run(
    mut config: watch::Receiver<Option<bool>>,
    sender: mpsc::Sender<Event>,
    context: egui::Context,
) {
    let mut controller = Controller::new(config.borrow().unwrap_or(false));
    let mut status = Status::Checking;
    let mut ticks = tokio::time::interval(Duration::from_millis(100));
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            biased;
            changed = config.changed() => {
                if changed.is_err() { break; }
                let Some(enabled) = *config.borrow_and_update() else { break; };
                controller.configure(enabled);
            }
            _ = controller.dispatch() => {}
            _ = ticks.tick() => {}
        }
        controller.update().await;
        let updated = controller.status();
        if status != updated {
            status = updated;
            let _ = sender.send(Event::ScreenPowerStatus(status.clone()));
            context.request_repaint();
        }
    }
    controller.restore().await;
}
