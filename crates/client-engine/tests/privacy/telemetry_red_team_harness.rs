use client_engine::telemetry::scrubber::SensitiveFieldPolicy;
use client_engine::telemetry::validator::UnknownKeyPolicy;
use client_engine::telemetry::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SubmissionMode {
    Validated,
    Direct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PolicyMode {
    Reject,
    Sanitize,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RedTeamCase {
    id: String,
    submission: SubmissionMode,
    policy: PolicyMode,
    event_name: String,
    payload: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RedTeamFixture {
    cases: Vec<RedTeamCase>,
}

#[derive(Debug, Clone)]
struct RedTeamSuiteReport {
    reject_attempts: usize,
    sanitize_attempts: usize,
    reject_sensitive_violations: usize,
    reject_diagnostics_count: usize,
    sanitize_diagnostics_count: usize,
    reject_queue_depth: usize,
    sanitize_queue_depth_before_drain: usize,
    sanitize_events: Vec<TelemetryEvent>,
    diagnostics_rendered: String,
}

fn fixture() -> RedTeamFixture {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = base
        .join("tests")
        .join("fixtures")
        .join("privacy")
        .join("telemetry_red_team_payloads.json");
    let content = fs::read_to_string(path).expect("red-team fixture should be readable");
    serde_json::from_str(&content).expect("red-team fixture should be valid JSON")
}

fn payload_to_telemetry_payload(
    case_id: &str,
    payload: &BTreeMap<String, Value>,
) -> Result<TelemetryPayload, String> {
    let mut mapped = BTreeMap::new();
    for (key, value) in payload {
        let telemetry_value = match value {
            Value::String(text) => TelemetryValue::Text(text.clone()),
            Value::Number(number) => {
                if let Some(integer) = number.as_i64() {
                    TelemetryValue::Integer(integer)
                } else if let Some(float) = number.as_f64() {
                    TelemetryValue::Float(float)
                } else {
                    return Err(format!("case '{case_id}': unsupported number for key '{key}'"));
                }
            }
            _ => {
                return Err(format!(
                    "case '{case_id}': key '{key}' must be string/number in v1 test fixture"
                ))
            }
        };
        mapped.insert(key.clone(), telemetry_value);
    }
    Ok(mapped)
}

fn run_privacy_violation_suite(cases: &[RedTeamCase], rounds: usize) -> RedTeamSuiteReport {
    let mut reject_service =
        TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::reject());
    let mut sanitize_service =
        TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::sanitize());

    let mut reject_attempts = 0;
    let mut sanitize_attempts = 0;
    let mut reject_sensitive_violations = 0;

    for _ in 0..rounds {
        for case in cases {
            let payload = payload_to_telemetry_payload(&case.id, &case.payload)
                .expect("fixture payload must convert to telemetry payload");

            match case.policy {
                PolicyMode::Reject => {
                    reject_attempts += 1;
                    match case.submission {
                        SubmissionMode::Validated => {
                            let result = reject_service.enqueue_validated(
                                case.event_name.clone(),
                                payload,
                                UnknownKeyPolicy::Reject,
                            );
                            match result {
                                Ok(()) => panic!("case '{}' should reject under policy", case.id),
                                Err(error) => {
                                    if error.code.as_str() == "sensitive_field_violation" {
                                        reject_sensitive_violations += 1;
                                    }
                                }
                            }
                        }
                        SubmissionMode::Direct => {
                            let before = reject_service.queue_depth();
                            reject_service.enqueue(TelemetryEvent::new(case.event_name.clone(), payload));
                            let after = reject_service.queue_depth();
                            assert_eq!(
                                before, after,
                                "case '{}' should not enqueue when direct+reject violation occurs",
                                case.id
                            );
                        }
                    }
                }
                PolicyMode::Sanitize => {
                    sanitize_attempts += 1;
                    match case.submission {
                        SubmissionMode::Validated => {
                            sanitize_service
                                .enqueue_validated(
                                    case.event_name.clone(),
                                    payload,
                                    UnknownKeyPolicy::Reject,
                                )
                                .expect("validated sanitize case should pass with scrubbing");
                        }
                        SubmissionMode::Direct => {
                            sanitize_service.enqueue(TelemetryEvent::new(case.event_name.clone(), payload));
                        }
                    }
                }
            }
        }
    }

    let reject_diagnostics = reject_service.take_privacy_diagnostics();
    let sanitize_diagnostics = sanitize_service.take_privacy_diagnostics();
    let sanitize_queue_depth_before_drain = sanitize_service.queue_depth();
    let sanitize_events = sanitize_service.drain_batch(10_000);
    let diagnostics_rendered = format!("{reject_diagnostics:?}\n{sanitize_diagnostics:?}");

    RedTeamSuiteReport {
        reject_attempts,
        sanitize_attempts,
        reject_sensitive_violations,
        reject_diagnostics_count: reject_diagnostics.len(),
        sanitize_diagnostics_count: sanitize_diagnostics.len(),
        reject_queue_depth: reject_service.queue_depth(),
        sanitize_queue_depth_before_drain,
        sanitize_events,
        diagnostics_rendered,
    }
}

#[test]
fn pii_like_fields_are_always_rejected_or_scrubbed_by_policy() {
    let data = fixture();
    let report = run_privacy_violation_suite(&data.cases, 1);

    assert_eq!(report.reject_attempts, 3);
    assert_eq!(report.sanitize_attempts, 3);
    assert_eq!(report.reject_sensitive_violations, 3);
    assert_eq!(report.reject_diagnostics_count, 3);
    // This fixture injects at least five sanitize-mode sensitive-field violations:
    // username, email, phone_number, packet_payload, and raw_destination_host_list.
    // Future scrubber improvements may split nested payload findings into more entries.
    assert!(report.sanitize_diagnostics_count >= 5);

    for event in &report.sanitize_events {
        assert!(!event.payload.contains_key("username"));
        assert!(!event.payload.contains_key("email"));
        assert!(!event.payload.contains_key("phone_number"));
        assert!(!event.payload.contains_key("packet_payload"));
    }
}

#[test]
fn nested_sensitive_keys_are_detected() {
    let data = fixture();
    let report = run_privacy_violation_suite(&data.cases, 1);

    assert!(report.reject_diagnostics_count >= 3);
    assert!(
        report.diagnostics_rendered.contains("process_name.username")
            || report.diagnostics_rendered.contains("process_name.network.ip_address")
    );
}

#[test]
fn pipeline_remains_stable_under_repeated_violation_attempts() {
    let data = fixture();
    let report = run_privacy_violation_suite(&data.cases, 200);

    assert_eq!(report.reject_queue_depth, 0);
    assert_eq!(report.reject_sensitive_violations, report.reject_attempts);
    assert!(report.sanitize_queue_depth_before_drain > 0);
    assert!(!report.sanitize_events.is_empty());
}

#[test]
fn diagnostics_remain_non_sensitive() {
    let data = fixture();
    let report = run_privacy_violation_suite(&data.cases, 2);
    let rendered = report.diagnostics_rendered;

    assert!(!rendered.contains("red-user"));
    assert!(!rendered.contains("red@example.net"));
    assert!(!rendered.contains("10.10.10.10"));
    assert!(!rendered.contains("ops-admin"));
    assert!(!rendered.contains("0xDEADBEEF"));
    assert!(!rendered.contains("internal-a.local"));
}
