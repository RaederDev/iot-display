use super::{
    Status,
    policy::{Action, Policy, RETRY},
    sensor,
    wayland::Display,
};
use std::{
    future::pending,
    time::{Duration, Instant},
};

pub(super) struct Controller {
    policy: Policy,
    display: Option<Display>,
    owned_outputs: Vec<u32>,
    display_error: Option<String>,
    sensor_error: Option<String>,
    sensor: Option<sensor::Read>,
    next_sensor: Instant,
    next_display: Instant,
    sensor_waking: bool,
    initialize_power: bool,
}

impl Controller {
    pub fn new(enabled: bool) -> Self {
        let now = Instant::now();
        Self {
            policy: Policy::new(enabled, now),
            display: None,
            owned_outputs: vec![],
            display_error: Some("Checking Wayland display control.".into()),
            sensor_error: Some("Checking the EP-0106 sensor.".into()),
            sensor: None,
            next_sensor: now,
            next_display: now,
            sensor_waking: false,
            initialize_power: enabled,
        }
    }

    pub fn configure(&mut self, enabled: bool) {
        if enabled != self.policy.enabled {
            self.policy.configure(enabled, Instant::now());
            self.sensor_error = Some("Checking the EP-0106 sensor.".into());
            self.next_sensor = Instant::now();
            self.sensor_waking = false;
            self.initialize_power = enabled;
        }
    }

    pub async fn dispatch(&mut self) {
        let Some(display) = &mut self.display else {
            return pending().await;
        };
        if let Err(error) = display.dispatch().await {
            self.owned_outputs = display.owned_outputs();
            self.display = None;
            self.display_error = Some(format!(
                "Wayland display control disconnected: {error}. Check the graphical session."
            ));
            self.next_display = Instant::now() + RETRY;
            self.sensor_waking = false;
        }
    }

    pub async fn update(&mut self) {
        let now = Instant::now();
        self.poll_sensor(now).await;
        self.refresh_display(now).await;
        self.apply_policy(Instant::now());
    }

    async fn poll_sensor(&mut self, now: Instant) {
        if self.sensor.as_ref().is_some_and(sensor::Read::finished) {
            match self.sensor.take().unwrap().result().await {
                Ok(present) => {
                    self.sensor_error = None;
                    self.policy.presence(present, now);
                }
                Err(error) => self.sensor_error = Some(error),
            }
            self.next_sensor = now
                + if self.policy.enabled
                    && self.sensor_error.is_none()
                    && self.display_error.is_none()
                {
                    Duration::from_secs(1)
                } else {
                    RETRY
                };
        }
        if let Some(read) = &mut self.sensor
            && read.timeout(now)
        {
            self.sensor_error =
                Some("Sensor read timed out. Check the EP-0106 wiring and power.".into());
        }
        if self.sensor.is_none() && now >= self.next_sensor {
            self.sensor = Some(sensor::Read::start(now));
        }
    }

    async fn refresh_display(&mut self, now: Instant) {
        if let Some(display) = &mut self.display {
            self.display_error = display.problem(now);
        }
        if self.display_error.is_some() && now >= self.next_display {
            if let Some(display) = &mut self.display {
                display.restore().await;
                self.owned_outputs = display.owned_outputs();
            }
            match Display::connect().await {
                Ok(mut connected) => {
                    connected.restore_ownership(&self.owned_outputs);
                    self.display_error = connected.problem(Instant::now());
                    // Restore outstanding app-owned blanking before resuming normal policy.
                    connected.request(true, true, Instant::now());
                    self.display = Some(connected);
                    self.initialize_power = self.policy.enabled;
                }
                Err(error) => self.display_error = Some(error),
            }
            self.next_display = Instant::now() + RETRY;
        }
    }

    fn apply_policy(&mut self, now: Instant) {
        let healthy = self.sensor_error.is_none() && self.display_error.is_none();
        if self.policy.enabled && healthy {
            self.next_sensor = self.next_sensor.min(now + Duration::from_secs(1));
        }
        self.policy.health(healthy, now);
        if !healthy || !self.policy.enabled {
            self.sensor_waking = false;
        }
        if let Some(display) = &mut self.display {
            // Acquire power control visibly, including after a previous process died asleep.
            if self.initialize_power && self.display_error.is_none() {
                display.request(true, false, now);
                self.initialize_power = false;
            }
            if self.sensor_waking && display.all_on() {
                self.policy.woke(now);
                self.sensor_waking = false;
            }
            let resumed = display.take_resumed();
            if let Some(action) = self.policy.decide(now, display.inputs_idle(), resumed) {
                match action {
                    Action::Sleep => {
                        display.request(false, false, now);
                    }
                    Action::Restore => {
                        display.request(true, true, now);
                    }
                    Action::InputWake => {
                        display.request(true, false, now);
                    }
                    Action::SensorWake => {
                        if display.request(true, false, now) {
                            self.sensor_waking = true;
                        }
                    }
                }
            }
        }
    }

    pub fn status(&self) -> Status {
        let errors: Vec<_> = [self.sensor_error.as_deref(), self.display_error.as_deref()]
            .into_iter()
            .flatten()
            .collect();
        if errors.is_empty() {
            Status::Available
        } else {
            Status::Unavailable(errors.join(" "))
        }
    }

    pub async fn restore(&mut self) {
        if let Some(display) = &mut self.display {
            display.restore().await;
        }
    }
}
