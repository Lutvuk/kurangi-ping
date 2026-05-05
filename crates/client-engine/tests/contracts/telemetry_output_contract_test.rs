use client_engine::routing::{RoutingState, RoutingTransition, RoutingTrigger};
use client_engine::routing::{build_failover_state_payload, FailoverUiState};
use client_engine::telemetry::events::relay_failover::{
    emit_relay_failed, emit_relay_recovered, RelayFailoverEmissionPolicy,
};
use client_engine::telemetry::events::routing::emit_routing_event;
use client_engine::telemetry::TelemetryService;
use std::collections::HashSet;

fn telemetry_allowlist() -> HashSet<&'static str> {
    HashSet::from([
        "app_opened",
        "game_detected",
        "routing_enabled",
        "ping_measured",
        "routing_disabled",
        "crash_reported",
        "relay_failed",
        "relay_recovered",
        "onboarding_completed",
    ])
}

fn transition(
    from: RoutingState,
    to: RoutingState,
    trigger: RoutingTrigger,
    failure_code: Option<&str>,
) -> RoutingTransition {
    RoutingTransition {
        from,
        to,
        trigger,
        failure_code: failure_code.map(ToString::to_string),
    }
}

#[test]
fn emitted_routing_lifecycle_events_conform_to_telemetry_allowlist() {
    let allowlist = telemetry_allowlist();

    let enabled = emit_routing_event(&transition(
        RoutingState::Off,
        RoutingState::Connecting,
        RoutingTrigger::EnableRequested,
        None,
    ))
    .expect("routing_enabled transition should emit event");
    assert!(allowlist.contains(enabled.name));

    let disabled = emit_routing_event(&transition(
        RoutingState::Connected,
        RoutingState::Off,
        RoutingTrigger::DisableRequested,
        None,
    ))
    .expect("routing_disabled transition should emit event");
    assert!(allowlist.contains(disabled.name));

    let failed = emit_routing_event(&transition(
        RoutingState::Connecting,
        RoutingState::Failed,
        RoutingTrigger::ConnectionAttemptFailed,
        Some("ROUTE_ALL_ATTEMPTS_FAILED"),
    ))
    .expect("relay_failed transition should emit event");
    assert!(allowlist.contains(failed.name));
}

#[test]
fn emitted_routing_events_keep_contract_payload_shape() {
    let event = emit_routing_event(&transition(
        RoutingState::Connecting,
        RoutingState::Failed,
        RoutingTrigger::ConnectionAttemptFailed,
        Some("ROUTE_RETRY_BUDGET_EXHAUSTED"),
    ))
    .expect("failed transition should emit event");

    assert_eq!(event.name, "relay_failed");
    assert_eq!(event.payload.state, "failed");
    assert_eq!(event.payload.previous_state, "connecting");
    assert_eq!(
        event.payload.failure_code.as_deref(),
        Some("ROUTE_RETRY_BUDGET_EXHAUSTED")
    );
}

#[test]
fn emitted_relay_failover_events_conform_to_telemetry_allowlist() {
    let allowlist = telemetry_allowlist();
    let mut telemetry = TelemetryService::new();
    let policy = RelayFailoverEmissionPolicy::default();

    let failed_payload = build_failover_state_payload(
        FailoverUiState::Failed,
        Some("sin-01"),
        Some("nrt-01"),
        Some("dead_relay_detected"),
    );
    let recovered_payload = build_failover_state_payload(
        FailoverUiState::Recovered,
        Some("sin-01"),
        Some("nrt-01"),
        Some("switch_successful"),
    );

    emit_relay_failed(&mut telemetry, &failed_payload, 2, policy)
        .expect("relay_failed should emit");
    emit_relay_recovered(&mut telemetry, &recovered_payload, 1, policy)
        .expect("relay_recovered should emit");

    let events = telemetry.drain_batch(10);
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| allowlist.contains(event.name.as_str())));
}

