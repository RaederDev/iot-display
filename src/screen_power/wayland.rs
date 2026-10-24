use super::policy::{IDLE, OutputState, TIMEOUT};
use crate::Result;
use std::{
    collections::BTreeMap,
    io,
    os::fd::{AsFd, OwnedFd},
    time::Instant,
};
use tokio::io::unix::AsyncFd;
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{wl_callback, wl_output, wl_registry, wl_seat},
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};
use wayland_protocols_wlr::output_power_management::v1::client::{
    zwlr_output_power_manager_v1, zwlr_output_power_v1,
};

struct Output {
    proxy: wl_output::WlOutput,
    power: Option<zwlr_output_power_v1::ZwlrOutputPowerV1>,
    state: OutputState,
    failed: bool,
}

struct Seat {
    proxy: wl_seat::WlSeat,
    notification: Option<ext_idle_notification_v1::ExtIdleNotificationV1>,
    idle: bool,
}

#[derive(Default)]
struct State {
    outputs: BTreeMap<u32, Output>,
    seats: BTreeMap<u32, Seat>,
    power: Option<(u32, zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1)>,
    idle: Option<(u32, ext_idle_notifier_v1::ExtIdleNotifierV1)>,
    synced: bool,
    resumed: bool,
    error: Option<String>,
}

impl State {
    fn attach(&mut self, qh: &QueueHandle<Self>) {
        for (id, output) in &mut self.outputs {
            if output.power.is_none()
                && !output.failed
                && let Some((_, manager)) = &self.power
            {
                output.power = Some(manager.get_output_power(&output.proxy, qh, *id));
            }
        }
        for (id, seat) in &mut self.seats {
            if seat.notification.is_none()
                && let Some((_, manager)) = &self.idle
            {
                seat.notification = Some(manager.get_idle_notification(
                    IDLE.as_millis() as u32,
                    &seat.proxy,
                    qh,
                    *id,
                ));
            }
        }
    }

    fn problem(&self) -> Option<String> {
        if let Some(error) = &self.error {
            Some(error.clone())
        } else if self.idle.is_none() {
            Some(
                "The compositor does not offer ext-idle-notify. Run in a labwc Wayland session."
                    .into(),
            )
        } else if self.power.is_none() {
            Some("The compositor does not offer output power management. Run in a labwc Wayland session.".into())
        } else if self.seats.is_empty() {
            Some("No Wayland input seat is available. Check the graphical session.".into())
        } else if self.outputs.is_empty() {
            Some("No Wayland display output is available. Check the display connection.".into())
        } else if self.outputs.values().any(|output| output.failed) {
            Some("An output rejected power control. Check compositor support and stop competing screen-power tools.".into())
        } else if self
            .outputs
            .values()
            .any(|output| output.state.mode.is_none())
        {
            Some("Waiting for the compositor to report display power state.".into())
        } else {
            None
        }
    }
}

pub(super) struct Display {
    connection: Connection,
    queue: EventQueue<State>,
    socket: AsyncFd<OwnedFd>,
    state: State,
}

impl Display {
    pub async fn connect() -> Result<Self> {
        let connection = Connection::connect_to_env().map_err(|error| {
            format!(
                "Cannot connect to Wayland: {error}. Start the app inside the graphical session."
            )
        })?;
        Self::from_connection(connection).await
    }

