use std::collections::BTreeMap;

use crate::onboarding::{
    OnboardingCompletionDecision, OnboardingCompletionOutcome, OnboardingLifecycleState,
    OnboardingStep,
};
use crate::telemetry::validator::{
    validate_event_payload, TelemetryValidationErrorCode, UnknownKeyPolicy,
};
use crate::telemetry::{TelemetryPayload, TelemetryService, TelemetryValue};

pub const ONBOARDING_COMPLETED_EVENT_NAME: &str = "onboarding_completed";
pub const ONBOARDING_COMPLETED_ALLOWED_KEYS: [&str; 3] =
    ["onboarding_duration_s", "game_id", "success_path"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingCompletedEmitStatus {
    Emitted,
    SkippedCompletionGate,
    SkippedDuplicateCycle,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OnboardingCompletedEmissionTracker {
    pub last_emitted_completion_cycle_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnboardingCompletedSchemaErrorCode {
    MissingRequiredField,
    InvalidPayloadKeys,
    InvalidPayloadValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingCompletedSchemaError {
    pub code: OnboardingCompletedSchemaErrorCode,
    pub message: String,
}

impl OnboardingCompletedSchemaError {
    fn new(code: OnboardingCompletedSchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_onboarding_completed(
    telemetry: &mut TelemetryService,
    decision: &OnboardingCompletionDecision,
    completion_cycle_id: &str,
    onboarding_duration_s: u64,
    game_id: &str,
    tracker: &mut OnboardingCompletedEmissionTracker,
) -> Result<OnboardingCompletedEmitStatus, OnboardingCompletedSchemaError> {
    if !(decision.should_emit_completed_event
        && matches!(decision.outcome, OnboardingCompletionOutcome::Completed))
    {
        return Ok(OnboardingCompletedEmitStatus::SkippedCompletionGate);
    }

    let cycle_id = completion_cycle_id.trim();
    if cycle_id.is_empty() {
        return Err(OnboardingCompletedSchemaError::new(
            OnboardingCompletedSchemaErrorCode::MissingRequiredField,
            "onboarding_completed requires non-empty completion_cycle_id",
        ));
    }

    if tracker.last_emitted_completion_cycle_id.as_deref() == Some(cycle_id) {
        return Ok(OnboardingCompletedEmitStatus::SkippedDuplicateCycle);
    }

    let payload = build_payload(decision, onboarding_duration_s, game_id)?;
    validate_onboarding_completed_payload(&payload)?;
    telemetry
        .enqueue_validated(
            ONBOARDING_COMPLETED_EVENT_NAME,
            payload,
            UnknownKeyPolicy::Reject,
        )
        .map_err(map_validator_error)?;

    tracker.last_emitted_completion_cycle_id = Some(cycle_id.to_string());
    Ok(OnboardingCompletedEmitStatus::Emitted)
}

fn build_payload(
    decision: &OnboardingCompletionDecision,
    onboarding_duration_s: u64,
    game_id: &str,
) -> Result<TelemetryPayload, OnboardingCompletedSchemaError> {
    let duration = i64::try_from(onboarding_duration_s).map_err(|_| {
        OnboardingCompletedSchemaError::new(
            OnboardingCompletedSchemaErrorCode::InvalidPayloadValue,
            "onboarding_duration_s exceeds supported integer range",
        )
    })?;
    let success_path = summarize_success_path(&decision.next_state_machine.state)?;

    Ok(BTreeMap::from([
        (
            "onboarding_duration_s".to_string(),
            TelemetryValue::Integer(duration),
        ),
        (
            "game_id".to_string(),
            TelemetryValue::Text(sanitize_game_id(game_id)),
        ),
        (
            "success_path".to_string(),
            TelemetryValue::Text(success_path),
        ),
    ]))
}

fn summarize_success_path(
    state: &OnboardingLifecycleState,
) -> Result<String, OnboardingCompletedSchemaError> {
    let OnboardingLifecycleState::Completed { completed_steps } = state else {
        return Err(OnboardingCompletedSchemaError::new(
            OnboardingCompletedSchemaErrorCode::InvalidPayloadValue,
            "success_path can only be generated from completed onboarding state",
        ));
    };

    if completed_steps.is_empty() {
        return Err(OnboardingCompletedSchemaError::new(
            OnboardingCompletedSchemaErrorCode::MissingRequiredField,
            "success_path requires at least one completed step",
        ));
    }

    Ok(completed_steps
        .iter()
        .map(|step| step_code(*step))
        .collect::<Vec<_>>()
        .join(">"))
}

fn step_code(step: OnboardingStep) -> &'static str {
    match step {
        OnboardingStep::Welcome => "welcome",
        OnboardingStep::PermissionCheck => "permission_check",
        OnboardingStep::RelayTest => "relay_test",
        OnboardingStep::GameDetectionTest => "game_detection_test",
        OnboardingStep::FirstConnect => "first_connect",
    }
}

fn sanitize_game_id(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "unknown_game".to_string();
    }

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
    {
        normalized
    } else {
        "unknown_game".to_string()
    }
}

pub fn validate_onboarding_completed_payload(
    payload: &TelemetryPayload,
) -> Result<(), OnboardingCompletedSchemaError> {
    validate_event_payload(
        ONBOARDING_COMPLETED_EVENT_NAME,
        payload,
        UnknownKeyPolicy::Reject,
    )
    .map_err(map_validator_error)?;

    match payload.get("onboarding_duration_s") {
        Some(TelemetryValue::Integer(value)) if *value >= 0 => {}
        _ => {
            return Err(OnboardingCompletedSchemaError::new(
                OnboardingCompletedSchemaErrorCode::MissingRequiredField,
                "onboarding_completed payload requires non-negative onboarding_duration_s",
            ))
        }
    }

    match payload.get("game_id") {
        Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
        _ => {
            return Err(OnboardingCompletedSchemaError::new(
                OnboardingCompletedSchemaErrorCode::MissingRequiredField,
                "onboarding_completed payload requires non-empty game_id",
            ))
        }
    }

    match payload.get("success_path") {
        Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
        _ => {
            return Err(OnboardingCompletedSchemaError::new(
                OnboardingCompletedSchemaErrorCode::MissingRequiredField,
                "onboarding_completed payload requires non-empty success_path",
            ))
        }
    }

    Ok(())
}

fn map_validator_error(
    error: crate::telemetry::validator::TelemetryValidationError,
) -> OnboardingCompletedSchemaError {
    let code = match error.code {
        TelemetryValidationErrorCode::UnknownEventName
        | TelemetryValidationErrorCode::UnknownPayloadKey
        | TelemetryValidationErrorCode::SensitiveFieldViolation => {
            OnboardingCompletedSchemaErrorCode::InvalidPayloadKeys
        }
        TelemetryValidationErrorCode::MissingRequiredKey => {
            OnboardingCompletedSchemaErrorCode::MissingRequiredField
        }
    };
    OnboardingCompletedSchemaError::new(code, error.message)
}

#[cfg(test)]
mod tests {
    use super::{
        emit_onboarding_completed, validate_onboarding_completed_payload,
        OnboardingCompletedEmissionTracker, OnboardingCompletedEmitStatus,
        OnboardingCompletedSchemaErrorCode,
    };
    use crate::onboarding::{
        OnboardingCompletionDecision, OnboardingCompletionOutcome, OnboardingCompletionReasonCode,
        OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
    };
    use crate::telemetry::{TelemetryService, TelemetryValue};
    use std::collections::BTreeMap;

    fn completed_decision() -> OnboardingCompletionDecision {
        OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Completed,
            reason_code: OnboardingCompletionReasonCode::CompletedOnFirstSuccessfulConnect,
            next_state_machine: OnboardingStateMachine {
                state: OnboardingLifecycleState::Completed {
                    completed_steps: vec![
                        OnboardingStep::Welcome,
                        OnboardingStep::PermissionCheck,
                        OnboardingStep::RelayTest,
                        OnboardingStep::GameDetectionTest,
                        OnboardingStep::FirstConnect,
                    ],
                },
            },
            should_emit_completed_event: true,
        }
    }

    fn pending_decision() -> OnboardingCompletionDecision {
        OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Pending,
            reason_code: OnboardingCompletionReasonCode::AwaitingSuccessfulConnect,
            next_state_machine: OnboardingStateMachine::new(),
            should_emit_completed_event: false,
        }
    }

    #[test]
    fn emits_only_when_completion_gate_succeeds() {
        let mut telemetry = TelemetryService::new();
        let mut tracker = OnboardingCompletedEmissionTracker::default();

        let skipped = emit_onboarding_completed(
            &mut telemetry,
            &pending_decision(),
            "cycle-001",
            10,
            "ffxiv",
            &mut tracker,
        )
        .expect("pending gate should skip cleanly");
        assert_eq!(
            skipped,
            OnboardingCompletedEmitStatus::SkippedCompletionGate
        );
        assert_eq!(telemetry.queue_depth(), 0);

        let emitted = emit_onboarding_completed(
            &mut telemetry,
            &completed_decision(),
            "cycle-001",
            12,
            "ffxiv",
            &mut tracker,
        )
        .expect("successful gate should emit");
        assert_eq!(emitted, OnboardingCompletedEmitStatus::Emitted);
        assert_eq!(telemetry.queue_depth(), 1);
    }

    #[test]
    fn payload_contains_duration_and_success_path_summary() {
        let mut telemetry = TelemetryService::new();
        let mut tracker = OnboardingCompletedEmissionTracker::default();
        emit_onboarding_completed(
            &mut telemetry,
            &completed_decision(),
            "cycle-002",
            37,
            "ffxiv",
            &mut tracker,
        )
        .expect("completed onboarding should emit");

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("onboarding event should be queued");
        assert_eq!(event.name, "onboarding_completed");
        assert_eq!(event.payload.len(), 3);
        assert_eq!(
            event.payload.get("onboarding_duration_s"),
            Some(&TelemetryValue::Integer(37))
        );
        assert_eq!(
            event.payload.get("game_id"),
            Some(&TelemetryValue::Text("ffxiv".to_string()))
        );
        assert_eq!(
            event.payload.get("success_path"),
            Some(&TelemetryValue::Text(
                "welcome>permission_check>relay_test>game_detection_test>first_connect".to_string()
            ))
        );
    }

    #[test]
    fn same_completion_cycle_is_deduplicated() {
        let mut telemetry = TelemetryService::new();
        let decision = completed_decision();
        let mut tracker = OnboardingCompletedEmissionTracker::default();

        let first = emit_onboarding_completed(
            &mut telemetry,
            &decision,
            "cycle-003",
            20,
            "ffxiv",
            &mut tracker,
        )
        .expect("first emit should succeed");
        let second = emit_onboarding_completed(
            &mut telemetry,
            &decision,
            "cycle-003",
            20,
            "ffxiv",
            &mut tracker,
        )
        .expect("duplicate cycle should be ignored");

        assert_eq!(first, OnboardingCompletedEmitStatus::Emitted);
        assert_eq!(second, OnboardingCompletedEmitStatus::SkippedDuplicateCycle);
        assert_eq!(telemetry.queue_depth(), 1);
    }

    #[test]
    fn payload_is_allowlisted_and_game_id_is_privacy_safe() {
        let mut telemetry = TelemetryService::new();
        let mut tracker = OnboardingCompletedEmissionTracker::default();

        emit_onboarding_completed(
            &mut telemetry,
            &completed_decision(),
            "cycle-004",
            18,
            "ffxiv@private-domain",
            &mut tracker,
        )
        .expect("unsafe game id should be sanitized");

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("event should be queued");
        assert_eq!(
            event.payload.get("game_id"),
            Some(&TelemetryValue::Text("unknown_game".to_string()))
        );

        let invalid = BTreeMap::from([
            (
                "onboarding_duration_s".to_string(),
                TelemetryValue::Integer(22),
            ),
            (
                "game_id".to_string(),
                TelemetryValue::Text("ffxiv".to_string()),
            ),
            (
                "success_path".to_string(),
                TelemetryValue::Text("welcome>first_connect".to_string()),
            ),
            (
                "private_field".to_string(),
                TelemetryValue::Text("secret".to_string()),
            ),
        ]);

        let error = validate_onboarding_completed_payload(&invalid)
            .expect_err("non-allowlisted keys must fail validation");
        assert_eq!(
            error.code,
            OnboardingCompletedSchemaErrorCode::InvalidPayloadKeys
        );
    }
}
