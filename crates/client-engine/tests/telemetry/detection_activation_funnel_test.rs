use client_engine::detection::state_resolver::{
    DetectionMetadata, DetectionReasonCode, DetectionResolution, DetectionState,
};
use client_engine::routing::{
    can_activate_routing, RoutingState, RoutingTransition, RoutingTrigger,
};
use client_engine::telemetry::events::detection::{
    emit_game_detected_event, validate_game_detected_payload,
};
use client_engine::telemetry::events::routing::emit_routing_event;
use client_engine::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct FunnelReport {
    scenario: &'static str,
    event_names: Vec<String>,
    checkpoints: Vec<String>,
    drop_off_point: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum FunnelScenario {
    Success,
    NotFound,
    Error,
}

fn detection_resolution(scenario: FunnelScenario) -> DetectionResolution {
    match scenario {
        FunnelScenario::Success => DetectionResolution {
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
        },
        FunnelScenario::NotFound => DetectionResolution {
            state: DetectionState::NotFound,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 1_700_000_000_000,
                last_detected_at_unix_ms: None,
                stale_after_ms: 15_000,
                stale_age_ms: None,
                reason_code: None,
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        },
        FunnelScenario::Error => DetectionResolution {
            state: DetectionState::Error,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 1_700_000_000_000,
                last_detected_at_unix_ms: Some(1_699_999_990_000),
                stale_after_ms: 15_000,
                stale_age_ms: Some(10_000),
                reason_code: Some(DetectionReasonCode::PermissionDenied),
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        },
    }
}

fn routing_lifecycle_to_telemetry(
    event_name: &str,
    payload: &client_engine::telemetry::events::routing::RoutingEventPayload,
) -> TelemetryEvent {
    let mut body = BTreeMap::from([
        (
            "state".to_string(),
            TelemetryValue::Text(payload.state.to_string()),
        ),
        (
            "previous_state".to_string(),
            TelemetryValue::Text(payload.previous_state.to_string()),
        ),
    ]);

    if let Some(code) = &payload.failure_code {
        body.insert(
            "failure_code".to_string(),
            TelemetryValue::Text(code.clone()),
        );
    }

    TelemetryEvent::new(event_name.to_string(), body)
}

fn validate_routing_payload_shape(event: &TelemetryEvent) -> Result<(), String> {
    let allowed_keys = BTreeSet::from([
        "state".to_string(),
        "previous_state".to_string(),
        "failure_code".to_string(),
    ]);

    let actual_keys = event.payload.keys().cloned().collect::<BTreeSet<_>>();
    if !actual_keys.is_subset(&allowed_keys) {
        return Err(format!(
            "routing payload contains unsupported keys for {}",
            event.name
        ));
    }

    match event.payload.get("state") {
        Some(TelemetryValue::Text(v)) if !v.trim().is_empty() => {}
        _ => return Err(format!("{} payload missing non-empty state", event.name)),
    }

    match event.payload.get("previous_state") {
        Some(TelemetryValue::Text(v)) if !v.trim().is_empty() => {}
        _ => {
            return Err(format!(
                "{} payload missing non-empty previous_state",
                event.name
            ))
        }
    }

    match event.name.as_str() {
        "routing_enabled" | "routing_disabled" => {
            if event.payload.contains_key("failure_code") {
                return Err(format!(
                    "{} should not include failure_code in success/normal flow",
                    event.name
                ));
            }
        }
        "relay_failed" => {
            if let Some(value) = event.payload.get("failure_code") {
                match value {
                    TelemetryValue::Text(v) if !v.trim().is_empty() => {}
                    _ => return Err("relay_failed failure_code must be non-empty text".to_string()),
                }
            }
        }
        _ => return Err(format!("unexpected routing event: {}", event.name)),
    }

    Ok(())
}

fn assert_funnel_event_schema(event: &TelemetryEvent) {
    if event.name == "game_detected" {
        validate_game_detected_payload(&event.payload)
            .expect("game_detected payload schema must stay compliant");
    } else {
        validate_routing_payload_shape(event).expect("routing payload schema must stay compliant");
    }
}

fn run_detection_activation_funnel(scenario: FunnelScenario) -> FunnelReport {
    let scenario_name = match scenario {
        FunnelScenario::Success => "success",
        FunnelScenario::NotFound => "not_found_drop_off",
        FunnelScenario::Error => "error_drop_off",
    };

    let resolution = detection_resolution(scenario);
    let mut telemetry = TelemetryService::new();
    let mut checkpoints = vec![format!("detection_resolved:{}", resolution.state.as_str())];

    let emitted = emit_game_detected_event(&mut telemetry, DetectionState::NotFound, &resolution)
        .expect("detection telemetry guard should not fail unexpectedly");
    checkpoints.push(format!("game_detected_emitted:{}", emitted));

    let mut events = telemetry.drain_batch(16);

    let decision = can_activate_routing(&resolution);
    checkpoints.push(format!("routing_gate_allowed:{}", decision.can_activate));

    let mut drop_off_point = None;
    if decision.can_activate {
        let transition = RoutingTransition {
            from: RoutingState::Off,
            to: RoutingState::Connecting,
            trigger: RoutingTrigger::EnableRequested,
            failure_code: None,
        };

        if let Some(routing_event) = emit_routing_event(&transition) {
            checkpoints.push("routing_enabled_event_emitted:true".to_string());
            events.push(routing_lifecycle_to_telemetry(
                routing_event.name,
                &routing_event.payload,
            ));
        }
    } else {
        let reason = decision
            .reason_code
            .map(|code| code.as_str().to_string())
            .unwrap_or_else(|| "unknown_reason".to_string());
        drop_off_point = Some(format!("routing_gate_denied:{reason}"));
        checkpoints.push("routing_enabled_event_emitted:false".to_string());
    }

    for event in &events {
        assert_funnel_event_schema(event);
    }

    let event_names = events.iter().map(|event| event.name.clone()).collect();

    FunnelReport {
        scenario: scenario_name,
        event_names,
        checkpoints,
        drop_off_point,
    }
}

fn find_event_index(events: &[String], name: &str) -> Option<usize> {
    events.iter().position(|event_name| event_name == name)
}

#[test]
fn success_flow_emits_game_detected_before_routing_activation() {
    let report = run_detection_activation_funnel(FunnelScenario::Success);

    let game_detected_idx =
        find_event_index(&report.event_names, "game_detected").unwrap_or_else(|| {
            panic!(
                "missing game_detected event; checkpoints={:?}; drop_off={:?}",
                report.checkpoints, report.drop_off_point
            )
        });
    let routing_enabled_idx = find_event_index(&report.event_names, "routing_enabled")
        .unwrap_or_else(|| {
            panic!(
                "missing routing_enabled event; checkpoints={:?}; drop_off={:?}",
                report.checkpoints, report.drop_off_point
            )
        });

    assert!(
        game_detected_idx < routing_enabled_idx,
        "event order invalid; scenario={}; events={:?}; checkpoints={:?}",
        report.scenario,
        report.event_names,
        report.checkpoints
    );
}

#[test]
fn negative_flows_do_not_emit_invalid_activation_events() {
    let not_found_report = run_detection_activation_funnel(FunnelScenario::NotFound);
    assert!(
        !not_found_report
            .event_names
            .iter()
            .any(|name| name == "routing_enabled"),
        "not_found flow must drop off before activation; checkpoints={:?}; drop_off={:?}",
        not_found_report.checkpoints,
        not_found_report.drop_off_point
    );
    assert_eq!(
        not_found_report.drop_off_point.as_deref(),
        Some("routing_gate_denied:detection_not_found")
    );

    let error_report = run_detection_activation_funnel(FunnelScenario::Error);
    assert!(
        !error_report
            .event_names
            .iter()
            .any(|name| name == "routing_enabled"),
        "error flow must not emit activation event; checkpoints={:?}; drop_off={:?}",
        error_report.checkpoints,
        error_report.drop_off_point
    );
    assert_eq!(
        error_report.drop_off_point.as_deref(),
        Some("routing_gate_denied:detection_error_permission_denied")
    );
}

#[test]
fn funnel_payload_schema_stays_compliant_for_all_emitted_events() {
    let report = run_detection_activation_funnel(FunnelScenario::Success);

    let expected_events =
        BTreeSet::from(["game_detected".to_string(), "routing_enabled".to_string()]);
    let actual_events = report.event_names.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(
        actual_events, expected_events,
        "unexpected funnel event set in success flow; checkpoints={:?}",
        report.checkpoints
    );
}

#[test]
fn drop_off_points_are_explicit_for_debugging() {
    let not_found_report = run_detection_activation_funnel(FunnelScenario::NotFound);
    let error_report = run_detection_activation_funnel(FunnelScenario::Error);

    assert!(
        not_found_report.drop_off_point.is_some(),
        "not_found flow must produce explicit drop-off point"
    );
    assert!(
        error_report.drop_off_point.is_some(),
        "error flow must produce explicit drop-off point"
    );
}
