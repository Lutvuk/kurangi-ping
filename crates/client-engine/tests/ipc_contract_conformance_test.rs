use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContractTarget {
    RoutingLifecycleResponse,
    DetectionStatusResponse,
    RoutingStateChangedEvent,
    MetricsPingSampledEvent,
    DetectionStatusUpdatedEvent,
}

#[derive(Debug, Clone, PartialEq)]
struct ContractCase {
    name: &'static str,
    target: ContractTarget,
    payload: Value,
    expected_valid: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ContractConformanceResult {
    name: &'static str,
    target: ContractTarget,
    expected_valid: bool,
    actual_valid: bool,
    diagnostics: Vec<String>,
}

fn run_ipc_contract_suite() -> Vec<ContractConformanceResult> {
    let cases = vec![
        ContractCase {
            name: "routing_response_valid",
            target: ContractTarget::RoutingLifecycleResponse,
            payload: json!({
                "state": "connecting",
                "reason_code": null,
                "message": null
            }),
            expected_valid: true,
        },
        ContractCase {
            name: "routing_response_invalid_state",
            target: ContractTarget::RoutingLifecycleResponse,
            payload: json!({
                "state": "arming",
                "reason_code": "ipc_unknown_failure"
            }),
            expected_valid: false,
        },
        ContractCase {
            name: "detection_response_valid",
            target: ContractTarget::DetectionStatusResponse,
            payload: json!({
                "state": "detected",
                "game_id": "ffxiv",
                "process_name": "ffxiv_dx11.exe",
                "detection_time_ms": 1700000000000u64,
                "reason_code": null,
                "message": null
            }),
            expected_valid: true,
        },
        ContractCase {
            name: "detection_response_missing_state",
            target: ContractTarget::DetectionStatusResponse,
            payload: json!({
                "game_id": "ffxiv"
            }),
            expected_valid: false,
        },
        ContractCase {
            name: "routing_event_valid",
            target: ContractTarget::RoutingStateChangedEvent,
            payload: json!({
                "previous_state": "idle",
                "state": "active",
                "reason_code": null,
                "message": null
            }),
            expected_valid: true,
        },
        ContractCase {
            name: "routing_event_invalid_previous_state",
            target: ContractTarget::RoutingStateChangedEvent,
            payload: json!({
                "previous_state": "booting",
                "state": "active"
            }),
            expected_valid: false,
        },
        ContractCase {
            name: "metrics_event_valid",
            target: ContractTarget::MetricsPingSampledEvent,
            payload: json!({
                "sampled_at_unix_ms": 1700000001000u64,
                "state": "live",
                "baseline_ping_ms": 210.5,
                "routed_ping_ms": 152.0,
                "jitter_ms": 3.2,
                "packet_loss_pct": 0.0,
                "reason_code": null
            }),
            expected_valid: true,
        },
        ContractCase {
            name: "metrics_event_invalid_packet_loss_type",
            target: ContractTarget::MetricsPingSampledEvent,
            payload: json!({
                "sampled_at_unix_ms": 1700000001000u64,
                "state": "degraded",
                "baseline_ping_ms": 210.5,
                "routed_ping_ms": null,
                "jitter_ms": null,
                "packet_loss_pct": "0.0"
            }),
            expected_valid: false,
        },
        ContractCase {
            name: "detection_event_valid",
            target: ContractTarget::DetectionStatusUpdatedEvent,
            payload: json!({
                "state": "not_detected",
                "game_id": null,
                "process_name": null,
                "detection_time_ms": 1700000002000u64,
                "reason_code": "ipc_detection_scan_failed",
                "message": "game detection scan failed"
            }),
            expected_valid: true,
        },
        ContractCase {
            name: "detection_event_invalid_reason_code_type",
            target: ContractTarget::DetectionStatusUpdatedEvent,
            payload: json!({
                "state": "not_detected",
                "reason_code": 42
            }),
            expected_valid: false,
        },
    ];

    cases
        .iter()
        .map(|case| {
            let diagnostics = match case.target {
                ContractTarget::RoutingLifecycleResponse => {
                    validate_routing_response_payload(&case.payload)
                }
                ContractTarget::DetectionStatusResponse => {
                    validate_detection_response_payload(&case.payload)
                }
                ContractTarget::RoutingStateChangedEvent => {
                    validate_routing_event_payload(&case.payload)
                }
                ContractTarget::MetricsPingSampledEvent => {
                    validate_metrics_event_payload(&case.payload)
                }
                ContractTarget::DetectionStatusUpdatedEvent => {
                    validate_detection_event_payload(&case.payload)
                }
            };

            ContractConformanceResult {
                name: case.name,
                target: case.target.clone(),
                expected_valid: case.expected_valid,
                actual_valid: diagnostics.is_empty(),
                diagnostics,
            }
        })
        .collect()
}

fn validate_routing_response_payload(payload: &Value) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let Some(object) = payload.as_object() else {
        return vec!["payload must be object".to_string()];
    };

