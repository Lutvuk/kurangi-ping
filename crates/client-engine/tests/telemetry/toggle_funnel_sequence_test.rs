use client_engine::db::RouteSessionCloseRecord;
use client_engine::detection::state_resolver::{
    DetectionMetadata, DetectionResolution, DetectionState,
};
use client_engine::routing::{
    DetectionGateDecision, OffPipelineReasonCode, OffPipelineResult, OffPipelineStatus,
    OnPipelineFailure, OnPipelineReasonCode, OnPipelineResult, OnPipelineStage, OnPipelineStatus,
    RoutingState, SessionHookResult, SessionHookStatus,
};
use client_engine::telemetry::events::detection::{
    emit_game_detected_event, validate_game_detected_payload,
};
use client_engine::telemetry::events::toggle_lifecycle::{
    emit_routing_disabled, emit_routing_enabled, validate_toggle_lifecycle_payload,
};
use client_engine::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunnelKpi {
    total_events: usize,
    activation_events: usize,
    activation_successes: usize,
    deactivation_events: usize,
    deactivation_successes: usize,
    failure_events: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToggleFunnelReport {
    scenario: &'static str,
    event_names: Vec<String>,
    checkpoints: Vec<String>,
    kpi: FunnelKpi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Success,
    ActivationFailure,
    OffFailure,
}

fn run_toggle_funnel_scenario(scenario: Scenario) -> ToggleFunnelReport {
    let mut telemetry = TelemetryService::new();
    let mut checkpoints = vec!["session_started".to_string()];

    let detection = detected_resolution();
    let emitted_detection = emit_game_detected_event(&mut telemetry, DetectionState::NotFound, &detection)
        .expect("game_detected emission should remain schema-safe");
    checkpoints.push(format!("game_detected_emitted={emitted_detection}"));

    match scenario {
        Scenario::Success => {
            let emitted_on = emit_routing_enabled(&mut telemetry, &on_ready_result())
                .expect("routing_enabled success emission should be valid");
            let emitted_off = emit_routing_disabled(
                &mut telemetry,
                &off_result(OffPipelineStatus::Completed, OffPipelineReasonCode::OffRequested),
            )
            .expect("routing_disabled success emission should be valid");
            checkpoints.push(format!("routing_enabled_emitted={emitted_on}"));
            checkpoints.push(format!("routing_disabled_emitted={emitted_off}"));
        }
        Scenario::ActivationFailure => {
            let emitted_on = emit_routing_enabled(
                &mut telemetry,
                &on_blocked_result(OnPipelineReasonCode::ManifestSignatureInvalid),
            )
            .expect("routing_enabled failed emission should be valid");
            checkpoints.push(format!("routing_enabled_failed_emitted={emitted_on}"));
        }
        Scenario::OffFailure => {
            let emitted_on = emit_routing_enabled(&mut telemetry, &on_ready_result())
                .expect("routing_enabled success emission should be valid");
            let emitted_off = emit_routing_disabled(
                &mut telemetry,
                &off_result(OffPipelineStatus::Failed, OffPipelineReasonCode::OffTeardownFailed),
            )
            .expect("routing_disabled failed emission should be valid");
            checkpoints.push(format!("routing_enabled_emitted={emitted_on}"));
            checkpoints.push(format!("routing_disabled_failed_emitted={emitted_off}"));
        }
    }

    let events = telemetry.drain_batch(32);
    validate_payloads(&events);
    validate_impossible_states(&events).expect("scenario should not produce impossible sequence states");

    let event_names = events.iter().map(|event| event.name.clone()).collect::<Vec<_>>();
    let kpi = summarize_kpi(&events);
    checkpoints.push(format!("total_events={}", kpi.total_events));
    checkpoints.push(format!("activation_successes={}", kpi.activation_successes));
    checkpoints.push(format!("deactivation_successes={}", kpi.deactivation_successes));
    checkpoints.push(format!("failure_events={}", kpi.failure_events));

    ToggleFunnelReport {
        scenario: match scenario {
            Scenario::Success => "success",
            Scenario::ActivationFailure => "activation_failure",
            Scenario::OffFailure => "off_failure",
        },
        event_names,
        checkpoints,
        kpi,
    }
}

fn detected_resolution() -> DetectionResolution {
    DetectionResolution {
        state: DetectionState::Detected,
        metadata: DetectionMetadata {
            scanned_at_unix_ms: 1_700_000_000_000,
            last_detected_at_unix_ms: Some(1_700_000_000_000),
            stale_after_ms: 15_000,
            stale_age_ms: Some(0),
            reason_code: None,
            match_count: 1,
            matched_process_id: Some(4242),
            matched_game_id: Some("ffxiv".to_string()),
            matched_executable_name: Some("ffxiv_dx11.exe".to_string()),
        },
    }
}

fn on_ready_result() -> OnPipelineResult {
    OnPipelineResult {
        detection_gate: DetectionGateDecision {
            can_activate: true,
            detection_state: DetectionState::Detected,
            reason_code: None,
        },
        manifest_gate: None,
        route_precheck: None,
        status: OnPipelineStatus::Ready {
            manifest_version: "2026.08.0".to_string(),
            candidates: Vec::new(),
            protocol_order: Vec::new(),
        },
    }
}

fn on_blocked_result(reason_code: OnPipelineReasonCode) -> OnPipelineResult {
    OnPipelineResult {
        detection_gate: DetectionGateDecision {
            can_activate: false,
            detection_state: DetectionState::NotFound,
            reason_code: None,
        },
        manifest_gate: None,
        route_precheck: None,
        status: OnPipelineStatus::Blocked {
            failure: OnPipelineFailure {
                stage: OnPipelineStage::ManifestGate,
                reason_code,
            },
        },
    }
}

fn off_result(status: OffPipelineStatus, reason_code: OffPipelineReasonCode) -> OffPipelineResult {
    OffPipelineResult {
        initial_state: RoutingState::Connected,
        final_state: if status == OffPipelineStatus::Failed {
            RoutingState::Failed
        } else {
            RoutingState::Off
        },
        status,
        reason_code,
        teardown_attempted: true,
        teardown_error: None,
        transitions: Vec::new(),
        session_hook: SessionHookResult {
            operation: "close_route_session",
            status: SessionHookStatus::Persisted,
        },
        close_record: RouteSessionCloseRecord {
            session_id: "sess-1".to_string(),
            ended_at: "2026-08-01T00:00:03Z".to_string(),
            end_reason: Some(reason_code.as_failure_code().to_string()),
        },
    }
}

fn summarize_kpi(events: &[TelemetryEvent]) -> FunnelKpi {
    let activation_events = events
        .iter()
        .filter(|event| event.name == "routing_enabled")
        .count();
    let activation_successes = events
        .iter()
        .filter(|event| event.name == "routing_enabled" && payload_result(event) == Some("success"))
        .count();
    let deactivation_events = events
        .iter()
        .filter(|event| event.name == "routing_disabled")
        .count();
    let deactivation_successes = events
        .iter()
        .filter(|event| event.name == "routing_disabled" && payload_result(event) == Some("success"))
        .count();
    let failure_events = events
        .iter()
        .filter(|event| matches!(payload_result(event), Some("failed")))
        .count();

    FunnelKpi {
        total_events: events.len(),
        activation_events,
        activation_successes,
        deactivation_events,
        deactivation_successes,
        failure_events,
    }
}

fn payload_result(event: &TelemetryEvent) -> Option<&str> {
    match event.payload.get("result") {
        Some(TelemetryValue::Text(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn validate_payloads(events: &[TelemetryEvent]) {
    for event in events {
        if event.name == "game_detected" {
            validate_game_detected_payload(&event.payload)
                .expect("game_detected payload must remain compliant");
        } else {
            validate_toggle_lifecycle_payload(&event.payload)
                .expect("toggle lifecycle payload must remain compliant");
        }
    }
}

fn validate_impossible_states(events: &[TelemetryEvent]) -> Result<(), String> {
    let names = events
        .iter()
        .map(|event| event.name.clone())
        .collect::<Vec<_>>();

    let game_detected_idx = names.iter().position(|name| name == "game_detected");
    let routing_enabled_success_idx = events
        .iter()
        .position(|event| event.name == "routing_enabled" && payload_result(event) == Some("success"));
    let routing_disabled_success_idx = events
        .iter()
        .position(|event| event.name == "routing_disabled" && payload_result(event) == Some("success"));

    if let Some(enabled_idx) = routing_enabled_success_idx {
        let Some(detected_idx) = game_detected_idx else {
            return Err("routing_enabled success emitted before game_detected exists".to_string());
        };
        if detected_idx > enabled_idx {
            return Err("routing_enabled success emitted before game_detected".to_string());
        }
    }

    if let Some(disabled_idx) = routing_disabled_success_idx {
        let Some(enabled_idx) = routing_enabled_success_idx else {
            return Err("routing_disabled success emitted without routing_enabled success".to_string());
        };
        if enabled_idx > disabled_idx {
            return Err("routing_disabled success emitted before routing_enabled success".to_string());
        }
    }

    Ok(())
}

fn validate_missing_or_duplicate_critical(event_names: &[String]) -> Result<(), String> {
    let required = ["game_detected", "routing_enabled", "routing_disabled"];
    for name in required {
        let count = event_names.iter().filter(|event_name| event_name.as_str() == name).count();
        if count == 0 {
            return Err(format!("missing critical event: {name}"));
        }
        if count > 1 {
            return Err(format!("duplicated critical event: {name}"));
        }
    }
    Ok(())
}

fn load_timeline_fixture(name: &str) -> Vec<String> {
    let path = fixture_path(name);
    let raw = fs::read_to_string(&path).expect("timeline fixture should be readable");
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("toggle_event_timelines")
        .join(name)
}

#[test]
fn success_flow_includes_expected_event_order_and_payload_fields() {
    let report = run_toggle_funnel_scenario(Scenario::Success);
    let expected = load_timeline_fixture("success.timeline");

    assert_eq!(report.event_names, expected);
    assert_eq!(report.kpi.activation_successes, 1);
    assert_eq!(report.kpi.deactivation_successes, 1);
    assert_eq!(report.kpi.failure_events, 0);
}

#[test]
fn failure_and_off_paths_do_not_emit_impossible_sequence_states() {
    let activation_failure = run_toggle_funnel_scenario(Scenario::ActivationFailure);
    assert_eq!(activation_failure.kpi.activation_events, 1);
    assert_eq!(activation_failure.kpi.activation_successes, 0);
    assert_eq!(activation_failure.kpi.deactivation_events, 0);

    let off_failure = run_toggle_funnel_scenario(Scenario::OffFailure);
    assert_eq!(off_failure.kpi.activation_successes, 1);
    assert_eq!(off_failure.kpi.deactivation_events, 1);
    assert_eq!(off_failure.kpi.deactivation_successes, 0);
}

#[test]
fn sequence_validation_fails_on_missing_or_duplicated_critical_events() {
    let missing = load_timeline_fixture("missing_enabled.timeline");
    let missing_err =
        validate_missing_or_duplicate_critical(&missing).expect_err("missing event must fail");
    assert_eq!(missing_err, "missing critical event: routing_enabled");

    let duplicated = load_timeline_fixture("duplicate_enabled.timeline");
    let duplicate_err =
        validate_missing_or_duplicate_critical(&duplicated).expect_err("duplicate event must fail");
    assert_eq!(duplicate_err, "duplicated critical event: routing_enabled");
}

#[test]
fn funnel_report_output_is_useful_for_kpi_diagnostics() {
    let report = run_toggle_funnel_scenario(Scenario::Success);
    let checkpoint_set = report
        .checkpoints
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(report.scenario, "success");
    assert!(report.kpi.total_events >= 3);
    assert!(checkpoint_set.contains("session_started"));
    assert!(
        checkpoint_set
            .iter()
            .any(|row| row.starts_with("activation_successes=")),
        "checkpoint should contain activation KPI summary"
    );
    assert!(
        checkpoint_set
            .iter()
            .any(|row| row.starts_with("deactivation_successes=")),
        "checkpoint should contain deactivation KPI summary"
    );
}
