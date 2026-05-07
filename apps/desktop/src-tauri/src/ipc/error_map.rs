use client_engine::detection::commands::RescanCommandErrorCode;
use client_engine::detection::state_resolver::DetectionState;
use client_engine::metrics::{MetricsState, MetricsStateReasonCode};
use client_engine::routing::ToggleRejectionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcErrorKind {
    InvalidState,
    CommandRejected,
    DetectionScanFailed,
    MetricsStreamUnavailable,
    Timeout,
    UnknownFailure,
}

pub fn map_ipc_error_reason(kind: IpcErrorKind) -> &'static str {
    match kind {
        IpcErrorKind::InvalidState => "ipc_invalid_state",
        IpcErrorKind::CommandRejected => "ipc_command_rejected",
        IpcErrorKind::DetectionScanFailed => "ipc_detection_scan_failed",
        IpcErrorKind::MetricsStreamUnavailable => "ipc_metrics_stream_unavailable",
        IpcErrorKind::Timeout => "ipc_timeout",
        IpcErrorKind::UnknownFailure => "ipc_unknown_failure",
    }
}

pub fn map_routing_rejection_kind(kind: Option<ToggleRejectionKind>) -> IpcErrorKind {
    match kind {
        Some(ToggleRejectionKind::UnsupportedStateForCommand) => IpcErrorKind::InvalidState,
        Some(ToggleRejectionKind::StateMachineRejectedTransition) => IpcErrorKind::CommandRejected,
        None => IpcErrorKind::UnknownFailure,
    }
}

pub fn map_rescan_error_kind(code: RescanCommandErrorCode) -> IpcErrorKind {
    match code {
        RescanCommandErrorCode::ScanAlreadyInProgress => IpcErrorKind::CommandRejected,
        RescanCommandErrorCode::InternalStateUnavailable => IpcErrorKind::UnknownFailure,
    }
}

pub fn map_detection_state_error_kind(state: DetectionState) -> Option<IpcErrorKind> {
    match state {
        DetectionState::Error => Some(IpcErrorKind::DetectionScanFailed),
        DetectionState::Detected | DetectionState::NotFound | DetectionState::Stale => None,
    }
}

pub fn map_metrics_reason_kind(
    state: MetricsState,
    reason_code: Option<MetricsStateReasonCode>,
) -> Option<IpcErrorKind> {
    match reason_code {
        Some(MetricsStateReasonCode::FreshnessTimeout) => Some(IpcErrorKind::Timeout),
        Some(_) => Some(IpcErrorKind::MetricsStreamUnavailable),
        None if matches!(state, MetricsState::Error) => Some(IpcErrorKind::MetricsStreamUnavailable),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_detection_state_error_kind, map_ipc_error_reason, map_metrics_reason_kind,
        map_rescan_error_kind, map_routing_rejection_kind, IpcErrorKind,
    };
    use crate::ipc::contracts::IPC_REASON_CODES;
    use client_engine::detection::commands::RescanCommandErrorCode;
    use client_engine::detection::state_resolver::DetectionState;
    use client_engine::metrics::{MetricsState, MetricsStateReasonCode};
    use client_engine::routing::ToggleRejectionKind;

    #[test]
    fn normalized_reason_codes_are_bounded_and_stable() {
        let mapped = [
            map_ipc_error_reason(IpcErrorKind::InvalidState),
            map_ipc_error_reason(IpcErrorKind::CommandRejected),
            map_ipc_error_reason(IpcErrorKind::DetectionScanFailed),
            map_ipc_error_reason(IpcErrorKind::MetricsStreamUnavailable),
            map_ipc_error_reason(IpcErrorKind::Timeout),
            map_ipc_error_reason(IpcErrorKind::UnknownFailure),
        ];

        assert_eq!(
            mapped,
            [
                "ipc_invalid_state",
                "ipc_command_rejected",
                "ipc_detection_scan_failed",
                "ipc_metrics_stream_unavailable",
                "ipc_timeout",
                "ipc_unknown_failure",
            ]
        );
    }

    #[test]
    fn routing_and_detection_failures_map_to_expected_reason_kinds() {
        assert_eq!(
            map_routing_rejection_kind(Some(ToggleRejectionKind::UnsupportedStateForCommand)),
            IpcErrorKind::InvalidState
        );
        assert_eq!(
            map_routing_rejection_kind(Some(ToggleRejectionKind::StateMachineRejectedTransition)),
            IpcErrorKind::CommandRejected
        );
        assert_eq!(
            map_routing_rejection_kind(None),
            IpcErrorKind::UnknownFailure
        );

        assert_eq!(
            map_rescan_error_kind(RescanCommandErrorCode::ScanAlreadyInProgress),
            IpcErrorKind::CommandRejected
        );
        assert_eq!(
            map_rescan_error_kind(RescanCommandErrorCode::InternalStateUnavailable),
            IpcErrorKind::UnknownFailure
        );
        assert_eq!(
            map_detection_state_error_kind(DetectionState::Error),
            Some(IpcErrorKind::DetectionScanFailed)
        );
        assert_eq!(map_detection_state_error_kind(DetectionState::Detected), None);
    }

    #[test]
    fn metrics_and_unknown_paths_use_safe_fallback() {
        assert_eq!(
            map_metrics_reason_kind(
                MetricsState::Degraded,
                Some(MetricsStateReasonCode::FreshnessTimeout)
            ),
            Some(IpcErrorKind::Timeout)
        );
        assert_eq!(
            map_metrics_reason_kind(
                MetricsState::Error,
                Some(MetricsStateReasonCode::InvalidComputationWindow)
            ),
            Some(IpcErrorKind::MetricsStreamUnavailable)
        );
        assert_eq!(
            map_metrics_reason_kind(MetricsState::Error, None),
            Some(IpcErrorKind::MetricsStreamUnavailable)
        );
        assert_eq!(map_metrics_reason_kind(MetricsState::Live, None), None);
    }

    #[test]
    fn mapped_codes_exist_in_global_contract_allowlist() {
        let all = [
            map_ipc_error_reason(IpcErrorKind::InvalidState),
            map_ipc_error_reason(IpcErrorKind::CommandRejected),
            map_ipc_error_reason(IpcErrorKind::DetectionScanFailed),
            map_ipc_error_reason(IpcErrorKind::MetricsStreamUnavailable),
            map_ipc_error_reason(IpcErrorKind::Timeout),
            map_ipc_error_reason(IpcErrorKind::UnknownFailure),
        ];

        for code in all {
            assert!(
                IPC_REASON_CODES.contains(&code),
                "reason code '{code}' must be present in contracts"
            );
        }
    }
}
