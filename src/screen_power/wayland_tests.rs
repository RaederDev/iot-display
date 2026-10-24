use super::*;
use std::{
    os::unix::net::UnixStream,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};
use wayland_protocols::ext::idle_notify::v1::server::{
    ext_idle_notification_v1 as notification, ext_idle_notifier_v1 as idle,
};
use wayland_protocols_wlr::output_power_management::v1::server::{
    zwlr_output_power_manager_v1 as manager, zwlr_output_power_v1 as power,
};
use wayland_server::protocol::{wl_output as output, wl_seat as seat};
use wayland_server::{
    Client, DataInit, Dispatch as ServerDispatch, Display as ServerDisplay, DisplayHandle,
    GlobalDispatch, New, Resource, WEnum as ServerEnum,
};

#[derive(Default)]
struct Compositor {
    powers: Vec<power::ZwlrOutputPowerV1>,
    notifications: Vec<notification::ExtIdleNotificationV1>,
    suppress_ack: bool,
}

enum Control {
    Idle,
    Resume,
    SuppressAck,
    Fail,
    ExternalOff,
    AddOutput,
    RemoveOutput,
    Stop,
}

struct Server {
    commands: mpsc::Sender<(Control, tokio::sync::oneshot::Sender<()>)>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    fn start(with_idle: bool, with_power: bool) -> (Self, Connection) {
        let (client, socket) = UnixStream::pair().unwrap();
        let (commands, receiver) = mpsc::channel::<(Control, tokio::sync::oneshot::Sender<()>)>();
        let thread = thread::spawn(move || {
            let mut display = ServerDisplay::<Compositor>::new().unwrap();
            let mut handle = display.handle();
            handle.insert_client(socket, Arc::new(())).unwrap();
            handle.create_global::<Compositor, output::WlOutput, _>(4, ());
            let mut extra_output = None;
            handle.create_global::<Compositor, seat::WlSeat, _>(7, ());
            if with_power {
                handle.create_global::<Compositor, manager::ZwlrOutputPowerManagerV1, _>(1, ());
            }
            if with_idle {
                handle.create_global::<Compositor, idle::ExtIdleNotifierV1, _>(1, ());
            }
            let mut state = Compositor::default();
            loop {
                if let Ok((command, done)) = receiver.recv_timeout(Duration::from_millis(1)) {
                    match command {
                        Control::Stop => break,
                        Control::Idle => {
                            for notification in &state.notifications {
                                notification.idled();
                            }
                        }
                        Control::Resume => {
                            for notification in &state.notifications {
                                notification.resumed();
                            }
                        }
                        Control::SuppressAck => state.suppress_ack = true,
                        Control::ExternalOff => {
                            for power in &state.powers {
                                power.mode(power::Mode::Off);
                            }
                        }
                        Control::Fail => {
                            for power in &state.powers {
                                if power.is_alive() {
                                    power.failed();
                                }
                            }
                        }
                        Control::AddOutput => {
                            extra_output =
                                Some(handle.create_global::<Compositor, output::WlOutput, _>(4, ()))
                        }
                        Control::RemoveOutput => {
                            if let Some(id) = extra_output.take() {
                                handle.remove_global::<Compositor>(id);
                            }
                        }
                    }
                    display.flush_clients().unwrap();
                    let _ = done.send(());
                }
                display.dispatch_clients(&mut state).unwrap();
                display.flush_clients().unwrap();
            }
        });
        (
            Self {
                commands,
                thread: Some(thread),
            },
            Connection::from_socket(client).unwrap(),
        )
    }

    async fn send(&self, control: Control) {
        let (done, wait) = tokio::sync::oneshot::channel();
        self.commands.send((control, done)).unwrap();
        tokio::time::timeout(Duration::from_secs(2), wait)
            .await
            .unwrap()
            .unwrap();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let (done, _) = tokio::sync::oneshot::channel();
        let _ = self.commands.send((Control::Stop, done));
        self.thread.take().unwrap().join().unwrap();
    }
}

macro_rules! global {
    ($interface:ty) => {
        impl GlobalDispatch<$interface, ()> for Compositor {
            fn bind(
                _: &mut Self,
                _: &DisplayHandle,
                _: &Client,
                resource: New<$interface>,
                _: &(),
                init: &mut DataInit<'_, Self>,
            ) {
                init.init(resource, ());
            }
        }
    };
}

