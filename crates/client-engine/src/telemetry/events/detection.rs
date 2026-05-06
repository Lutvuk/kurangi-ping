use std::collections::BTreeMap;

use crate::detection::state_resolver::{DetectionResolution, DetectionState};
use crate::telemetry::{TelemetryPayload, TelemetryService, TelemetryValue};
use crate::telemetry::validator::{
    validate_event_payload, TelemetryValidationErrorCode, UnknownKeyPolicy,
};

pub const GAME_DETECTED_EVENT_NAME: &str = "game_detected";
pub const GAME_DETECTED_ALLOWED_KEYS: [&str; 3] =
    ["game_id", "process_name", "detection_time_ms"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetectedSchemaErrorCode {
    InvalidTransition,
    MissingRequiredField,
    InvalidPayloadKeys,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDetectedSchemaError {
    pub code: GameDetectedSchemaErrorCode,
    pub message: String,
}

impl GameDetectedSchemaError {
    fn new(code: GameDetectedSchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_game_detected_event(
    telemetry: &mut TelemetryService,
    previous_state: DetectionState,
    resolution: &DetectionResolution,
) -> Result<bool, GameDetectedSchemaError> {
    if !(previous_state != DetectionState::Detected
        && resolution.state == DetectionState::Detected)
    {
        return Ok(false);
    }

    let game_id =
        resolution
            .metadata
            .matched_game_id
            .clone()
            .ok_or_else(|| {
                GameDetectedSchemaError::new(
                    GameDetectedSchemaErrorCode::MissingRequiredField,
                    "game_detected requires matched game_id",
                )
            })?;

    let process_name = resolution
        .metadata
        .matched_executable_name
        .clone()
        .ok_or_else(|| {
            GameDetectedSchemaError::new(
                GameDetectedSchemaErrorCode::MissingRequiredField,
                "game_detected requires matched process_name",
            )
        })?;

    let detection_time_ms = i64::try_from(resolution.metadata.scanned_at_unix_ms).map_err(|_| {
        GameDetectedSchemaError::new(
            GameDetectedSchemaErrorCode::MissingRequiredField,
            "game_detected requires valid detection_time_ms",
        )
    })?;

    let payload = BTreeMap::from([
        ("game_id".to_string(), TelemetryValue::Text(game_id)),
        ("process_name".to_string(), TelemetryValue::Text(process_name)),
        (
            "detection_time_ms".to_string(),
            TelemetryValue::Integer(detection_time_ms),
        ),
    ]);

    validate_game_detected_payload(&payload)?;
    telemetry
        .enqueue_validated(GAME_DETECTED_EVENT_NAME, payload, UnknownKeyPolicy::Reject)
        .map_err(map_validator_error)?;
    Ok(true)
}

pub fn validate_game_detected_payload(
    payload: &TelemetryPayload,
) -> Result<(), GameDetectedSchemaError> {
    validate_event_payload(GAME_DETECTED_EVENT_NAME, payload, UnknownKeyPolicy::Reject)
        .map_err(map_validator_error)?;

    match payload.get("game_id") {
        Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
        _ => {
            return Err(GameDetectedSchemaError::new(
                GameDetectedSchemaErrorCode::MissingRequiredField,
                "game_detected payload requires non-empty game_id",
            ));
        }
    }

    match payload.get("process_name") {
        Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
        _ => {
            return Err(GameDetectedSchemaError::new(
                GameDetectedSchemaErrorCode::MissingRequiredField,
                "game_detected payload requires non-empty process_name",
            ));
        }
    }

    match payload.get("detection_time_ms") {
        Some(TelemetryValue::Integer(value)) if *value >= 0 => {}
        _ => {
            return Err(GameDetectedSchemaError::new(
                GameDetectedSchemaErrorCode::MissingRequiredField,
                "game_detected payload requires non-negative detection_time_ms",
            ));
        }
    }

    Ok(())
}

fn map_validator_error(
    error: crate::telemetry::validator::TelemetryValidationError,
) -> GameDetectedSchemaError {
    let code = match error.code {
        TelemetryValidationErrorCode::UnknownEventName
        | TelemetryValidationErrorCode::UnknownPayloadKey
        | TelemetryValidationErrorCode::SensitiveFieldViolation => {
            GameDetectedSchemaErrorCode::InvalidPayloadKeys
        }
        TelemetryValidationErrorCode::MissingRequiredKey => {
            GameDetectedSchemaErrorCode::MissingRequiredField
        }
    };
    GameDetectedSchemaError::new(code, error.message)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::detection::state_resolver::{
        DetectionMetadata, DetectionReasonCode, DetectionResolution, DetectionState,
    };
    use crate::telemetry::{TelemetryService, TelemetryValue};

    use super::{
        emit_game_detected_event, validate_game_detected_payload, GameDetectedSchemaErrorCode,
    };

    fn detected_resolution() -> DetectionResolution {
        DetectionResolution {
            state: DetectionState::Detected,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 120_000,
                last_detected_at_unix_ms: Some(120_000),
                stale_after_ms: 15_000,
                stale_age_ms: Some(0),
                reason_code: None,
                match_count: 1,
                matched_process_id: Some(222),
                matched_game_id: Some("ffxiv".to_string()),
                matched_executable_name: Some("ffxiv_dx11.exe".to_string()),
            },
        }
    }

    #[test]
    fn emits_only_on_successful_detection_transition() {
        let mut telemetry = TelemetryService::new();
        let resolution = detected_resolution();

        let emitted = emit_game_detected_event(&mut telemetry, DetectionState::NotFound, &resolution)
            .expect("successful transition should enqueue event");
        assert!(emitted);
        assert_eq!(telemetry.queue_depth(), 1);

        let emitted =
            emit_game_detected_event(&mut telemetry, DetectionState::Detected, &resolution)
                .expect("detected->detected should not enqueue");
        assert!(!emitted);
        assert_eq!(telemetry.queue_depth(), 1);
    }

    #[test]
    fn enqueued_payload_uses_only_allowlisted_keys() {
        let mut telemetry = TelemetryService::new();
        let resolution = detected_resolution();

        emit_game_detected_event(&mut telemetry, DetectionState::NotFound, &resolution)
            .expect("should emit game_detected");
        let batch = telemetry.drain_batch(10);
        assert_eq!(batch.len(), 1);
        let event = &batch[0];
        assert_eq!(event.name, "game_detected");
        assert_eq!(event.payload.len(), 3);
        assert!(event.payload.contains_key("game_id"));
        assert!(event.payload.contains_key("process_name"));
        assert!(event.payload.contains_key("detection_time_ms"));
    }

    #[test]
    fn non_allowlisted_fields_are_rejected_before_enqueue() {
        let payload = BTreeMap::from([
            ("game_id".to_string(), TelemetryValue::Text("ffxiv".to_string())),
            (
                "process_name".to_string(),
                TelemetryValue::Text("ffxiv_dx11.exe".to_string()),
            ),
            (
                "detection_time_ms".to_string(),
                TelemetryValue::Integer(120_000),
            ),
            ("pid".to_string(), TelemetryValue::Integer(999)),
        ]);

        let error = validate_game_detected_payload(&payload).expect_err("extra key must fail");
        assert_eq!(error.code, GameDetectedSchemaErrorCode::InvalidPayloadKeys);
    }

    #[test]
    fn missing_required_transition_metadata_fails_schema_guard() {
        let mut telemetry = TelemetryService::new();
        let mut resolution = detected_resolution();
        resolution.metadata.matched_game_id = None;

        let error = emit_game_detected_event(&mut telemetry, DetectionState::NotFound, &resolution)
            .expect_err("missing required payload field must fail");
        assert_eq!(error.code, GameDetectedSchemaErrorCode::MissingRequiredField);
        assert_eq!(telemetry.queue_depth(), 0);
    }

    #[test]
    fn non_detected_target_state_does_not_emit() {
        let mut telemetry = TelemetryService::new();
        let resolution = DetectionResolution {
            state: DetectionState::Stale,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 150_000,
                last_detected_at_unix_ms: Some(120_000),
                stale_after_ms: 10_000,
                stale_age_ms: Some(30_000),
                reason_code: Some(DetectionReasonCode::StaleWindowExceeded),
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        };

        let emitted = emit_game_detected_event(&mut telemetry, DetectionState::Detected, &resolution)
            .expect("stale transition should not emit");
        assert!(!emitted);
        assert_eq!(telemetry.queue_depth(), 0);
    }
}
