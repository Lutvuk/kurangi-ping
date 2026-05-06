//! Telemetry batching boundary for privacy-safe event delivery.

pub mod events;
pub mod scrubber;
pub mod validator;

use std::collections::BTreeMap;
use scrubber::{
    scrub_or_reject_payload, SensitiveFieldAction, SensitiveFieldDiagnostic,
    SensitiveFieldPolicy,
};
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
    queue: Vec<TelemetryEvent>,
    sensitive_field_policy: SensitiveFieldPolicy,
    privacy_diagnostics: Vec<SensitiveFieldDiagnostic>,
}

impl TelemetryService {
    pub fn new() -> Self {
        Self::with_sensitive_field_policy(SensitiveFieldPolicy::default())
    }

    pub fn with_sensitive_field_policy(policy: SensitiveFieldPolicy) -> Self {
        Self {
            queue: Vec::new(),
            sensitive_field_policy: policy,
            privacy_diagnostics: Vec::new(),
        }
    }

    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    pub fn privacy_diagnostics(&self) -> &[SensitiveFieldDiagnostic] {
        &self.privacy_diagnostics
    }

    pub fn take_privacy_diagnostics(&mut self) -> Vec<SensitiveFieldDiagnostic> {
        std::mem::take(&mut self.privacy_diagnostics)
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
                self.queue.push(TelemetryEvent::new(event.name, scrubbed.payload));
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
            self.record_privacy_violations(&name, &scrubbed.violations, SensitiveFieldAction::Sanitized);
        }

        self.queue.push(TelemetryEvent::new(name, scrubbed.payload));
        Ok(())
    }

    pub fn drain_batch(&mut self, max_items: usize) -> Vec<TelemetryEvent> {
        // TODO(KP-083/KP-084): replace with retry-aware batch lifecycle.
        let take = max_items.min(self.queue.len());
        self.queue.drain(0..take).collect()
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
    use super::scrubber::{SensitiveFieldAction, SensitiveFieldPolicy};
    use super::validator::UnknownKeyPolicy;
    use super::{TelemetryEvent, TelemetryPayload, TelemetryService, TelemetryValue};

    #[test]
    fn sanitize_policy_strips_nested_sensitive_fields_before_queue() {
        let mut service = TelemetryService::new();
        let payload = TelemetryPayload::from([
            ("game_id".to_string(), TelemetryValue::Text("ffxiv".to_string())),
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
        let mut service = TelemetryService::with_sensitive_field_policy(SensitiveFieldPolicy::reject());
        let payload = TelemetryPayload::from([
            ("game_id".to_string(), TelemetryValue::Text("ffxiv".to_string())),
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
}
