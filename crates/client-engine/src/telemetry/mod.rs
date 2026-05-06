//! Telemetry batching boundary for privacy-safe event delivery.

pub mod batch_queue;
pub mod delivery_client;
pub mod events;
pub mod scrubber;
pub mod validator;

use batch_queue::{BackpressureDiagnostic, BatchQueue, BatchQueueConfig};
use scrubber::{
    scrub_or_reject_payload, SensitiveFieldAction, SensitiveFieldDiagnostic, SensitiveFieldPolicy,
};
use std::collections::BTreeMap;
use validator::{
    validate_event_payload, TelemetryValidationError, TelemetryValidationErrorCode,
    UnknownKeyPolicy,
};

#[derive(Debug, Clone)]
pub enum TelemetryValue {
    Text(String),
    Integer(i64),
    Float(f64),
}

impl PartialEq for TelemetryValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(left), Self::Text(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left.to_bits() == right.to_bits(),
            _ => false,
        }
    }
}

impl Eq for TelemetryValue {}

pub type TelemetryPayload = BTreeMap<String, TelemetryValue>;

/// Lightweight telemetry event placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryEvent {
    pub name: String,
    pub payload: TelemetryPayload,
}

impl TelemetryEvent {
    pub fn new(name: impl Into<String>, payload: TelemetryPayload) -> Self {
        Self {
            name: name.into(),
            payload,
        }
    }
}

/// Public batching interface used by upper orchestration layers.
#[derive(Debug, Clone)]
pub struct TelemetryService {
    batch_queue: BatchQueue,
    sensitive_field_policy: SensitiveFieldPolicy,
    privacy_diagnostics: Vec<SensitiveFieldDiagnostic>,
}

impl TelemetryService {
    pub fn new() -> Self {
        Self::with_policies(SensitiveFieldPolicy::default(), BatchQueueConfig::default())
    }

    pub fn with_sensitive_field_policy(policy: SensitiveFieldPolicy) -> Self {
        Self::with_policies(policy, BatchQueueConfig::default())
    }

    pub fn with_batch_queue_config(config: BatchQueueConfig) -> Self {
        Self::with_policies(SensitiveFieldPolicy::default(), config)
    }

    pub fn with_policies(
        sensitive_field_policy: SensitiveFieldPolicy,
        batch_queue_config: BatchQueueConfig,
    ) -> Self {
        Self {
            batch_queue: BatchQueue::new(batch_queue_config),
            sensitive_field_policy,
            privacy_diagnostics: Vec::new(),
        }
    }

    pub fn queue_depth(&self) -> usize {
        self.batch_queue.depth()
    }

    pub fn privacy_diagnostics(&self) -> &[SensitiveFieldDiagnostic] {
        &self.privacy_diagnostics
    }

    pub fn take_privacy_diagnostics(&mut self) -> Vec<SensitiveFieldDiagnostic> {
        std::mem::take(&mut self.privacy_diagnostics)
    }

    pub fn backpressure_diagnostics(&self) -> &[BackpressureDiagnostic] {
        self.batch_queue.backpressure_diagnostics()
    }

    pub fn take_backpressure_diagnostics(&mut self) -> Vec<BackpressureDiagnostic> {
        self.batch_queue.take_backpressure_diagnostics()
    }

    pub fn set_sensitive_field_policy(&mut self, policy: SensitiveFieldPolicy) {
        self.sensitive_field_policy = policy;
    }

    pub fn enqueue(&mut self, event: TelemetryEvent) {
        match scrub_or_reject_payload(&event.name, &event.payload, self.sensitive_field_policy) {
            Ok(scrubbed) => {
                if !scrubbed.violations.is_empty() {
                    self.record_privacy_violations(
                        &event.name,
                        &scrubbed.violations,
                        SensitiveFieldAction::Sanitized,
                    );
                }
                self.batch_queue
                    .enqueue_event(TelemetryEvent::new(event.name, scrubbed.payload));
            }
            Err(error) => {
                self.record_privacy_violations(
                    &error.event_name,
                    std::slice::from_ref(&error.violation),
                    SensitiveFieldAction::Rejected,
                );
            }
        }
    }

