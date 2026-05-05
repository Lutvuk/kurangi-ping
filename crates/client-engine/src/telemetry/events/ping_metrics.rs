use std::collections::{BTreeMap, BTreeSet};

use crate::metrics::PingMetrics;
use crate::telemetry::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};

pub const PING_MEASURED_EVENT_NAME: &str = "ping_measured";
pub const PING_MEASURED_ALLOWED_KEYS: [&str; 4] = [
    "baseline_ping_ms",
    "routed_ping_ms",
    "jitter_ms",
    "packet_loss_pct",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingMeasuredEmissionPolicy {
    pub max_queue_depth: usize,
    pub min_emit_interval_ms: u64,
}

impl Default for PingMeasuredEmissionPolicy {
    fn default() -> Self {
        Self {
            max_queue_depth: 1_000,
            min_emit_interval_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PingMeasuredEmitState {
    pub last_emitted_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingMeasuredEmitStatus {
    Emitted,
    SkippedCadence,
    DroppedBackpressure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PingMeasuredSchemaErrorCode {
    MissingRequiredField,
    InvalidPayloadKeys,
    InvalidPayloadValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingMeasuredSchemaError {
    pub code: PingMeasuredSchemaErrorCode,
    pub message: String,
}

impl PingMeasuredSchemaError {
    fn new(code: PingMeasuredSchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_ping_measured_event(
    telemetry: &mut TelemetryService,
    metrics: &PingMetrics,
    sampled_at_unix_ms: u64,
    emission_state: &mut PingMeasuredEmitState,
    policy: PingMeasuredEmissionPolicy,
) -> Result<PingMeasuredEmitStatus, PingMeasuredSchemaError> {
    if let Some(last_emitted) = emission_state.last_emitted_at_unix_ms {
        let elapsed = sampled_at_unix_ms.saturating_sub(last_emitted);
        if elapsed < policy.min_emit_interval_ms {
            return Ok(PingMeasuredEmitStatus::SkippedCadence);
        }
    }

    if telemetry.queue_depth() >= policy.max_queue_depth {
        return Ok(PingMeasuredEmitStatus::DroppedBackpressure);
    }

    let payload = build_payload(metrics)?;
    validate_ping_measured_payload(&payload)?;
    telemetry.enqueue(TelemetryEvent::new(PING_MEASURED_EVENT_NAME, payload));
    emission_state.last_emitted_at_unix_ms = Some(sampled_at_unix_ms);
    Ok(PingMeasuredEmitStatus::Emitted)
}

fn build_payload(metrics: &PingMetrics) -> Result<TelemetryPayload, PingMeasuredSchemaError> {
    let routed_ping_ms = metrics.routed_ping_ms.ok_or_else(|| {
        PingMeasuredSchemaError::new(
            PingMeasuredSchemaErrorCode::MissingRequiredField,
            "ping_measured requires routed_ping_ms",
        )
    })?;
    let jitter_ms = metrics.jitter_ms.ok_or_else(|| {
        PingMeasuredSchemaError::new(
            PingMeasuredSchemaErrorCode::MissingRequiredField,
            "ping_measured requires jitter_ms",
        )
    })?;
    let packet_loss_pct = metrics.packet_loss_pct.ok_or_else(|| {
        PingMeasuredSchemaError::new(
            PingMeasuredSchemaErrorCode::MissingRequiredField,
            "ping_measured requires packet_loss_pct",
        )
    })?;

    Ok(BTreeMap::from([
        (
            "baseline_ping_ms".to_string(),
            TelemetryValue::Float(metrics.baseline_ping_ms),
        ),
        (
            "routed_ping_ms".to_string(),
            TelemetryValue::Float(routed_ping_ms),
        ),
        ("jitter_ms".to_string(), TelemetryValue::Float(jitter_ms)),
        (
            "packet_loss_pct".to_string(),
            TelemetryValue::Float(packet_loss_pct),
        ),
    ]))
}

pub fn validate_ping_measured_payload(
    payload: &TelemetryPayload,
) -> Result<(), PingMeasuredSchemaError> {
    let allowed = BTreeSet::from(PING_MEASURED_ALLOWED_KEYS.map(ToString::to_string));
    let keys = payload.keys().cloned().collect::<BTreeSet<_>>();
    if keys != allowed {
        return Err(PingMeasuredSchemaError::new(
            PingMeasuredSchemaErrorCode::InvalidPayloadKeys,
            "ping_measured payload keys must exactly match allowlist",
        ));
    }

    validate_float_field(payload, "baseline_ping_ms", 0.0, None)?;
    validate_float_field(payload, "routed_ping_ms", 0.0, None)?;
    validate_float_field(payload, "jitter_ms", 0.0, None)?;
    validate_float_field(payload, "packet_loss_pct", 0.0, Some(100.0))?;

    Ok(())
}

fn validate_float_field(
    payload: &TelemetryPayload,
    key: &str,
    min: f64,
    max: Option<f64>,
) -> Result<(), PingMeasuredSchemaError> {
    let value = match payload.get(key) {
        Some(TelemetryValue::Float(value)) => *value,
        _ => {
            return Err(PingMeasuredSchemaError::new(
                PingMeasuredSchemaErrorCode::MissingRequiredField,
                format!("ping_measured payload requires numeric {key}"),
            ))
        }
    };

    if !value.is_finite() || value < min {
        return Err(PingMeasuredSchemaError::new(
            PingMeasuredSchemaErrorCode::InvalidPayloadValue,
            format!("ping_measured payload field {key} must be finite and >= {min}"),
        ));
    }

    if let Some(max_bound) = max {
        if value > max_bound {
            return Err(PingMeasuredSchemaError::new(
                PingMeasuredSchemaErrorCode::InvalidPayloadValue,
                format!("ping_measured payload field {key} must be <= {max_bound}"),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        emit_ping_measured_event, validate_ping_measured_payload, PingMeasuredEmitState,
        PingMeasuredEmissionPolicy, PingMeasuredEmitStatus, PingMeasuredSchemaErrorCode,
    };
    use crate::metrics::PingMetrics;
    use crate::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};
    use std::collections::BTreeMap;

    fn sample_metrics() -> PingMetrics {
        PingMetrics {
            baseline_ping_ms: 212.5,
            routed_ping_ms: Some(154.2),
            jitter_ms: Some(4.1),
            packet_loss_pct: Some(0.0),
            sample_count: 6,
        }
    }

    #[test]
    fn payload_contains_only_allowlisted_ping_fields() {
        let mut telemetry = TelemetryService::new();
        let mut state = PingMeasuredEmitState::default();

        let status = emit_ping_measured_event(
            &mut telemetry,
            &sample_metrics(),
            1_700_000_000_000,
            &mut state,
            PingMeasuredEmissionPolicy::default(),
        )
        .expect("valid metrics should emit");

        assert_eq!(status, PingMeasuredEmitStatus::Emitted);
        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("ping_measured event should be queued");
        assert_eq!(event.name, "ping_measured");
        assert_eq!(event.payload.len(), 4);
        assert!(event.payload.contains_key("baseline_ping_ms"));
        assert!(event.payload.contains_key("routed_ping_ms"));
        assert!(event.payload.contains_key("jitter_ms"));
        assert!(event.payload.contains_key("packet_loss_pct"));
    }

    #[test]
    fn invalid_payload_keys_are_rejected_before_queueing() {
        let payload = BTreeMap::from([
            ("baseline_ping_ms".to_string(), TelemetryValue::Float(210.0)),
            ("routed_ping_ms".to_string(), TelemetryValue::Float(155.0)),
            ("jitter_ms".to_string(), TelemetryValue::Float(4.0)),
            ("packet_loss_pct".to_string(), TelemetryValue::Float(1.0)),
            ("relay_id".to_string(), TelemetryValue::Text("sin-01".to_string())),
        ]);

        let error = validate_ping_measured_payload(&payload).expect_err("extra key must fail");
        assert_eq!(error.code, PingMeasuredSchemaErrorCode::InvalidPayloadKeys);
    }

    #[test]
    fn emit_cadence_follows_measurement_loop_policy() {
        let mut telemetry = TelemetryService::new();
        let mut state = PingMeasuredEmitState::default();
        let policy = PingMeasuredEmissionPolicy {
            max_queue_depth: 50,
            min_emit_interval_ms: 1_000,
        };

        let first = emit_ping_measured_event(
            &mut telemetry,
            &sample_metrics(),
            10_000,
            &mut state,
            policy,
        )
        .expect("first emission should pass");
        assert_eq!(first, PingMeasuredEmitStatus::Emitted);

        let second = emit_ping_measured_event(
            &mut telemetry,
            &sample_metrics(),
            10_500,
            &mut state,
            policy,
        )
        .expect("cadence skip should not error");
        assert_eq!(second, PingMeasuredEmitStatus::SkippedCadence);

        let third = emit_ping_measured_event(
            &mut telemetry,
            &sample_metrics(),
            11_000,
            &mut state,
            policy,
        )
        .expect("interval boundary should emit");
        assert_eq!(third, PingMeasuredEmitStatus::Emitted);
        assert_eq!(telemetry.queue_depth(), 2);
    }

    #[test]
    fn queue_backpressure_does_not_block_probe_pipeline() {
        let mut telemetry = TelemetryService::new();
        let mut state = PingMeasuredEmitState::default();
        telemetry.enqueue(TelemetryEvent::new("existing_event", Default::default()));

        let status = emit_ping_measured_event(
            &mut telemetry,
            &sample_metrics(),
            20_000,
            &mut state,
            PingMeasuredEmissionPolicy {
                max_queue_depth: 1,
                min_emit_interval_ms: 0,
            },
        )
        .expect("backpressure should be non-fatal");

        assert_eq!(status, PingMeasuredEmitStatus::DroppedBackpressure);
        assert_eq!(telemetry.queue_depth(), 1);
    }

    #[test]
    fn missing_required_numeric_fields_fail_schema_guard() {
        let mut telemetry = TelemetryService::new();
        let mut state = PingMeasuredEmitState::default();
        let mut metrics = sample_metrics();
        metrics.routed_ping_ms = None;

        let error = emit_ping_measured_event(
            &mut telemetry,
            &metrics,
            30_000,
            &mut state,
            PingMeasuredEmissionPolicy::default(),
        )
        .expect_err("missing routed ping must fail before enqueue");

        assert_eq!(error.code, PingMeasuredSchemaErrorCode::MissingRequiredField);
        assert_eq!(telemetry.queue_depth(), 0);
    }
}