    async fn from_connection(connection: Connection) -> Result<Self> {
        let queue = connection.new_event_queue();
        connection.display().get_registry(&queue.handle(), ());
        let socket = AsyncFd::new(
            connection
                .as_fd()
                .try_clone_to_owned()
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let mut display = Self {
            connection,
            queue,
            socket,
            state: State::default(),
        };
        // The first sync discovers globals; the second receives newly bound objects' initial states.
        display.sync().await?;
        display.sync().await?;
        Ok(display)
    }

    async fn sync(&mut self) -> Result<()> {
        self.state.synced = false;
        self.connection.display().sync(&self.queue.handle(), ());
        tokio::time::timeout(TIMEOUT, async {
            while !self.state.synced {
                self.dispatch().await?;
            }
            Ok(())
        })
        .await
        .map_err(|_| {
            "Wayland did not respond within two seconds. Check the compositor.".to_string()
        })?
    }

    pub async fn dispatch(&mut self) -> Result<()> {
        self.queue.flush().map_err(|error| error.to_string())?;
        if self
            .queue
            .dispatch_pending(&mut self.state)
            .map_err(|error| error.to_string())?
            > 0
        {
            return Ok(());
        }
        let Some(read) = self.queue.prepare_read() else {
            return Ok(());
        };
        let mut ready = self
            .socket
            .readable()
            .await
            .map_err(|error| error.to_string())?;
        if let Ok(result) = ready.try_io(|_| {
            match read.read() {
                // A zero-event libwayland read must clear readiness to avoid spinning.
                Ok(0) => Err(io::ErrorKind::WouldBlock.into()),
                Ok(count) => Ok(count),
                Err(wayland_client::backend::WaylandError::Io(error)) => Err(error),
                Err(error) => Err(io::Error::other(error)),
            }
        }) {
            result.map_err(|error| error.to_string())?;
        }
        self.queue
            .dispatch_pending(&mut self.state)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn problem(&mut self, now: Instant) -> Option<String> {
        for output in self.state.outputs.values_mut() {
            if output.state.timed_out(now) {
                self.state.error = Some("The compositor did not confirm a display power change within two seconds. Check the display and competing power tools.".into());
            }
        }
        self.state.problem()
    }

    pub fn inputs_idle(&self) -> bool {
        !self.state.seats.is_empty() && self.state.seats.values().all(|seat| seat.idle)
    }

    pub fn take_resumed(&mut self) -> bool {
        std::mem::take(&mut self.state.resumed)
    }

    pub fn request(&mut self, on: bool, restore_only: bool, now: Instant) -> bool {
        let mut changed = false;
        for output in self.state.outputs.values_mut() {
            if restore_only && !output.state.blanked {
                continue;
            }
            if let Some(power) = &output.power
                && !output.failed
                && output.state.request(on, now)
            {
                power.set_mode(if on {
                    zwlr_output_power_v1::Mode::On
                } else {
                    zwlr_output_power_v1::Mode::Off
                });
                changed = true;
            }
        }
        changed
    }

    pub fn all_on(&self) -> bool {
        !self.state.outputs.is_empty()
            && self.state.outputs.values().all(|output| {
                !output.failed && output.state.mode == Some(true) && output.state.pending.is_none()
            })
    }

    pub fn owned_outputs(&self) -> Vec<u32> {
        self.state
            .outputs
            .iter()
            .filter_map(|(id, output)| output.state.blanked.then_some(*id))
            .collect()
    }

    pub fn restore_ownership(&mut self, ids: &[u32]) {
        for id in ids {
            if let Some(output) = self.state.outputs.get_mut(id) {
                output.state.blanked = true;
            }
        }
    }

    pub fn restored(&self) -> bool {
        self.owned_outputs().is_empty()
    }

    pub async fn restore(&mut self) {
        self.request(true, true, Instant::now());
        let _ = tokio::time::timeout(TIMEOUT, async {
            while !self.restored() {
                self.dispatch().await?;
            }
            Ok::<_, String>(())
        })
        .await;
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                match interface.as_str() {
                    "zwlr_output_power_manager_v1" => {
                        state.power = Some((name, registry.bind(name, 1, qh, ())))
                    }
                    "ext_idle_notifier_v1" => {
                        state.idle = Some((name, registry.bind(name, 1, qh, ())))
                    }
                    "wl_output" => {
                        state.outputs.insert(
                            name,
                            Output {
                                proxy: registry.bind(name, version.min(4), qh, ()),
                                power: None,
                                state: OutputState::default(),
                                failed: false,
                            },
                        );
                    }
                    "wl_seat" => {
                        state.seats.insert(
                            name,
                            Seat {
                                proxy: registry.bind(name, version.min(7), qh, ()),
                                notification: None,
                                idle: false,
                            },
                        );
                    }
                    _ => {}
                }
                state.attach(qh);
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some(output) = state.outputs.remove(&name) {
                    if let Some(power) = output.power {
                        power.destroy();
                    }
                    if output.proxy.version() >= 3 {
                        output.proxy.release();
                    }
                }
                if let Some(seat) = state.seats.remove(&name) {
                    if let Some(notification) = seat.notification {
                        notification.destroy();
                    }
                    if seat.proxy.version() >= 5 {
                        seat.proxy.release();
                    }
                }
                if state.power.as_ref().is_some_and(|(id, _)| *id == name) {
                    state.power = None;
                }
                if state.idle.as_ref().is_some_and(|(id, _)| *id == name) {
                    state.idle = None;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, u32> for State {
    fn event(
        state: &mut Self,
        _: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(seat) = state.seats.get_mut(id) {
            match event {
                ext_idle_notification_v1::Event::Idled => seat.idle = true,
                ext_idle_notification_v1::Event::Resumed => {
                    seat.idle = false;
                    state.resumed = true;
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<zwlr_output_power_v1::ZwlrOutputPowerV1, u32> for State {
    fn event(
        state: &mut Self,
        power: &zwlr_output_power_v1::ZwlrOutputPowerV1,
        event: zwlr_output_power_v1::Event,
        id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(output) = state.outputs.get_mut(id) {
            match event {
                zwlr_output_power_v1::Event::Mode {
                    mode: WEnum::Value(mode),
                } => output
                    .state
                    .confirmed(mode == zwlr_output_power_v1::Mode::On),
                zwlr_output_power_v1::Event::Failed => {
                    output.failed = true;
                    power.destroy();
                    output.power = None;
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        _: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        state.synced = true;
    }
}

delegate_noop!(State: ignore wl_output::WlOutput);
delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_noop!(State: ignore ext_idle_notifier_v1::ExtIdleNotifierV1);
delegate_noop!(State: ignore zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1);

#[cfg(test)]
#[path = "wayland_tests.rs"]
mod tests;