    pub fn enqueue_validated(
        &mut self,
        event_name: impl Into<String>,
        payload: TelemetryPayload,
        unknown_key_policy: UnknownKeyPolicy,
    ) -> Result<(), TelemetryValidationError> {
        let name = event_name.into();
        let allowlisted = validate_event_payload(&name, &payload, unknown_key_policy)?;
        let scrubbed = scrub_or_reject_payload(&name, &allowlisted, self.sensitive_field_policy)
            .map_err(|error| {
                self.record_privacy_violations(
                    &error.event_name,
                    std::slice::from_ref(&error.violation),
                    SensitiveFieldAction::Rejected,
                );
                TelemetryValidationError::new(
                    TelemetryValidationErrorCode::SensitiveFieldViolation,
                    format!(
                        "{}: telemetry event '{}' contains prohibited field '{}' ({})",
                        TelemetryValidationErrorCode::SensitiveFieldViolation.as_str(),
                        error.event_name,
                        error.violation.path,
                        error.violation.code.as_str()
                    ),
                )
            })?;

        if !scrubbed.violations.is_empty() {
            self.record_privacy_violations(
                &name,
                &scrubbed.violations,
                SensitiveFieldAction::Sanitized,
            );
        }

        self.batch_queue
            .enqueue_event(TelemetryEvent::new(name, scrubbed.payload));
        Ok(())
    }

    pub fn drain_batch(&mut self, max_items: usize) -> Vec<TelemetryEvent> {
        self.batch_queue.build_batch(max_items)
    }

    fn record_privacy_violations(
        &mut self,
        event_name: &str,
        violations: &[scrubber::SensitiveFieldViolation],
        action: SensitiveFieldAction,
    ) {
        for violation in violations {
            self.privacy_diagnostics.push(SensitiveFieldDiagnostic {
                event_name: event_name.to_string(),
                field_path: violation.path.clone(),
                code: violation.code,
                action,
            });
        }
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::batch_queue::{BackpressurePolicy, BatchQueueConfig};
    use super::scrubber::{SensitiveFieldAction, SensitiveFieldPolicy};
    use super::validator::UnknownKeyPolicy;
    use super::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};

