use super::apply_orchestrator::{
    RecoverableUpdateError, RecoverableUpdateErrorCode, UpdateApplyOutcome, UpdateCheckOutcome,
    UpdateDownloadOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdaterStateTag {
    UpToDate,
    UpdateAvailable,
    Downloading,
    ReadyToRestart,
    UpdateError,
}

impl UpdaterStateTag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UpToDate => "up_to_date",
            Self::UpdateAvailable => "update_available",
            Self::Downloading => "downloading",
            Self::ReadyToRestart => "ready_to_restart",
            Self::UpdateError => "update_error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdaterErrorReasonCode {
    InvalidVersionMetadata,
    FeedUnavailable,
    DownloadTransportFailed,
    PackageSignatureMismatch,
    PackageCorrupt,
    ApplyFailed,
    UnknownFailure,
}

impl UpdaterErrorReasonCode {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::InvalidVersionMetadata => "updater_invalid_version_metadata",
            Self::FeedUnavailable => "updater_feed_unavailable",
            Self::DownloadTransportFailed => "updater_download_transport_failed",
            Self::PackageSignatureMismatch => "updater_package_signature_mismatch",
            Self::PackageCorrupt => "updater_package_corrupt",
            Self::ApplyFailed => "updater_apply_failed",
            Self::UnknownFailure => "updater_unknown_failure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdaterState {
    UpToDate,
    UpdateAvailable { target_version: String },
    Downloading { target_version: String },
    ReadyToRestart { target_version: String },
    UpdateError { reason: UpdaterErrorReasonCode },
}

