use crate::integrations::Provider;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const MINIMUM_ATTEMPT_GAP: Duration = Duration::from_secs(5);

#[derive(Default)]
struct Deadline {
    earliest_attempt: Option<Instant>,
    blocked: bool,
    automatic: Option<(String, Instant)>,
}

/// Deadlines survive cache loss, cancellation, Clear cache, and settings changes.
#[derive(Default)]
pub(super) struct Pacing {
    deadlines: HashMap<Provider, Deadline>,
}

impl Pacing {
    pub fn allowed(&self, provider: Provider, now: Instant) -> bool {
        self.deadlines.get(&provider).is_none_or(|deadline| {
            !deadline.blocked && deadline.earliest_attempt.is_none_or(|at| now >= at)
        })
    }

    pub fn automatic_due(&self, provider: Provider, key: &str, now: Instant) -> bool {
        self.deadlines
            .get(&provider)
            .and_then(|deadline| deadline.automatic.as_ref())
            .is_none_or(|(identity, at)| identity != key || now >= *at)
    }

    pub fn started(&mut self, provider: Provider, now: Instant) {
        self.deadlines.entry(provider).or_default().earliest_attempt =
            Some(now + MINIMUM_ATTEMPT_GAP);
    }

    pub fn succeeded(&mut self, provider: Provider, key: String, now: Instant, seconds: u64) {
        self.deadlines.entry(provider).or_default().automatic =
            Some((key, now + Duration::from_secs(seconds)));
    }

    pub fn seed(&mut self, provider: Provider, key: String, now: Instant, seconds: u64) {
        if self.automatic_due(provider, &key, now) {
            self.succeeded(provider, key, now, seconds);
        }
    }

    pub fn failed(&mut self, provider: Provider, now: Instant, seconds: u64) {
        let deadline = self.deadlines.entry(provider).or_default();
        let Some(at) = now.checked_add(Duration::from_secs(seconds)) else {
            // An unrepresentable server cooldown blocks requests for the rest of this launch.
            deadline.blocked = true;
            return;
        };
        deadline.earliest_attempt = Some(
            deadline
                .earliest_attempt
                .map_or(at, |existing| existing.max(at)),
        );
    }

    pub fn invalidate(&mut self, provider: Provider) {
        if let Some(deadline) = self.deadlines.get_mut(&provider) {
            deadline.automatic = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_schedule_does_not_depend_on_cache_and_new_parameters_keep_minimum_gap() {
        let now = Instant::now();
        let mut pacing = Pacing::default();
        pacing.started(Provider::Weather, now);
        pacing.succeeded(Provider::Weather, "vienna".into(), now, 3600);
        assert!(!pacing.automatic_due(Provider::Weather, "vienna", now + Duration::from_secs(1)));
        assert!(pacing.automatic_due(Provider::Weather, "graz", now + Duration::from_secs(1)));
        assert!(!pacing.allowed(Provider::Weather, now + Duration::from_secs(1)));
        assert!(pacing.allowed(Provider::Weather, now + Duration::from_secs(5)));
        assert!(pacing.automatic_due(Provider::Weather, "vienna", now + Duration::from_secs(3600)));
    }

    #[test]
    fn oversized_server_cooldown_never_panics_or_allows_an_early_retry() {
        let now = Instant::now();
        let mut pacing = Pacing::default();
        pacing.failed(Provider::PiHole, now, u64::MAX);
        pacing.invalidate(Provider::PiHole);
        assert!(!pacing.allowed(Provider::PiHole, now + Duration::from_secs(86400)));
    }

    #[test]
    fn cached_startup_deadline_keeps_original_age() {
        let now = Instant::now();
        let mut pacing = Pacing::default();
        pacing.seed(Provider::PiHole, "host".into(), now, 40);
        pacing.seed(
            Provider::PiHole,
            "host".into(),
            now + Duration::from_secs(20),
            40,
        );
        assert!(!pacing.automatic_due(Provider::PiHole, "host", now + Duration::from_secs(39)));
        assert!(pacing.automatic_due(Provider::PiHole, "host", now + Duration::from_secs(40)));
    }

    #[test]
    fn invalidation_does_not_bypass_attempt_gap_or_server_cooldown() {
        let now = Instant::now();
        let mut pacing = Pacing::default();
        pacing.started(Provider::PiHole, now);
        pacing.failed(Provider::PiHole, now, 120);
        pacing.invalidate(Provider::PiHole);
        assert!(!pacing.allowed(Provider::PiHole, now + Duration::from_secs(119)));
        assert!(pacing.allowed(Provider::PiHole, now + Duration::from_secs(120)));
    }
}