global!(output::WlOutput);
global!(seat::WlSeat);
global!(manager::ZwlrOutputPowerManagerV1);
global!(idle::ExtIdleNotifierV1);
macro_rules! ignore_requests {
    ($interface:ty) => {
        impl ServerDispatch<$interface, ()> for Compositor {
            fn request(
                _: &mut Self,
                _: &Client,
                _: &$interface,
                _: <$interface as Resource>::Request,
                _: &(),
                _: &DisplayHandle,
                _: &mut DataInit<'_, Self>,
            ) {
            }
        }
    };
}

ignore_requests!(output::WlOutput);
ignore_requests!(seat::WlSeat);
ignore_requests!(notification::ExtIdleNotificationV1);

impl ServerDispatch<manager::ZwlrOutputPowerManagerV1, ()> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &manager::ZwlrOutputPowerManagerV1,
        request: manager::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let manager::Request::GetOutputPower { id, .. } = request {
            let power = init.init(id, ());
            power.mode(power::Mode::On);
            state.powers.push(power);
        }
    }
}

impl ServerDispatch<power::ZwlrOutputPowerV1, ()> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        power: &power::ZwlrOutputPowerV1,
        request: power::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        if let power::Request::SetMode {
            mode: ServerEnum::Value(mode),
        } = request
            && !state.suppress_ack
        {
            power.mode(mode);
        }
    }
}

impl ServerDispatch<idle::ExtIdleNotifierV1, ()> for Compositor {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &idle::ExtIdleNotifierV1,
        request: idle::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let idle::Request::GetIdleNotification { id, timeout, .. } = request {
            assert_eq!(timeout, 600_000);
            state.notifications.push(init.init(id, ()));
        }
    }
}

#[tokio::test]
async fn real_protocol_idle_power_acknowledgements_and_hotplug() {
    let (server, connection) = Server::start(true, true);
    let mut display = Display::from_connection(connection).await.unwrap();
    assert!(display.problem(Instant::now()).is_none());
    assert!(display.all_on());
    assert!(!display.inputs_idle());
    server.send(Control::ExternalOff).await;
    display.sync().await.unwrap();
    display.restore().await;
    assert!(
        !display.all_on(),
        "restore must leave outputs the app did not blank alone"
    );
    display.request(true, false, Instant::now());
    display.sync().await.unwrap();
    assert!(display.all_on());
    server.send(Control::Idle).await;
    display.sync().await.unwrap();
    assert!(display.inputs_idle());
    assert!(display.request(false, false, Instant::now()));
    assert!(!display.restored());
    display.sync().await.unwrap();
    assert!(!display.all_on());
    server.send(Control::Resume).await;
    display.sync().await.unwrap();
    assert!(display.take_resumed());
    assert!(!display.take_resumed());
    assert!(!display.inputs_idle());
    display.restore().await;
    assert!(display.all_on());
    assert!(display.restored());
    server.send(Control::AddOutput).await;
    display.sync().await.unwrap();
    display.sync().await.unwrap();
    assert_eq!(display.state.outputs.len(), 2);
    assert!(display.problem(Instant::now()).is_none());
    server.send(Control::RemoveOutput).await;
    display.sync().await.unwrap();
    assert_eq!(display.state.outputs.len(), 1);
    server.send(Control::Fail).await;
    display.sync().await.unwrap();
    assert!(
        display
            .problem(Instant::now())
            .unwrap()
            .contains("rejected")
    );
}

#[tokio::test]
async fn missing_idle_protocol_prevents_availability() {
    let (_server, connection) = Server::start(false, true);
    let mut display = Display::from_connection(connection).await.unwrap();
    assert!(
        display
            .problem(Instant::now())
            .unwrap()
            .contains("ext-idle-notify")
    );
}

#[tokio::test]
async fn missing_power_acknowledgement_is_a_failure_and_restore_is_bounded() {
    let (server, connection) = Server::start(true, true);
    let mut display = Display::from_connection(connection).await.unwrap();
    server.send(Control::SuppressAck).await;
    let now = Instant::now();
    assert!(display.request(false, false, now));
    display.sync().await.unwrap();
    assert!(
        display
            .problem(now + TIMEOUT)
            .unwrap()
            .contains("did not confirm")
    );
    let started = Instant::now();
    display.restore().await;
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(
        !display.restored(),
        "unconfirmed restoration retains ownership"
    );
}

#[tokio::test]
async fn missing_power_protocol_prevents_availability() {
    let (_server, connection) = Server::start(true, false);
    let mut display = Display::from_connection(connection).await.unwrap();
    assert!(
        display
            .problem(Instant::now())
            .unwrap()
            .contains("output power management")
    );
}