    #[test]
    fn sanitize_policy_strips_nested_sensitive_fields_before_queue() {
        let mut service = TelemetryService::new();
        let payload = TelemetryPayload::from([
            (
                "game_id".to_string(),
                TelemetryValue::Text("ffxiv".to_string()),
            ),
            (
                "process_name".to_string(),
                TelemetryValue::Text(
                    r#"{"exe":"ffxiv_dx11.exe","username":"player-secret"}"#.to_string(),
                ),
            ),
            (
                "detection_time_ms".to_string(),
                TelemetryValue::Integer(1_700_000_000_000),
            ),
        ]);

        service
            .enqueue_validated("game_detected", payload, UnknownKeyPolicy::Reject)
            .expect("sanitize mode should scrub and continue");

        assert_eq!(service.queue_depth(), 1);
        let event = service
            .drain_batch(1)
            .pop()
            .expect("event should be queued");
        let scrubbed_text = match event.payload.get("process_name") {
            Some(TelemetryValue::Text(value)) => value,
            _ => panic!("process_name should be stored as text"),
        };
        assert!(!scrubbed_text.contains("username"));
        assert!(!scrubbed_text.contains("player-secret"));

        let diagnostics = service.take_privacy_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].event_name, "game_detected");
        assert_eq!(diagnostics[0].field_path, "process_name.username");
        assert_eq!(diagnostics[0].action, SensitiveFieldAction::Sanitized);
    }

    #[test]
    fn reject_policy_blocks_sensitive_payload_with_non_sensitive_error() {
        let mut service =
            TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::reject());
        let payload = TelemetryPayload::from([
            (
                "game_id".to_string(),
                TelemetryValue::Text("ffxiv".to_string()),
            ),
            (
                "process_name".to_string(),
                TelemetryValue::Text(
                    r#"{"exe":"ffxiv_dx11.exe","username":"player-secret"}"#.to_string(),
                ),
            ),
            (
                "detection_time_ms".to_string(),
                TelemetryValue::Integer(1_700_000_000_000),
            ),
        ]);

        let error = service
            .enqueue_validated("game_detected", payload, UnknownKeyPolicy::Reject)
            .expect_err("reject mode should block sensitive field");
        assert_eq!(error.code.as_str(), "sensitive_field_violation");
        assert!(!error.message.contains("player-secret"));
        assert_eq!(service.queue_depth(), 0);

        let diagnostics = service.take_privacy_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].action, SensitiveFieldAction::Rejected);
        assert_eq!(diagnostics[0].field_path, "process_name.username");
    }

    #[test]
    fn direct_enqueue_never_persists_prohibited_fields() {
        let mut service = TelemetryService::new();
        service.enqueue(TelemetryEvent::new(
            "unsafe_event",
            TelemetryPayload::from([
                (
                    "username".to_string(),
                    TelemetryValue::Text("private-name".to_string()),
                ),
                (
                    "reason_code".to_string(),
                    TelemetryValue::Text("safe_generic_issue".to_string()),
                ),
            ]),
        ));

        let event = service
            .drain_batch(1)
            .pop()
            .expect("sanitized event should remain queueable");
        assert!(!event.payload.contains_key("username"));
        assert_eq!(event.payload.len(), 1);

        let diagnostics = service.take_privacy_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].action, SensitiveFieldAction::Sanitized);
        assert_eq!(diagnostics[0].field_path, "username");
    }

    #[test]
    fn queue_backpressure_is_centralized_and_deterministic() {
        let mut service = TelemetryService::with_batch_queue_config(BatchQueueConfig {
            max_queue_depth: 2,
            max_batch_size: 200,
            backpressure_policy: BackpressurePolicy::DropOldest,
        });

        service.enqueue(TelemetryEvent::new("first", Default::default()));
        service.enqueue(TelemetryEvent::new("second", Default::default()));
        service.enqueue(TelemetryEvent::new("third", Default::default()));

        assert_eq!(service.queue_depth(), 2);
        let batch = service.drain_batch(10);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].name, "second");
        assert_eq!(batch[1].name, "third");

        let backpressure_logs = service.take_backpressure_diagnostics();
        assert_eq!(backpressure_logs.len(), 1);
        assert_eq!(backpressure_logs[0].dropped_event_name, "first");
        assert_eq!(backpressure_logs[0].incoming_event_name, "third");
    }

    #[test]
    fn validated_enqueue_remains_non_blocking_under_backpressure() {
        let mut service = TelemetryService::with_batch_queue_config(BatchQueueConfig {
            max_queue_depth: 1,
            max_batch_size: 200,
            backpressure_policy: BackpressurePolicy::DropOldest,
        });

        service
            .enqueue_validated(
                "routing_enabled",
                TelemetryPayload::from([
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
                ]),
                UnknownKeyPolicy::Reject,
            )
            .expect("first event should enqueue");
        service
            .enqueue_validated(
                "routing_enabled",
                TelemetryPayload::from([
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
                ]),
                UnknownKeyPolicy::Reject,
            )
            .expect("queue overflow should not block telemetry producer");

        assert_eq!(service.queue_depth(), 1);
        let diagnostics = service.take_backpressure_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].dropped_event_name, "routing_enabled");
        assert_eq!(diagnostics[0].incoming_event_name, "routing_enabled");
    }
}
