use std::collections::{BTreeMap, BTreeSet};

use super::TelemetryPayload;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownKeyPolicy {
    Reject,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryValidationErrorCode {
    UnknownEventName,
    UnknownPayloadKey,
    MissingRequiredKey,
    SensitiveFieldViolation,
}

impl TelemetryValidationErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownEventName => "unknown_event_name",
            Self::UnknownPayloadKey => "unknown_payload_key",
            Self::MissingRequiredKey => "missing_required_key",
            Self::SensitiveFieldViolation => "sensitive_field_violation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryValidationError {
    pub code: TelemetryValidationErrorCode,
    pub message: String,
}

impl TelemetryValidationError {
    pub(crate) fn new(code: TelemetryValidationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct EventSchema {
    allowed_keys: BTreeSet<&'static str>,
    required_keys: BTreeSet<&'static str>,
}

#[derive(Debug, Clone)]
pub struct SchemaAllowlist {
    by_event: BTreeMap<&'static str, EventSchema>,
}

impl SchemaAllowlist {
    pub fn default_v1() -> Self {
        let mut by_event = BTreeMap::new();
        by_event.insert("app_opened", schema_with_required(&[], &[]));
        by_event.insert(
            "game_detected",
            schema_with_required(
                &["game_id", "process_name", "detection_time_ms"],
                &["game_id", "process_name", "detection_time_ms"],
            ),
        );
        by_event.insert(
            "routing_enabled",
            schema_with_required(
                &["result", "reason_code", "lifecycle_state"],
                &["result", "reason_code", "lifecycle_state"],
            ),
        );
        by_event.insert(
            "ping_measured",
            schema_with_required(
                &[
                    "baseline_ping_ms",
                    "routed_ping_ms",
                    "jitter_ms",
                    "packet_loss_pct",
                ],
                &[
                    "baseline_ping_ms",
                    "routed_ping_ms",
                    "jitter_ms",
                    "packet_loss_pct",
                ],
            ),
        );
        by_event.insert(
            "routing_disabled",
            schema_with_required(
                &["result", "reason_code", "lifecycle_state"],
                &["result", "reason_code", "lifecycle_state"],
            ),
        );
        by_event.insert("crash_reported", schema_with_required(&[], &[]));
        by_event.insert(
            "relay_failed",
            schema_with_required(
                &[
                    "reason_code",
                    "previous_relay_id",
                    "next_relay_id",
                    "failover_state",
                    "attempt_count",
                ],
                &[
                    "reason_code",
                    "previous_relay_id",
                    "next_relay_id",
                    "failover_state",
                    "attempt_count",
                ],
            ),
        );
        by_event.insert(
            "relay_recovered",
            schema_with_required(
                &[
                    "reason_code",
                    "previous_relay_id",
                    "next_relay_id",
                    "failover_state",
                    "attempt_count",
                ],
                &[
                    "reason_code",
                    "previous_relay_id",
                    "next_relay_id",
                    "failover_state",
                    "attempt_count",
                ],
            ),
        );
        by_event.insert("onboarding_completed", schema_with_required(&[], &[]));

        Self { by_event }
    }
}

fn schema_with_required(allowed: &[&'static str], required: &[&'static str]) -> EventSchema {
    EventSchema {
        allowed_keys: allowed.iter().copied().collect::<BTreeSet<_>>(),
        required_keys: required.iter().copied().collect::<BTreeSet<_>>(),
    }
}

pub fn validate_event_payload(
    event_name: &str,
    payload: &TelemetryPayload,
    policy: UnknownKeyPolicy,
) -> Result<TelemetryPayload, TelemetryValidationError> {
    let allowlist = SchemaAllowlist::default_v1();
    let schema = allowlist.by_event.get(event_name).ok_or_else(|| {
        TelemetryValidationError::new(
            TelemetryValidationErrorCode::UnknownEventName,
            format!(
                "{}: telemetry event '{}' is not in allowlist",
                TelemetryValidationErrorCode::UnknownEventName.as_str(),
                event_name
            ),
        )
    })?;

    let mut unknown = payload
        .keys()
        .filter(|key| !schema.allowed_keys.contains(key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();

    if !unknown.is_empty() && matches!(policy, UnknownKeyPolicy::Reject) {
        return Err(TelemetryValidationError::new(
            TelemetryValidationErrorCode::UnknownPayloadKey,
            format!(
                "{}: telemetry event '{}' contains unsupported key '{}'",
                TelemetryValidationErrorCode::UnknownPayloadKey.as_str(),
                event_name,
                unknown[0]
            ),
        ));
    }

    let sanitized = if unknown.is_empty() {
        payload.clone()
    } else {
        payload
            .iter()
            .filter(|(key, _)| schema.allowed_keys.contains(key.as_str()))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<TelemetryPayload>()
    };

    for required_key in &schema.required_keys {
        if !sanitized.contains_key(*required_key) {
            return Err(TelemetryValidationError::new(
                TelemetryValidationErrorCode::MissingRequiredKey,
                format!(
                    "{}: telemetry event '{}' is missing required key '{}'",
                    TelemetryValidationErrorCode::MissingRequiredKey.as_str(),
                    event_name,
                    required_key
                ),
            ));
        }
    }

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::{validate_event_payload, UnknownKeyPolicy};
    use crate::telemetry::{TelemetryPayload, TelemetryValue};

    #[test]
    fn allowlist_enforces_event_specific_keys() {
        let payload = TelemetryPayload::from([
            ("baseline_ping_ms".to_string(), TelemetryValue::Float(210.0)),
            ("routed_ping_ms".to_string(), TelemetryValue::Float(160.0)),
            ("jitter_ms".to_string(), TelemetryValue::Float(5.0)),
            ("packet_loss_pct".to_string(), TelemetryValue::Float(0.0)),
            (
                "relay_id".to_string(),
                TelemetryValue::Text("sin-01".to_string()),
            ),
        ]);

        let error = validate_event_payload("ping_measured", &payload, UnknownKeyPolicy::Reject)
            .expect_err("unknown key should be rejected");
        assert_eq!(error.code.as_str(), "unknown_payload_key");
    }

    #[test]
    fn unknown_keys_can_be_dropped_under_policy_mode() {
        let payload = TelemetryPayload::from([
            (
                "result".to_string(),
                TelemetryValue::Text("success".to_string()),
            ),
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
                TelemetryValue::Text("internal.example.net".to_string()),
            ),
        ]);

        let sanitized = validate_event_payload("routing_enabled", &payload, UnknownKeyPolicy::Drop)
            .expect("unknown key should be dropped");
        assert_eq!(sanitized.len(), 3);
        assert!(!sanitized.contains_key("private_host"));
    }

    #[test]
    fn validation_failures_return_deterministic_reason_codes() {
        let error = validate_event_payload(
            "unknown_event",
            &TelemetryPayload::new(),
            UnknownKeyPolicy::Reject,
        )
        .expect_err("unknown event should fail deterministically");
        assert_eq!(error.code.as_str(), "unknown_event_name");

        let missing = validate_event_payload(
            "relay_failed",
            &TelemetryPayload::from([(
                "reason_code".to_string(),
                TelemetryValue::Text("dead_relay_detected".to_string()),
            )]),
            UnknownKeyPolicy::Reject,
        )
        .expect_err("missing required keys should fail deterministically");
        assert_eq!(missing.code.as_str(), "missing_required_key");
    }
}
