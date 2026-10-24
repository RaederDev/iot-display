use std::time::{Duration, Instant};

pub(super) const IDLE: Duration = Duration::from_secs(600);
pub(super) const COOLDOWN: Duration = Duration::from_secs(300);
pub(super) const TIMEOUT: Duration = Duration::from_secs(2);
pub(super) const RETRY: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Sleep,
    SensorWake,
    InputWake,
    Restore,
}

pub(super) struct Policy {
    pub enabled: bool,
    healthy: bool,
    present: bool,
    last_presence: Instant,
    wake_after: Instant,
}

impl Policy {
    pub fn new(enabled: bool, now: Instant) -> Self {
        Self {
            enabled,
            healthy: false,
            present: false,
            last_presence: now,
            wake_after: now,
        }
    }

    pub fn configure(&mut self, enabled: bool, now: Instant) {
        if self.enabled != enabled {
            self.enabled = enabled;
            self.last_presence = now;
            self.present = false;
        }
    }

    pub fn health(&mut self, healthy: bool, now: Instant) {
        if healthy && !self.healthy {
            self.last_presence = now;
        }
        self.healthy = healthy;
        if !healthy {
            self.present = false;
        }
    }

    pub fn presence(&mut self, present: bool, now: Instant) {
        self.present = present;
        if self.enabled && present {
            self.last_presence = now;
        }
    }

    pub fn decide(&self, now: Instant, inputs_idle: bool, resumed: bool) -> Option<Action> {
        if !self.enabled || !self.healthy {
            Some(Action::Restore)
        } else if resumed {
            Some(Action::InputWake)
        } else if self.present && now >= self.wake_after {
            Some(Action::SensorWake)
        } else if inputs_idle && now.duration_since(self.last_presence) >= IDLE {
            Some(Action::Sleep)
        } else {
            None
        }
    }

    pub fn woke(&mut self, now: Instant) {
        self.wake_after = now + COOLDOWN;
    }
}

#[derive(Default)]
pub(super) struct OutputState {
    pub mode: Option<bool>,
    pub pending: Option<(bool, Instant)>,
    pub blanked: bool,
}

impl OutputState {
    pub fn request(&mut self, on: bool, now: Instant) -> bool {
        if self.pending.is_some_and(|(target, _)| target == on)
            || (self.pending.is_none() && self.mode == Some(on) && !(on && self.blanked))
        {
            return false;
        }
        if !on {
            // Retain ownership even if the off acknowledgement is lost or arrives late.
            self.blanked = true;
        }
        self.pending = Some((on, now + TIMEOUT));
        true
    }

    pub fn confirmed(&mut self, on: bool) {
        self.mode = Some(on);
        if self.pending.is_some_and(|(target, _)| target == on) {
            self.pending = None;
        }
        if on && self.pending.is_none_or(|(target, _)| target) {
            self.blanked = false;
        }
    }

    pub fn timed_out(&mut self, now: Instant) -> bool {
        if self.pending.is_some_and(|(_, deadline)| now >= deadline) {
            self.pending = None;
            true
        } else {
            false
        }
    }
}