    require_enum(
        object.get("state"),
        &["idle", "connecting", "active", "degraded", "error"],
        "state",
        &mut diagnostics,
    );
    require_optional_string(object.get("reason_code"), "reason_code", &mut diagnostics);
    require_optional_string(object.get("message"), "message", &mut diagnostics);
    diagnostics
}

fn validate_detection_response_payload(payload: &Value) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let Some(object) = payload.as_object() else {
        return vec!["payload must be object".to_string()];
    };

    require_enum(
        object.get("state"),
        &["detected", "not_detected"],
        "state",
        &mut diagnostics,
    );
    require_optional_string(object.get("game_id"), "game_id", &mut diagnostics);
    require_optional_string(object.get("process_name"), "process_name", &mut diagnostics);
    require_optional_u64(object.get("detection_time_ms"), "detection_time_ms", &mut diagnostics);
    require_optional_string(object.get("reason_code"), "reason_code", &mut diagnostics);
    require_optional_string(object.get("message"), "message", &mut diagnostics);
    diagnostics
}

fn validate_routing_event_payload(payload: &Value) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let Some(object) = payload.as_object() else {
        return vec!["payload must be object".to_string()];
    };

    require_enum(
        object.get("previous_state"),
        &["idle", "connecting", "active", "degraded", "error"],
        "previous_state",
        &mut diagnostics,
    );
    require_enum(
        object.get("state"),
        &["idle", "connecting", "active", "degraded", "error"],
        "state",
        &mut diagnostics,
    );
    require_optional_string(object.get("reason_code"), "reason_code", &mut diagnostics);
    require_optional_string(object.get("message"), "message", &mut diagnostics);
    diagnostics
}

fn validate_metrics_event_payload(payload: &Value) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let Some(object) = payload.as_object() else {
        return vec!["payload must be object".to_string()];
    };

    require_u64(object.get("sampled_at_unix_ms"), "sampled_at_unix_ms", &mut diagnostics);
    require_enum(
        object.get("state"),
        &["live", "measuring", "degraded"],
        "state",
        &mut diagnostics,
    );
    require_optional_number(object.get("baseline_ping_ms"), "baseline_ping_ms", &mut diagnostics);
    require_optional_number(object.get("routed_ping_ms"), "routed_ping_ms", &mut diagnostics);
    require_optional_number(object.get("jitter_ms"), "jitter_ms", &mut diagnostics);
    require_optional_number(object.get("packet_loss_pct"), "packet_loss_pct", &mut diagnostics);
    require_optional_string(object.get("reason_code"), "reason_code", &mut diagnostics);
    diagnostics
}

fn validate_detection_event_payload(payload: &Value) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let Some(object) = payload.as_object() else {
        return vec!["payload must be object".to_string()];
    };

    require_enum(
        object.get("state"),
        &["detected", "not_detected"],
        "state",
        &mut diagnostics,
    );
    require_optional_string(object.get("game_id"), "game_id", &mut diagnostics);
    require_optional_string(object.get("process_name"), "process_name", &mut diagnostics);
    require_optional_u64(object.get("detection_time_ms"), "detection_time_ms", &mut diagnostics);
    require_optional_string(object.get("reason_code"), "reason_code", &mut diagnostics);
    require_optional_string(object.get("message"), "message", &mut diagnostics);
    diagnostics
}

