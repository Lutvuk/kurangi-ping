use std::collections::{BTreeMap, BTreeSet};

use crate::routing::{
    OffPipelineResult, OffPipelineStatus, OnPipelineResult, OnPipelineStatus,
};
use crate::telemetry::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};

pub const ROUTING_ENABLED_EVENT_NAME: &str = "routing_enabled";
pub const ROUTING_DISABLED_EVENT_NAME: &str = "routing_disabled";
pub const TOGGLE_LIFECYCLE_ALLOWED_KEYS: [&str; 3] =
    ["result", "reason_code", "lifecycle_state"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToggleLifecycleSchemaErrorCode {
    InvalidPayloadKeys,
    MissingRequiredField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleLifecycleSchemaError {
    pub code: ToggleLifecycleSchemaErrorCode,
    pub message: String,
}

impl ToggleLifecycleSchemaError {
    fn new(code: ToggleLifecycleSchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_routing_enabled(
    telemetry: &mut TelemetryService,
    result: &OnPipelineResult,
) -> Result<bool, ToggleLifecycleSchemaError> {
    let (outcome, reason_code, lifecycle_state) = match &result.status {
        OnPipelineStatus::Ready { .. } => ("success", "none".to_string(), "active"),
        OnPipelineStatus::Blocked { failure } => (
            "failed",
            sanitize_reason_code(failure.reason_code.as_str()),
            "error",
        ),
    };

    let payload = build_payload(outcome, &reason_code, lifecycle_state);
    validate_toggle_lifecycle_payload(&payload)?;
    telemetry.enqueue(TelemetryEvent::new(ROUTING_ENABLED_EVENT_NAME, payload));
    Ok(true)
}

pub fn emit_routing_disabled(
    telemetry: &mut TelemetryService,
    result: &OffPipelineResult,
) -> Result<bool, ToggleLifecycleSchemaError> {
    let (outcome, reason_code, lifecycle_state) = match result.status {
        OffPipelineStatus::Completed | OffPipelineStatus::Idempotent => (
            "success",
            sanitize_reason_code(result.reason_code.as_str()),
            "idle",
        ),
        OffPipelineStatus::Failed => (
            "failed",
            sanitize_reason_code(result.reason_code.as_str()),
            "error",
        ),
    };

    let payload = build_payload(outcome, &reason_code, lifecycle_state);
    validate_toggle_lifecycle_payload(&payload)?;
    telemetry.enqueue(TelemetryEvent::new(ROUTING_DISABLED_EVENT_NAME, payload));
    Ok(true)
}

fn build_payload(outcome: &str, reason_code: &str, lifecycle_state: &str) -> TelemetryPayload {
    BTreeMap::from([
        ("result".to_string(), TelemetryValue::Text(outcome.to_string())),
        (
            "reason_code".to_string(),
            TelemetryValue::Text(reason_code.to_string()),
        ),
        (
            "lifecycle_state".to_string(),
            TelemetryValue::Text(lifecycle_state.to_string()),
        ),
    ])
}

fn sanitize_reason_code(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "safe_generic_issue".to_string();
    }

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        normalized
    } else {
        "safe_generic_issue".to_string()
    }
}

pub fn validate_toggle_lifecycle_payload(
    payload: &TelemetryPayload,
) -> Result<(), ToggleLifecycleSchemaError> {
    let allowed = BTreeSet::from(TOGGLE_LIFECYCLE_ALLOWED_KEYS.map(ToString::to_string));
    let keys = payload.keys().cloned().collect::<BTreeSet<_>>();
    if keys != allowed {
        return Err(ToggleLifecycleSchemaError::new(
            ToggleLifecycleSchemaErrorCode::InvalidPayloadKeys,
            "toggle lifecycle payload keys must exactly match allowlist",
        ));
    }

    for key in TOGGLE_LIFECYCLE_ALLOWED_KEYS {
        match payload.get(key) {
            Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
            _ => {
                return Err(ToggleLifecycleSchemaError::new(
                    ToggleLifecycleSchemaErrorCode::MissingRequiredField,
                    format!("toggle lifecycle payload requires non-empty {key}"),
                ))
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        emit_routing_disabled, emit_routing_enabled, validate_toggle_lifecycle_payload,
        ToggleLifecycleSchemaErrorCode,
    };
    use crate::db::RouteSessionCloseRecord;
    use crate::detection::state_resolver::DetectionState;
    use crate::routing::{
        DetectionGateDecision, OffPipelineReasonCode, OffPipelineResult, OffPipelineStatus,
        OnPipelineFailure, OnPipelineReasonCode, OnPipelineResult, OnPipelineStage,
        OnPipelineStatus, RoutingState, SessionHookResult, SessionHookStatus,
    };
    use crate::telemetry::{TelemetryService, TelemetryValue};
    use std::collections::BTreeMap;

    fn blocked_on_result(reason_code: OnPipelineReasonCode) -> OnPipelineResult {
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
                    stage: OnPipelineStage::DetectionGate,
                    reason_code,
                },
            },
        }
    }

    fn ready_on_result() -> OnPipelineResult {
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

    #[test]
    fn routing_enabled_is_emitted_on_successful_on_completion() {
        let mut telemetry = TelemetryService::new();
        let emitted = emit_routing_enabled(&mut telemetry, &ready_on_result())
            .expect("routing_enabled emission should succeed");
        assert!(emitted);

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("routing_enabled event should be queued");
        assert_eq!(event.name, "routing_enabled");
        assert_eq!(
            event.payload.get("result"),
            Some(&TelemetryValue::Text("success".to_string()))
        );
        assert_eq!(
            event.payload.get("lifecycle_state"),
            Some(&TelemetryValue::Text("active".to_string()))
        );
    }

    #[test]
    fn routing_disabled_is_emitted_on_successful_off_completion() {
        let mut telemetry = TelemetryService::new();
        let result = off_result(OffPipelineStatus::Completed, OffPipelineReasonCode::OffRequested);
        let emitted = emit_routing_disabled(&mut telemetry, &result)
            .expect("routing_disabled emission should succeed");
        assert!(emitted);

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("routing_disabled event should be queued");
        assert_eq!(event.name, "routing_disabled");
        assert_eq!(
            event.payload.get("result"),
            Some(&TelemetryValue::Text("success".to_string()))
        );
        assert_eq!(
            event.payload.get("reason_code"),
            Some(&TelemetryValue::Text("off_requested".to_string()))
        );
        assert_eq!(
            event.payload.get("lifecycle_state"),
            Some(&TelemetryValue::Text("idle".to_string()))
        );
    }

    #[test]
    fn failure_payloads_include_normalized_reason_code() {
        let mut telemetry = TelemetryService::new();
        let on_failure = blocked_on_result(OnPipelineReasonCode::ManifestSignatureInvalid);
        emit_routing_enabled(&mut telemetry, &on_failure)
            .expect("failed on-flow should still emit reason-aware event");

        let off_failure = off_result(
            OffPipelineStatus::Failed,
            OffPipelineReasonCode::OffTeardownFailed,
        );
        emit_routing_disabled(&mut telemetry, &off_failure)
            .expect("failed off-flow should still emit reason-aware event");

        let events = telemetry.drain_batch(10);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].payload.get("reason_code"),
            Some(&TelemetryValue::Text(
                "manifest_signature_invalid".to_string()
            ))
        );
        assert_eq!(
            events[1].payload.get("reason_code"),
            Some(&TelemetryValue::Text("off_teardown_failed".to_string()))
        );
        assert_eq!(
            events[1].payload.get("lifecycle_state"),
            Some(&TelemetryValue::Text("error".to_string()))
        );
    }

    #[test]
    fn payload_validation_rejects_non_allowlisted_fields() {
        let payload = BTreeMap::from([
            ("result".to_string(), TelemetryValue::Text("success".to_string())),
            (
                "reason_code".to_string(),
                TelemetryValue::Text("none".to_string()),
            ),
            (
                "lifecycle_state".to_string(),
                TelemetryValue::Text("active".to_string()),
            ),
            (
                "private_host".to_string(),
                TelemetryValue::Text("sensitive.example.net".to_string()),
            ),
        ]);

        let err = validate_toggle_lifecycle_payload(&payload)
            .expect_err("extra field must fail allowlist check");
        assert_eq!(err.code, ToggleLifecycleSchemaErrorCode::InvalidPayloadKeys);
    }
}
