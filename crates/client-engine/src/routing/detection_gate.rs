use crate::detection::state_resolver::{DetectionReasonCode, DetectionResolution, DetectionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionGateReasonCode {
    DetectionNotFound,
    DetectionStale,
    DetectionErrorPermissionDenied,
    DetectionErrorSnapshotUnavailable,
    DetectionErrorEnumerationFailed,
    DetectionErrorUnsupportedPlatform,
    DetectionErrorUnknown,
}

impl DetectionGateReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectionGateReasonCode::DetectionNotFound => "detection_not_found",
            DetectionGateReasonCode::DetectionStale => "detection_stale",
            DetectionGateReasonCode::DetectionErrorPermissionDenied => {
                "detection_error_permission_denied"
            }
            DetectionGateReasonCode::DetectionErrorSnapshotUnavailable => {
                "detection_error_snapshot_unavailable"
            }
            DetectionGateReasonCode::DetectionErrorEnumerationFailed => {
                "detection_error_enumeration_failed"
            }
            DetectionGateReasonCode::DetectionErrorUnsupportedPlatform => {
                "detection_error_unsupported_platform"
            }
            DetectionGateReasonCode::DetectionErrorUnknown => "detection_error_unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionGateDecision {
    pub can_activate: bool,
    pub detection_state: DetectionState,
    pub reason_code: Option<DetectionGateReasonCode>,
}

pub fn can_activate_routing(resolution: &DetectionResolution) -> DetectionGateDecision {
    match resolution.state {
        DetectionState::Detected => DetectionGateDecision {
            can_activate: true,
            detection_state: resolution.state,
            reason_code: None,
        },
        DetectionState::NotFound => DetectionGateDecision {
            can_activate: false,
            detection_state: resolution.state,
            reason_code: Some(DetectionGateReasonCode::DetectionNotFound),
        },
        DetectionState::Stale => DetectionGateDecision {
            can_activate: false,
            detection_state: resolution.state,
            reason_code: Some(DetectionGateReasonCode::DetectionStale),
        },
        DetectionState::Error => DetectionGateDecision {
            can_activate: false,
            detection_state: resolution.state,
            reason_code: Some(map_error_reason(resolution.metadata.reason_code)),
        },
    }
}

fn map_error_reason(reason_code: Option<DetectionReasonCode>) -> DetectionGateReasonCode {
    match reason_code {
        Some(DetectionReasonCode::PermissionDenied) => {
            DetectionGateReasonCode::DetectionErrorPermissionDenied
        }
        Some(DetectionReasonCode::SnapshotUnavailable) => {
            DetectionGateReasonCode::DetectionErrorSnapshotUnavailable
        }
        Some(DetectionReasonCode::EnumerationFailed) => {
            DetectionGateReasonCode::DetectionErrorEnumerationFailed
        }
        Some(DetectionReasonCode::UnsupportedPlatform) => {
            DetectionGateReasonCode::DetectionErrorUnsupportedPlatform
        }
        Some(DetectionReasonCode::StaleWindowExceeded) | None => {
            DetectionGateReasonCode::DetectionErrorUnknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{can_activate_routing, DetectionGateReasonCode};
    use crate::detection::state_resolver::{
        DetectionMetadata, DetectionReasonCode, DetectionResolution, DetectionState,
    };

    fn resolution(
        state: DetectionState,
        reason_code: Option<DetectionReasonCode>,
    ) -> DetectionResolution {
        DetectionResolution {
            state,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 200_000,
                last_detected_at_unix_ms: None,
                stale_after_ms: 15_000,
                stale_age_ms: None,
                reason_code,
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        }
    }

    #[test]
    fn detected_state_allows_activation() {
        let decision = can_activate_routing(&resolution(DetectionState::Detected, None));
        assert!(decision.can_activate);
        assert_eq!(decision.reason_code, None);
    }

    #[test]
    fn not_found_and_stale_are_denied_deterministically() {
        let not_found = can_activate_routing(&resolution(DetectionState::NotFound, None));
        assert!(!not_found.can_activate);
        assert_eq!(
            not_found.reason_code,
            Some(DetectionGateReasonCode::DetectionNotFound)
        );

        let stale = can_activate_routing(&resolution(
            DetectionState::Stale,
            Some(DetectionReasonCode::StaleWindowExceeded),
        ));
        assert!(!stale.can_activate);
        assert_eq!(
            stale.reason_code,
            Some(DetectionGateReasonCode::DetectionStale)
        );
    }

    #[test]
    fn error_state_maps_to_actionable_reason_codes() {
        let permission = can_activate_routing(&resolution(
            DetectionState::Error,
            Some(DetectionReasonCode::PermissionDenied),
        ));
        assert_eq!(
            permission.reason_code,
            Some(DetectionGateReasonCode::DetectionErrorPermissionDenied)
        );

        let unknown = can_activate_routing(&resolution(DetectionState::Error, None));
        assert_eq!(
            unknown.reason_code,
            Some(DetectionGateReasonCode::DetectionErrorUnknown)
        );
    }
}