fn require_enum(
    value: Option<&Value>,
    allowed: &[&str],
    field: &str,
    diagnostics: &mut Vec<String>,
) {
    match value.and_then(Value::as_str) {
        Some(candidate) if allowed.contains(&candidate) => {}
        Some(candidate) => diagnostics.push(format!(
            "{field} must be one of [{}], received '{candidate}'",
            allowed.join(",")
        )),
        None => diagnostics.push(format!("{field} must be string enum")),
    }
}

fn require_u64(value: Option<&Value>, field: &str, diagnostics: &mut Vec<String>) {
    if value.and_then(Value::as_u64).is_none() {
        diagnostics.push(format!("{field} must be unsigned integer"));
    }
}

fn require_optional_u64(value: Option<&Value>, field: &str, diagnostics: &mut Vec<String>) {
    if let Some(candidate) = value {
        if !(candidate.is_null() || candidate.as_u64().is_some()) {
            diagnostics.push(format!("{field} must be unsigned integer or null"));
        }
    }
}

fn require_optional_string(value: Option<&Value>, field: &str, diagnostics: &mut Vec<String>) {
    if let Some(candidate) = value {
        if !(candidate.is_null() || candidate.as_str().is_some()) {
            diagnostics.push(format!("{field} must be string or null"));
        }
    }
}

fn require_optional_number(value: Option<&Value>, field: &str, diagnostics: &mut Vec<String>) {
    if let Some(candidate) = value {
        if !(candidate.is_null() || candidate.is_number()) {
            diagnostics.push(format!("{field} must be number or null"));
        }
    }
}

#[test]
fn ipc_payload_fields_match_documented_contracts() {
    let suite = run_ipc_contract_suite();
    let valid_cases = suite
        .iter()
        .filter(|entry| entry.expected_valid)
        .collect::<Vec<_>>();
    assert!(!valid_cases.is_empty(), "suite must include valid contract fixtures");

    for entry in valid_cases {
        assert!(
            entry.actual_valid,
            "expected '{}' to conform, diagnostics: {:?}",
            entry.name,
            entry.diagnostics
        );
    }
}

#[test]
fn missing_or_invalid_payload_fields_fail_conformance_assertions() {
    let suite = run_ipc_contract_suite();
    let invalid_cases = suite
        .iter()
        .filter(|entry| !entry.expected_valid)
        .collect::<Vec<_>>();
    assert!(
        !invalid_cases.is_empty(),
        "suite must include invalid contract fixtures"
    );

    for entry in invalid_cases {
        assert!(
            !entry.actual_valid,
            "expected '{}' to fail conformance checks",
            entry.name
        );
        assert!(
            !entry.diagnostics.is_empty(),
            "invalid case '{}' must provide diagnostics",
            entry.name
        );
    }
}

#[test]
fn output_contains_actionable_release_readiness_diagnostics() {
    let suite = run_ipc_contract_suite();
    let diagnostics = suite
        .iter()
        .filter(|entry| !entry.actual_valid)
        .flat_map(|entry| {
            entry
                .diagnostics
                .iter()
                .map(move |item| format!("{}::{item}", entry.name))
        })
        .collect::<Vec<_>>();

    assert!(!diagnostics.is_empty(), "expected at least one invalid fixture");
    assert!(
        diagnostics
            .iter()
            .any(|line| line.contains("routing_response_invalid_state::state")),
        "expected routing state diagnostic to be actionable"
    );
    assert!(
        diagnostics
            .iter()
            .any(|line| line.contains("metrics_event_invalid_packet_loss_type::packet_loss_pct")),
        "expected metrics field diagnostic to be actionable"
    );
}

#[test]
fn run_output_is_deterministic_across_repeated_executions() {
    let first = run_ipc_contract_suite();
    let second = run_ipc_contract_suite();
    assert_eq!(first, second);
}
