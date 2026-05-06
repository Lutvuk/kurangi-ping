use std::collections::BTreeMap;

use crate::routing::{normalize_reason_code, FailoverStatePayload};
use crate::telemetry::{TelemetryPayload, TelemetryService, TelemetryValue};
use crate::telemetry::validator::{
    validate_event_payload, TelemetryValidationErrorCode, UnknownKeyPolicy,
};

pub const RELAY_FAILED_EVENT_NAME: &str = "relay_failed";
pub const RELAY_RECOVERED_EVENT_NAME: &str = "relay_recovered";
pub const RELAY_FAILOVER_ALLOWED_KEYS: [&str; 5] = [
    "reason_code",
    "previous_relay_id",
    "next_relay_id",
    "failover_state",
    "attempt_count",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayFailoverEmissionPolicy {
    pub max_queue_depth: usize,
}

impl Default for RelayFailoverEmissionPolicy {
    fn default() -> Self {
        Self {
            max_queue_depth: 1_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayFailoverEmitStatus {
    Emitted,
    SkippedStateMismatch,
    DroppedBackpressure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayFailoverSchemaErrorCode {
    MissingRequiredField,
    InvalidPayloadKeys,
    InvalidPayloadValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayFailoverSchemaError {
    pub code: RelayFailoverSchemaErrorCode,
    pub message: String,
}

impl RelayFailoverSchemaError {
    fn new(code: RelayFailoverSchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_relay_failed(
    telemetry: &mut TelemetryService,
    state_payload: &FailoverStatePayload,
    attempt_count: usize,
    policy: RelayFailoverEmissionPolicy,
) -> Result<RelayFailoverEmitStatus, RelayFailoverSchemaError> {
    if !matches!(state_payload.current_state.as_str(), "switching" | "failed") {
        return Ok(RelayFailoverEmitStatus::SkippedStateMismatch);
    }

    enqueue_failover_event(
        telemetry,
        RELAY_FAILED_EVENT_NAME,
        state_payload,
        attempt_count,
        policy,
    )
}

pub fn emit_relay_recovered(
    telemetry: &mut TelemetryService,
    state_payload: &FailoverStatePayload,
    attempt_count: usize,
    policy: RelayFailoverEmissionPolicy,
) -> Result<RelayFailoverEmitStatus, RelayFailoverSchemaError> {
    if state_payload.current_state != "recovered" {
        return Ok(RelayFailoverEmitStatus::SkippedStateMismatch);
    }

    enqueue_failover_event(
        telemetry,
        RELAY_RECOVERED_EVENT_NAME,
        state_payload,
        attempt_count,
        policy,
    )
}

fn enqueue_failover_event(
    telemetry: &mut TelemetryService,
    event_name: &str,
    state_payload: &FailoverStatePayload,
    attempt_count: usize,
    policy: RelayFailoverEmissionPolicy,
) -> Result<RelayFailoverEmitStatus, RelayFailoverSchemaError> {
    if telemetry.queue_depth() >= policy.max_queue_depth {
        return Ok(RelayFailoverEmitStatus::DroppedBackpressure);
    }

    let payload = build_payload(state_payload, attempt_count)?;
    validate_failover_payload(&payload)?;
    telemetry
        .enqueue_validated(event_name, payload, UnknownKeyPolicy::Reject)
        .map_err(map_validator_error)?;
    Ok(RelayFailoverEmitStatus::Emitted)
}

fn build_payload(
    state_payload: &FailoverStatePayload,
    attempt_count: usize,
) -> Result<TelemetryPayload, RelayFailoverSchemaError> {
    let attempts = i64::try_from(attempt_count).map_err(|_| {
        RelayFailoverSchemaError::new(
            RelayFailoverSchemaErrorCode::InvalidPayloadValue,
            "attempt_count exceeds supported integer range",
        )
    })?;

    let reason_code = normalize_reason_code(Some(state_payload.reason_code.as_str()))
        .as_str()
        .to_string();
    let previous_relay_id = sanitize_relay_reference(state_payload.previous_relay_id.as_deref());
    let next_relay_id = sanitize_relay_reference(state_payload.next_relay_id.as_deref());
    let failover_state = sanitize_failover_state(&state_payload.current_state);

    Ok(BTreeMap::from([
        ("reason_code".to_string(), TelemetryValue::Text(reason_code)),
        (
            "previous_relay_id".to_string(),
            TelemetryValue::Text(previous_relay_id),
        ),
        (
            "next_relay_id".to_string(),
            TelemetryValue::Text(next_relay_id),
        ),
        (
            "failover_state".to_string(),
            TelemetryValue::Text(failover_state),
        ),
        ("attempt_count".to_string(), TelemetryValue::Integer(attempts)),
    ]))
}

fn sanitize_relay_reference(value: Option<&str>) -> String {
    let Some(raw) = value else {
        return "none".to_string();
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "none".to_string();
    }

    if trimmed.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') {
        trimmed.to_string()
    } else {
        "unknown_relay".to_string()
    }
}

fn sanitize_failover_state(value: &str) -> String {
    match value {
        "stable" | "switching" | "recovered" | "failed" => value.to_string(),
        _ => "failed".to_string(),
    }
}

pub fn validate_failover_payload(
    payload: &TelemetryPayload,
) -> Result<(), RelayFailoverSchemaError> {
    validate_event_payload(RELAY_FAILED_EVENT_NAME, payload, UnknownKeyPolicy::Reject)
        .map_err(map_validator_error)?;

    for key in [
        "reason_code",
        "previous_relay_id",
        "next_relay_id",
        "failover_state",
    ] {
        match payload.get(key) {
            Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
            _ => {
                return Err(RelayFailoverSchemaError::new(
                    RelayFailoverSchemaErrorCode::MissingRequiredField,
                    format!("relay failover payload requires non-empty {key}"),
                ))
            }
        }
    }

    match payload.get("attempt_count") {
        Some(TelemetryValue::Integer(value)) if *value >= 0 => {}
        _ => {
            return Err(RelayFailoverSchemaError::new(
                RelayFailoverSchemaErrorCode::MissingRequiredField,
                "relay failover payload requires non-negative attempt_count",
            ))
        }
    }

    Ok(())
}

fn map_validator_error(
    error: crate::telemetry::validator::TelemetryValidationError,
) -> RelayFailoverSchemaError {
    let code = match error.code {
        TelemetryValidationErrorCode::UnknownEventName
        | TelemetryValidationErrorCode::UnknownPayloadKey => {
            RelayFailoverSchemaErrorCode::InvalidPayloadKeys
        }
        TelemetryValidationErrorCode::MissingRequiredKey => {
            RelayFailoverSchemaErrorCode::MissingRequiredField
        }
    };
    RelayFailoverSchemaError::new(code, error.message)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::routing::{build_failover_state_payload, FailoverUiState};
    use crate::telemetry::events::relay_failover::{
        emit_relay_failed, emit_relay_recovered, RelayFailoverEmissionPolicy,
        RelayFailoverEmitStatus,
    };
    use crate::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};

    #[test]
    fn relay_failed_emits_on_terminal_and_transition_failures() {
        let mut telemetry = TelemetryService::new();
        let policy = RelayFailoverEmissionPolicy::default();
        let switching = build_failover_state_payload(
            FailoverUiState::Switching,
            Some("sin-01"),
            Some("nrt-01"),
            Some("dead_relay_detected"),
        );
        let failed = build_failover_state_payload(
            FailoverUiState::Failed,
            Some("sin-01"),
            None,
            Some("FAILOVER_RETRY_BUDGET_EXHAUSTED"),
        );

        let s1 = emit_relay_failed(&mut telemetry, &switching, 1, policy)
            .expect("switching failure should emit");
        let s2 = emit_relay_failed(&mut telemetry, &failed, 2, policy)
            .expect("terminal failure should emit");

        assert_eq!(s1, RelayFailoverEmitStatus::Emitted);
        assert_eq!(s2, RelayFailoverEmitStatus::Emitted);
        let events = telemetry.drain_batch(10);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| event.name == "relay_failed"));
    }

    #[test]
    fn recovery_event_emits_on_successful_post_failover_stabilization() {
        let mut telemetry = TelemetryService::new();
        let payload = build_failover_state_payload(
            FailoverUiState::Recovered,
            Some("sin-01"),
            Some("nrt-01"),
            Some("switch_successful"),
        );

        let status = emit_relay_recovered(
            &mut telemetry,
            &payload,
            3,
            RelayFailoverEmissionPolicy::default(),
        )
        .expect("recovery should emit");
        assert_eq!(status, RelayFailoverEmitStatus::Emitted);

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("relay_recovered event should exist");
        assert_eq!(event.name, "relay_recovered");
        assert_eq!(
            event.payload.get("reason_code"),
            Some(&TelemetryValue::Text("switch_successful".to_string()))
        );
    }

    #[test]
    fn payloads_respect_allowlist_and_avoid_sensitive_data() {
        let mut telemetry = TelemetryService::new();
        let payload = build_failover_state_payload(
            FailoverUiState::Failed,
            Some("sensitive-host.internal.example"),
            Some("nrt-01"),
            Some("unknown-private-reason"),
        );

        emit_relay_failed(
            &mut telemetry,
            &payload,
            1,
            RelayFailoverEmissionPolicy::default(),
        )
        .expect("payload should be normalized and emitted safely");

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("relay_failed event should exist");
        let keys = event.payload.keys().cloned().collect::<BTreeSet<_>>();
        let expected_keys = BTreeSet::from(
            [
                "reason_code",
                "previous_relay_id",
                "next_relay_id",
                "failover_state",
                "attempt_count",
            ]
            .map(ToString::to_string),
        );
        assert_eq!(keys, expected_keys);
        assert_eq!(
            event.payload.get("reason_code"),
            Some(&TelemetryValue::Text("safe_generic_issue".to_string()))
        );
        assert_eq!(
            event.payload.get("previous_relay_id"),
            Some(&TelemetryValue::Text("unknown_relay".to_string()))
        );
    }

    #[test]
    fn emission_path_tolerates_queue_backpressure_safely() {
        let mut telemetry = TelemetryService::new();
        telemetry.enqueue(TelemetryEvent::new("existing_event", Default::default()));
        let payload = build_failover_state_payload(
            FailoverUiState::Failed,
            Some("sin-01"),
            None,
            Some("dead_relay_detected"),
        );

        let status = emit_relay_failed(
            &mut telemetry,
            &payload,
            1,
            RelayFailoverEmissionPolicy { max_queue_depth: 1 },
        )
        .expect("backpressure must not crash emission path");

        assert_eq!(status, RelayFailoverEmitStatus::DroppedBackpressure);
        assert_eq!(telemetry.queue_depth(), 1);
    }
}
