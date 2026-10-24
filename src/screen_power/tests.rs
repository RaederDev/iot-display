use super::policy::*;
use std::time::{Duration, Instant};

fn healthy(now: Instant) -> Policy {
    let mut policy = Policy::new(true, now);
    policy.health(true, now);
    policy
}

#[test]
fn repeated_cycles_without_input_and_exact_idle_boundary() {
    let start = Instant::now();
    let mut policy = healthy(start);
    let mut output = OutputState {
        mode: Some(true),
        ..Default::default()
    };
    for cycle in 0..3 {
        let now = start + IDLE * cycle;
        policy.presence(false, now);
        assert_eq!(
            policy.decide(now + IDLE - Duration::from_millis(1), true, false),
            None
        );
        let sleep = now + IDLE;
        assert_eq!(policy.decide(sleep, true, false), Some(Action::Sleep));
        assert!(output.request(false, sleep));
        output.confirmed(false);
        assert!(
            !output.request(false, sleep),
            "already asleep must not resend off"
        );
        policy.presence(true, sleep);
        assert_eq!(policy.decide(sleep, true, false), Some(Action::SensorWake));
        assert!(output.request(true, sleep));
        output.confirmed(true);
        policy.woke(sleep);
        assert!(!output.blanked);
    }
}

#[test]
fn continuous_presence_extends_idle_even_during_cooldown() {
    let now = Instant::now();
    let mut policy = healthy(now);
    policy.woke(now);
    for second in 0..1200 {
        let tick = now + Duration::from_secs(second);
        policy.presence(true, tick);
        assert_ne!(policy.decide(tick, true, false), Some(Action::Sleep));
    }
    policy.presence(false, now + Duration::from_secs(1200));
    assert_eq!(
        policy.decide(now + Duration::from_secs(1798), true, false),
        None
    );
    assert_eq!(
        policy.decide(now + Duration::from_secs(1799), true, false),
        Some(Action::Sleep)
    );
}

#[test]
fn cooldown_applies_only_after_success_and_input_bypasses_it() {
    let now = Instant::now();
    let mut policy = healthy(now);
    policy.presence(true, now);
    assert_eq!(policy.decide(now, true, false), Some(Action::SensorWake));
    // A failed or merely already-on wake must not call woke().
    assert_eq!(
        policy.decide(now + Duration::from_secs(1), true, false),
        Some(Action::SensorWake)
    );
    policy.woke(now);
    assert_eq!(
        policy.decide(now + COOLDOWN - Duration::from_millis(1), true, false),
        None
    );
    assert_eq!(
        policy.decide(now + Duration::from_secs(1), false, true),
        Some(Action::InputWake)
    );
    policy.configure(true, now + Duration::from_secs(2));
    assert_eq!(
        policy.decide(now + Duration::from_secs(3), true, false),
        None
    );
    assert_eq!(
        policy.decide(now + COOLDOWN, true, false),
        Some(Action::SensorWake)
    );
}

#[test]
fn input_activity_and_inhibitors_prevent_sleep_and_input_wakes() {
    let now = Instant::now();
    let policy = healthy(now);
    // ext-idle-notify remains active while input occurs anywhere or an inhibitor exists.
    assert_eq!(policy.decide(now + IDLE * 2, false, false), None);
    assert_eq!(
        policy.decide(now + IDLE * 2, true, false),
        Some(Action::Sleep)
    );
    assert_eq!(
        policy.decide(now + IDLE * 2, false, true),
        Some(Action::InputWake)
    );
}

#[test]
fn unavailable_disabled_and_recovery_are_fail_awake_with_fresh_grace() {
    let now = Instant::now();
    let mut policy = healthy(now);
    policy.health(false, now + IDLE);
    assert_eq!(
        policy.decide(now + IDLE, true, false),
        Some(Action::Restore)
    );
    policy.health(true, now + IDLE);
    assert_eq!(policy.decide(now + IDLE, true, false), None);
    assert_eq!(
        policy.decide(now + IDLE * 2, true, false),
        Some(Action::Sleep)
    );
    policy.configure(false, now + IDLE * 2);
    assert_eq!(
        policy.decide(now + IDLE * 3, true, true),
        Some(Action::Restore)
    );
    policy.configure(true, now + IDLE * 3);
    assert_eq!(policy.decide(now + IDLE * 3, true, false), None);
}

#[test]
fn partial_and_late_acknowledgements_keep_restore_ownership() {
    let now = Instant::now();
    let mut outputs = [OutputState::default(), OutputState::default()];
    for output in &mut outputs {
        output.confirmed(true);
        assert!(output.request(false, now));
    }
    outputs[0].confirmed(false);
    assert!(!outputs[1].timed_out(now + TIMEOUT - Duration::from_millis(1)));
    assert!(outputs[1].timed_out(now + TIMEOUT));
    assert!(outputs.iter().all(|output| output.blanked));
    for output in &mut outputs {
        assert!(output.request(true, now + TIMEOUT));
    }
    outputs[1].confirmed(false);
    assert!(outputs[1].pending.is_some());
    outputs[0].confirmed(true);
    assert!(outputs[1].timed_out(now + TIMEOUT * 2));
    outputs[1].confirmed(true);
    assert!(outputs.iter().all(|output| !output.blanked));
    assert!(!outputs[1].request(true, now + TIMEOUT * 2));
}

#[test]
fn restore_reverses_unacknowledged_off_and_requests_coalesce() {
    let now = Instant::now();
    let mut output = OutputState {
        mode: Some(true),
        ..Default::default()
    };
    assert!(output.request(false, now));
    assert!(!output.request(false, now));
    assert!(output.request(true, now));
    output.confirmed(false);
    assert!(output.blanked);
    assert!(output.pending.unwrap().0);
    output.confirmed(true);
    assert!(!output.blanked);
}
