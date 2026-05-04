use client_engine::routing::{RoutingState, RoutingTransition, RoutingTrigger};
use client_engine::telemetry::events::routing::emit_routing_event;
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