impl UpdaterState {
    pub fn tag(&self) -> UpdaterStateTag {
        match self {
            Self::UpToDate => UpdaterStateTag::UpToDate,
            Self::UpdateAvailable { .. } => UpdaterStateTag::UpdateAvailable,
            Self::Downloading { .. } => UpdaterStateTag::Downloading,
            Self::ReadyToRestart { .. } => UpdaterStateTag::ReadyToRestart,
            Self::UpdateError { .. } => UpdaterStateTag::UpdateError,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdaterLifecycleSignal {
    CheckResult(UpdateCheckOutcome),
    BeginDownload,
    DownloadResult(UpdateDownloadOutcome),
    ApplyResult(UpdateApplyOutcome),
    AcknowledgeRestart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdaterSignalTag {
    CheckResult,
    BeginDownload,
    DownloadResult,
    ApplyResult,
    AcknowledgeRestart,
}

impl UpdaterLifecycleSignal {
    pub fn tag(&self) -> UpdaterSignalTag {
        match self {
            Self::CheckResult(_) => UpdaterSignalTag::CheckResult,
            Self::BeginDownload => UpdaterSignalTag::BeginDownload,
            Self::DownloadResult(_) => UpdaterSignalTag::DownloadResult,
            Self::ApplyResult(_) => UpdaterSignalTag::ApplyResult,
            Self::AcknowledgeRestart => UpdaterSignalTag::AcknowledgeRestart,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterStateTransition {
    pub from: UpdaterStateTag,
    pub to: UpdaterStateTag,
    pub signal_tag: UpdaterSignalTag,
    pub reason: Option<UpdaterErrorReasonCode>,
    pub state: UpdaterState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterStateTransitionError {
    pub from: UpdaterStateTag,
    pub signal_tag: UpdaterSignalTag,
    pub reason_code: &'static str,
}

pub fn map_updater_error_reason(
    recoverable_error: Option<&RecoverableUpdateError>,
    raw_reason: Option<&str>,
) -> UpdaterErrorReasonCode {
    if let Some(error) = recoverable_error {
        return match error.code {
            RecoverableUpdateErrorCode::InvalidCurrentVersion
            | RecoverableUpdateErrorCode::InvalidReleaseVersion => {
                UpdaterErrorReasonCode::InvalidVersionMetadata
            }
            RecoverableUpdateErrorCode::ReleaseFeedUnavailable => {
                UpdaterErrorReasonCode::FeedUnavailable
            }
            RecoverableUpdateErrorCode::DownloadTransportFailed => {
                UpdaterErrorReasonCode::DownloadTransportFailed
            }
            RecoverableUpdateErrorCode::SignatureMismatch => {
                UpdaterErrorReasonCode::PackageSignatureMismatch
            }
            RecoverableUpdateErrorCode::PackageCorrupt => UpdaterErrorReasonCode::PackageCorrupt,
            RecoverableUpdateErrorCode::ApplyFailed => UpdaterErrorReasonCode::ApplyFailed,
        };
    }

    match raw_reason.unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "feed_unavailable" => UpdaterErrorReasonCode::FeedUnavailable,
        "download_transport_failed" => UpdaterErrorReasonCode::DownloadTransportFailed,
        "signature_mismatch" => UpdaterErrorReasonCode::PackageSignatureMismatch,
        "package_corrupt" => UpdaterErrorReasonCode::PackageCorrupt,
        "apply_failed" => UpdaterErrorReasonCode::ApplyFailed,
        "invalid_version_metadata" => UpdaterErrorReasonCode::InvalidVersionMetadata,
        _ => UpdaterErrorReasonCode::UnknownFailure,
    }
}

pub fn resolve_updater_state(
    current: &UpdaterState,
    signal: UpdaterLifecycleSignal,
) -> Result<UpdaterStateTransition, UpdaterStateTransitionError> {
    let from = current.tag();
    let signal_tag = signal.tag();
    let next = match (current, &signal) {
        (_, UpdaterLifecycleSignal::AcknowledgeRestart) => UpdaterState::UpToDate,
        (_, UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpToDate)) => {
            UpdaterState::UpToDate
        }
        (_, UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpdateAvailable(plan))) => {
            UpdaterState::UpdateAvailable {
                target_version: plan.target_version.clone(),
            }
        }
        (_, UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::RecoverableError(error))) => {
            UpdaterState::UpdateError {
                reason: map_updater_error_reason(Some(error), None),
            }
        }
        (UpdaterState::UpdateAvailable { target_version }, UpdaterLifecycleSignal::BeginDownload) => {
            UpdaterState::Downloading {
                target_version: target_version.clone(),
            }
        }
        (
            UpdaterState::Downloading { target_version },
            UpdaterLifecycleSignal::DownloadResult(UpdateDownloadOutcome::Downloaded(_)),
        ) => UpdaterState::Downloading {
            target_version: target_version.clone(),
        },
        (
            UpdaterState::Downloading { .. },
            UpdaterLifecycleSignal::DownloadResult(UpdateDownloadOutcome::RecoverableError(error)),
        ) => UpdaterState::UpdateError {
            reason: map_updater_error_reason(Some(error), None),
        },
        (
            UpdaterState::Downloading { .. },
            UpdaterLifecycleSignal::ApplyResult(UpdateApplyOutcome::ReadyToRestart {
                target_version,
            }),
        ) => UpdaterState::ReadyToRestart {
            target_version: target_version.clone(),
        },
        (
            UpdaterState::Downloading { .. },
            UpdaterLifecycleSignal::ApplyResult(UpdateApplyOutcome::RecoverableError(error)),
        ) => UpdaterState::UpdateError {
            reason: map_updater_error_reason(Some(error), None),
        },
        (UpdaterState::UpdateError { .. }, UpdaterLifecycleSignal::BeginDownload) => {
            UpdaterState::Downloading {
                target_version: "retry_pending".to_string(),
            }
        }
        _ => {
            return Err(UpdaterStateTransitionError {
                from,
                signal_tag,
                reason_code: "invalid_transition",
            })
        }
    };

    let reason = match &next {
        UpdaterState::UpdateError { reason } => Some(*reason),
        _ => None,
    };
    let to = next.tag();
    Ok(UpdaterStateTransition {
        from,
        to,
        signal_tag,
        reason,
        state: next,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        map_updater_error_reason, resolve_updater_state, UpdaterErrorReasonCode, UpdaterLifecycleSignal,
        UpdaterState, UpdaterStateTag,
    };
    use crate::updater::{
        RecoverableUpdateError, RecoverableUpdateErrorCode, UpdateApplyOutcome, UpdateChannel,
        UpdateCheckOutcome, UpdateDownloadOutcome, UpdatePlan,
    };

    fn stable_plan(version: &str) -> UpdatePlan {
        UpdatePlan {
            target_version: version.to_string(),
            channel: UpdateChannel::Stable,
            package_url: "https://updates.example.com/app.zip".to_string(),
            expected_signature: "sig-v1".to_string(),
        }
    }

    #[test]
    fn states_include_required_minimum_set() {
        let tags = [
            UpdaterStateTag::UpToDate,
            UpdaterStateTag::UpdateAvailable,
            UpdaterStateTag::Downloading,
            UpdaterStateTag::ReadyToRestart,
            UpdaterStateTag::UpdateError,
        ];
        let codes = tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>();
        assert_eq!(
            codes,
            vec![
                "up_to_date",
                "update_available",
                "downloading",
                "ready_to_restart",
                "update_error"
            ]
        );
    }

    #[test]
    fn reason_codes_are_deterministic_and_stable() {
        assert_eq!(
            map_updater_error_reason(
                Some(&RecoverableUpdateError {
                    phase: crate::updater::UpdateOperationPhase::Check,
                    code: RecoverableUpdateErrorCode::InvalidCurrentVersion,
                }),
                None
            ),
            UpdaterErrorReasonCode::InvalidVersionMetadata
        );
        assert_eq!(
            UpdaterErrorReasonCode::InvalidVersionMetadata.as_code(),
            "updater_invalid_version_metadata"
        );
        assert_eq!(
            UpdaterErrorReasonCode::DownloadTransportFailed.as_code(),
            "updater_download_transport_failed"
        );
        assert_eq!(
            UpdaterErrorReasonCode::UnknownFailure.as_code(),
            "updater_unknown_failure"
        );
    }

    #[test]
    fn unknown_failures_map_to_safe_fallback_category() {
        let mapped = map_updater_error_reason(None, Some("totally_new_error"));
        assert_eq!(mapped, UpdaterErrorReasonCode::UnknownFailure);
    }

    #[test]
    fn state_transitions_are_validated() {
        let initial = UpdaterState::UpToDate;
        let update_available = resolve_updater_state(
            &initial,
            UpdaterLifecycleSignal::CheckResult(UpdateCheckOutcome::UpdateAvailable(stable_plan(
                "1.2.0",
            ))),
        )
        .expect("check result should move state to update_available");
        assert_eq!(update_available.to, UpdaterStateTag::UpdateAvailable);

        let downloading = resolve_updater_state(
            &update_available.state,
            UpdaterLifecycleSignal::BeginDownload,
        )
        .expect("begin_download should move update_available -> downloading");
        assert_eq!(downloading.to, UpdaterStateTag::Downloading);

        let ready = resolve_updater_state(
            &downloading.state,
            UpdaterLifecycleSignal::ApplyResult(UpdateApplyOutcome::ReadyToRestart {
                target_version: "1.2.0".to_string(),
            }),
        )
        .expect("apply result should move downloading -> ready_to_restart");
        assert_eq!(ready.to, UpdaterStateTag::ReadyToRestart);

        let reset = resolve_updater_state(&ready.state, UpdaterLifecycleSignal::AcknowledgeRestart)
            .expect("restart acknowledge should reset state");
        assert_eq!(reset.to, UpdaterStateTag::UpToDate);

        let invalid = resolve_updater_state(&initial, UpdaterLifecycleSignal::BeginDownload);
        assert!(invalid.is_err());
    }

    #[test]
    fn recoverable_failures_transition_to_update_error() {
        let downloading = UpdaterState::Downloading {
            target_version: "1.5.0".to_string(),
        };
        let transition = resolve_updater_state(
            &downloading,
            UpdaterLifecycleSignal::DownloadResult(UpdateDownloadOutcome::RecoverableError(
                RecoverableUpdateError {
                    phase: crate::updater::UpdateOperationPhase::Download,
                    code: RecoverableUpdateErrorCode::DownloadTransportFailed,
                },
            )),
        )
        .expect("download failure should map to update_error");

        assert_eq!(transition.to, UpdaterStateTag::UpdateError);
        assert_eq!(
            transition.reason,
            Some(UpdaterErrorReasonCode::DownloadTransportFailed)
        );
    }
}
