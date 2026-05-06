use std::collections::BTreeMap;

use crate::telemetry::validator::{
    validate_event_payload, TelemetryValidationErrorCode, UnknownKeyPolicy,
};
use crate::telemetry::{TelemetryPayload, TelemetryService, TelemetryValue};
use crate::updater::{UpdateChannel, UpdaterErrorReasonCode, UpdaterSignalTag, UpdaterState, UpdaterStateTransition};

pub const UPDATER_CHECK_EVENT_NAME: &str = "updater_check";
pub const UPDATER_AVAILABLE_EVENT_NAME: &str = "updater_available";
pub const UPDATER_DOWNLOAD_EVENT_NAME: &str = "updater_download";
pub const UPDATER_APPLY_EVENT_NAME: &str = "updater_apply";
pub const UPDATER_FAILURE_EVENT_NAME: &str = "updater_failure";
pub const UPDATER_ALLOWED_KEYS: [&str; 4] = ["state", "channel", "target_version", "reason_code"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdaterTelemetryEmitStatus {
    Emitted(&'static str),
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdaterTelemetrySchemaErrorCode {
    InvalidPayloadKeys,
    MissingRequiredField,
    InvalidPayloadValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterTelemetrySchemaError {
    pub code: UpdaterTelemetrySchemaErrorCode,
    pub message: String,
}

impl UpdaterTelemetrySchemaError {
    fn new(code: UpdaterTelemetrySchemaErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn emit_updater_event(
    telemetry: &mut TelemetryService,
    transition: &UpdaterStateTransition,
    channel: UpdateChannel,
) -> Result<UpdaterTelemetryEmitStatus, UpdaterTelemetrySchemaError> {
    let Some(event_name) = map_event_name(transition) else {
        return Ok(UpdaterTelemetryEmitStatus::Skipped);
    };

    let payload = build_payload(event_name, transition, channel)?;
    validate_updater_payload(event_name, &payload)?;
    telemetry
        .enqueue_validated(event_name, payload, UnknownKeyPolicy::Reject)
        .map_err(map_validator_error)?;

    Ok(UpdaterTelemetryEmitStatus::Emitted(event_name))
}

fn map_event_name(transition: &UpdaterStateTransition) -> Option<&'static str> {
    if matches!(transition.state, UpdaterState::UpdateError { .. }) {
        return Some(UPDATER_FAILURE_EVENT_NAME);
    }

    match transition.signal_tag {
        UpdaterSignalTag::CheckResult => match transition.state {
            UpdaterState::UpToDate => Some(UPDATER_CHECK_EVENT_NAME),
            UpdaterState::UpdateAvailable { .. } => Some(UPDATER_AVAILABLE_EVENT_NAME),
            _ => None,
        },
        UpdaterSignalTag::BeginDownload => Some(UPDATER_DOWNLOAD_EVENT_NAME),
        UpdaterSignalTag::ApplyResult => match transition.state {
            UpdaterState::ReadyToRestart { .. } => Some(UPDATER_APPLY_EVENT_NAME),
            _ => None,
        },
        UpdaterSignalTag::DownloadResult | UpdaterSignalTag::AcknowledgeRestart => None,
    }
}

fn build_payload(
    event_name: &str,
    transition: &UpdaterStateTransition,
    channel: UpdateChannel,
) -> Result<TelemetryPayload, UpdaterTelemetrySchemaError> {
    let mut payload = BTreeMap::from([
        (
            "state".to_string(),
            TelemetryValue::Text(transition.to.as_str().to_string()),
        ),
        (
            "channel".to_string(),
            TelemetryValue::Text(channel.as_str().to_string()),
        ),
    ]);

    if matches!(
        event_name,
        UPDATER_AVAILABLE_EVENT_NAME | UPDATER_DOWNLOAD_EVENT_NAME | UPDATER_APPLY_EVENT_NAME
    ) {
        let version = extract_target_version(&transition.state).ok_or_else(|| {
            UpdaterTelemetrySchemaError::new(
                UpdaterTelemetrySchemaErrorCode::MissingRequiredField,
                "updater event requires target_version for availability/download/apply stages",
            )
        })?;
        payload.insert(
            "target_version".to_string(),
            TelemetryValue::Text(sanitize_target_version(version)),
        );
    }

    if event_name == UPDATER_FAILURE_EVENT_NAME {
        let reason = transition
            .reason
            .unwrap_or(UpdaterErrorReasonCode::UnknownFailure)
            .as_code()
            .to_string();
        payload.insert("reason_code".to_string(), TelemetryValue::Text(reason));
    }

    Ok(payload)
}

fn extract_target_version(state: &UpdaterState) -> Option<&str> {
    match state {
        UpdaterState::UpdateAvailable { target_version }
        | UpdaterState::Downloading { target_version }
        | UpdaterState::ReadyToRestart { target_version } => Some(target_version.as_str()),
        UpdaterState::UpToDate | UpdaterState::UpdateError { .. } => None,
    }
}

fn sanitize_target_version(raw: &str) -> String {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return "unknown_version".to_string();
    }

    if normalized.len() > 32 {
        return "unknown_version".to_string();
    }

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_')
    {
        normalized
    } else {
        "unknown_version".to_string()
    }
}

pub fn validate_updater_payload(
    event_name: &str,
    payload: &TelemetryPayload,
) -> Result<(), UpdaterTelemetrySchemaError> {
    validate_event_payload(event_name, payload, UnknownKeyPolicy::Reject).map_err(map_validator_error)?;

    let required_keys = match event_name {
        UPDATER_CHECK_EVENT_NAME => &["state", "channel"][..],
        UPDATER_AVAILABLE_EVENT_NAME | UPDATER_DOWNLOAD_EVENT_NAME | UPDATER_APPLY_EVENT_NAME => {
            &["state", "channel", "target_version"][..]
        }
        UPDATER_FAILURE_EVENT_NAME => &["state", "channel", "reason_code"][..],
        _ => {
            return Err(UpdaterTelemetrySchemaError::new(
                UpdaterTelemetrySchemaErrorCode::InvalidPayloadKeys,
                format!("unsupported updater event name: {event_name}"),
            ))
        }
    };

    for key in required_keys {
        match payload.get(*key) {
            Some(TelemetryValue::Text(value)) if !value.trim().is_empty() => {}
            _ => {
                return Err(UpdaterTelemetrySchemaError::new(
                    UpdaterTelemetrySchemaErrorCode::MissingRequiredField,
                    format!("updater event payload requires non-empty {key}"),
                ))
            }
        }
    }

    // Block obvious sensitive path-like version contamination in target_version.
    if let Some(TelemetryValue::Text(value)) = payload.get("target_version") {
        if value.contains('\\') || value.contains('/') || value.contains(':') {
            return Err(UpdaterTelemetrySchemaError::new(
                UpdaterTelemetrySchemaErrorCode::InvalidPayloadValue,
                "target_version must not include path-like details",
            ));
        }
    }

    Ok(())
}

fn map_validator_error(
    error: crate::telemetry::validator::TelemetryValidationError,
) -> UpdaterTelemetrySchemaError {
    let code = match error.code {
        TelemetryValidationErrorCode::UnknownEventName
        | TelemetryValidationErrorCode::UnknownPayloadKey
        | TelemetryValidationErrorCode::SensitiveFieldViolation => {
            UpdaterTelemetrySchemaErrorCode::InvalidPayloadKeys
        }
        TelemetryValidationErrorCode::MissingRequiredKey => {
            UpdaterTelemetrySchemaErrorCode::MissingRequiredField
        }
    };

    UpdaterTelemetrySchemaError::new(code, error.message)
}

#[cfg(test)]
mod tests {
    use super::{
        emit_updater_event, validate_updater_payload, UpdaterTelemetryEmitStatus,
        UpdaterTelemetrySchemaErrorCode, UPDATER_ALLOWED_KEYS, UPDATER_APPLY_EVENT_NAME,
        UPDATER_AVAILABLE_EVENT_NAME, UPDATER_CHECK_EVENT_NAME, UPDATER_DOWNLOAD_EVENT_NAME,
        UPDATER_FAILURE_EVENT_NAME,
    };
    use crate::telemetry::{TelemetryService, TelemetryValue};
    use crate::updater::{
        resolve_updater_state, RecoverableUpdateError, RecoverableUpdateErrorCode, UpdateChannel,
        UpdateCheckOutcome, UpdateOperationPhase, UpdatePlan, UpdaterLifecycleSignal,
        UpdaterState,
    };
    use std::collections::BTreeMap;

    fn plan(version: &str) -> UpdatePlan {
        UpdatePlan {
            target_version: version.to_string(),
            channel: UpdateChannel::Stable,
            package_url: "https://updates.example.com/kp.zip".to_string(),
            expected_signature: "sig".to_string(),
        }
    }

    #[test]
    fn check_available_download_apply_and_failure_events_can_be_emitted() {
        let mut telemetry = TelemetryService::new();
        let mut state = UpdaterState::UpToDate;

        let available = resolve_updater_state(
            &state,
            UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpdateAvailable(plan("1.2.0"))),
        )
        .expect("check should resolve update_available");
        assert_eq!(
            emit_updater_event(&mut telemetry, &available, UpdateChannel::Stable)
                .expect("available event should emit"),
            UpdaterTelemetryEmitStatus::Emitted(UPDATER_AVAILABLE_EVENT_NAME)
        );
        state = available.state.clone();

        let download = resolve_updater_state(&state, UpdaterLifecycleSignal::BeginDownload)
            .expect("begin download should resolve downloading");
        assert_eq!(
            emit_updater_event(&mut telemetry, &download, UpdateChannel::Stable)
                .expect("download event should emit"),
            UpdaterTelemetryEmitStatus::Emitted(UPDATER_DOWNLOAD_EVENT_NAME)
        );
        state = download.state.clone();

        let applied = resolve_updater_state(
            &state,
            UpdaterLifecycleSignal::ApplyResult(crate::updater::UpdateApplyOutcome::ReadyToRestart {
                target_version: "1.2.0".to_string(),
            }),
        )
        .expect("apply success should resolve ready_to_restart");
        assert_eq!(
            emit_updater_event(&mut telemetry, &applied, UpdateChannel::Stable)
                .expect("apply event should emit"),
            UpdaterTelemetryEmitStatus::Emitted(UPDATER_APPLY_EVENT_NAME)
        );

        let check_up_to_date = resolve_updater_state(
            &UpdaterState::UpToDate,
            UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpToDate),
        )
        .expect("check up-to-date should resolve");
        assert_eq!(
            emit_updater_event(&mut telemetry, &check_up_to_date, UpdateChannel::Stable)
                .expect("check event should emit"),
            UpdaterTelemetryEmitStatus::Emitted(UPDATER_CHECK_EVENT_NAME)
        );

        let failure_transition = resolve_updater_state(
            &UpdaterState::Downloading {
                target_version: "1.2.0".to_string(),
            },
            UpdaterLifecycleSignal::ApplyResult(crate::updater::UpdateApplyOutcome::RecoverableError(
                RecoverableUpdateError {
                    phase: UpdateOperationPhase::Apply,
                    code: RecoverableUpdateErrorCode::ApplyFailed,
                },
            )),
        )
        .expect("failure transition should resolve");
        assert_eq!(
            emit_updater_event(&mut telemetry, &failure_transition, UpdateChannel::Stable)
                .expect("failure event should emit"),
            UpdaterTelemetryEmitStatus::Emitted(UPDATER_FAILURE_EVENT_NAME)
        );
    }

    #[test]
    fn payloads_follow_telemetry_allowlist_rules() {
        let payload = BTreeMap::from([
            (
                "state".to_string(),
                TelemetryValue::Text("update_available".to_string()),
            ),
            (
                "channel".to_string(),
                TelemetryValue::Text("stable".to_string()),
            ),
            (
                "target_version".to_string(),
                TelemetryValue::Text("1.2.0".to_string()),
            ),
        ]);
        validate_updater_payload(UPDATER_AVAILABLE_EVENT_NAME, &payload)
            .expect("valid updater payload should pass");

        let mut invalid = payload.clone();
        invalid.insert(
            "private_host".to_string(),
            TelemetryValue::Text("internal.example.net".to_string()),
        );
        let error = validate_updater_payload(UPDATER_AVAILABLE_EVENT_NAME, &invalid)
            .expect_err("extra payload key should fail allowlist");
        assert_eq!(error.code, UpdaterTelemetrySchemaErrorCode::InvalidPayloadKeys);

        assert_eq!(
            UPDATER_ALLOWED_KEYS,
            ["state", "channel", "target_version", "reason_code"]
        );
    }

    #[test]
    fn sensitive_release_or_local_path_details_are_excluded() {
        let mut telemetry = TelemetryService::new();
        let transition = resolve_updater_state(
            &UpdaterState::UpToDate,
            UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpdateAvailable(plan(
                "C:\\Users\\lutfi\\secret-build",
            ))),
        )
        .expect("transition should resolve");

        emit_updater_event(&mut telemetry, &transition, UpdateChannel::Stable)
            .expect("event should emit with sanitized version");
        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("event should be queued");
        assert_eq!(event.name, UPDATER_AVAILABLE_EVENT_NAME);
        assert_eq!(
            event.payload.get("target_version"),
            Some(&TelemetryValue::Text("unknown_version".to_string()))
        );
    }

    #[test]
    fn event_sequence_remains_deterministic() {
        let run_once = || {
            let mut telemetry = TelemetryService::new();
            let mut state = UpdaterState::UpToDate;

            let available = resolve_updater_state(
                &state,
                UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpdateAvailable(plan(
                    "1.2.0",
                ))),
            )
            .expect("check should resolve available");
            emit_updater_event(&mut telemetry, &available, UpdateChannel::Stable)
                .expect("available event should emit");
            state = available.state.clone();

            let download = resolve_updater_state(&state, UpdaterLifecycleSignal::BeginDownload)
                .expect("download should resolve");
            emit_updater_event(&mut telemetry, &download, UpdateChannel::Stable)
                .expect("download event should emit");
            state = download.state.clone();

            let failed = resolve_updater_state(
                &state,
                UpdaterLifecycleSignal::ApplyResult(crate::updater::UpdateApplyOutcome::RecoverableError(
                    RecoverableUpdateError {
                        phase: UpdateOperationPhase::Apply,
                        code: RecoverableUpdateErrorCode::ApplyFailed,
                    },
                )),
            )
            .expect("failure should resolve");
            emit_updater_event(&mut telemetry, &failed, UpdateChannel::Stable)
                .expect("failure event should emit");

            telemetry
                .drain_batch(16)
                .into_iter()
                .map(|event| event.name)
                .collect::<Vec<_>>()
        };

        let first = run_once();
        let second = run_once();
        assert_eq!(first, second);
        assert_eq!(
            first,
            vec![
                UPDATER_AVAILABLE_EVENT_NAME.to_string(),
                UPDATER_DOWNLOAD_EVENT_NAME.to_string(),
                UPDATER_FAILURE_EVENT_NAME.to_string(),
            ]
        );
    }
}
