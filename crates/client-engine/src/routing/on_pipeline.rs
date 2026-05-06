//! ON activation gate chain: detection -> manifest -> route precheck.

use super::{
    can_activate_routing, verify_manifest_or_fail, DetectionGateDecision, DetectionGateReasonCode,
    DetectionResolution, ManifestFailureCode, ManifestGateResult, RelayManifestDto, RouteCandidate,
    RouteProtocol,
};
use crate::security::signature::ManifestSignatureVerifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnPipelineStage {
    DetectionGate,
    ManifestGate,
    RoutePrecheck,
}

impl OnPipelineStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DetectionGate => "detection_gate",
            Self::ManifestGate => "manifest_gate",
            Self::RoutePrecheck => "route_precheck",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnPipelineReasonCode {
    DetectionNotFound,
    DetectionStale,
    DetectionErrorPermissionDenied,
    DetectionErrorSnapshotUnavailable,
    DetectionErrorEnumerationFailed,
    DetectionErrorUnsupportedPlatform,
    DetectionErrorUnknown,
    ManifestSignatureInvalid,
    ManifestExpired,
    ManifestEmptyCandidates,
    PrecheckNoProtocolsConfigured,
    PrecheckNoCandidatesAvailable,
}

impl OnPipelineReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DetectionNotFound => "detection_not_found",
            Self::DetectionStale => "detection_stale",
            Self::DetectionErrorPermissionDenied => "detection_error_permission_denied",
            Self::DetectionErrorSnapshotUnavailable => "detection_error_snapshot_unavailable",
            Self::DetectionErrorEnumerationFailed => "detection_error_enumeration_failed",
            Self::DetectionErrorUnsupportedPlatform => "detection_error_unsupported_platform",
            Self::DetectionErrorUnknown => "detection_error_unknown",
            Self::ManifestSignatureInvalid => "manifest_signature_invalid",
            Self::ManifestExpired => "manifest_expired",
            Self::ManifestEmptyCandidates => "manifest_empty_candidates",
            Self::PrecheckNoProtocolsConfigured => "precheck_no_protocols_configured",
            Self::PrecheckNoCandidatesAvailable => "precheck_no_candidates_available",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePrecheckResult {
    pub ready_for_activation: bool,
    pub protocol_count: usize,
    pub candidate_count: usize,
    pub reason_code: Option<OnPipelineReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnPipelineFailure {
    pub stage: OnPipelineStage,
    pub reason_code: OnPipelineReasonCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnPipelineStatus {
    Ready {
        manifest_version: String,
        candidates: Vec<RouteCandidate>,
        protocol_order: Vec<RouteProtocol>,
    },
    Blocked {
        failure: OnPipelineFailure,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnPipelineResult {
    pub detection_gate: DetectionGateDecision,
    pub manifest_gate: Option<ManifestGateResult>,
    pub route_precheck: Option<RoutePrecheckResult>,
    pub status: OnPipelineStatus,
}

pub fn execute_on_pipeline<V: ManifestSignatureVerifier>(
    detection_resolution: &DetectionResolution,
    verifier: &V,
    manifest: &RelayManifestDto,
    now_unix_s: u64,
    protocol_order: &[RouteProtocol],
) -> OnPipelineResult {
    let detection_gate = can_activate_routing(detection_resolution);
    if !detection_gate.can_activate {
        let reason = map_detection_reason(
            detection_gate
                .reason_code
                .unwrap_or(DetectionGateReasonCode::DetectionErrorUnknown),
        );
        return OnPipelineResult {
            detection_gate,
            manifest_gate: None,
            route_precheck: None,
            status: OnPipelineStatus::Blocked {
                failure: OnPipelineFailure {
                    stage: OnPipelineStage::DetectionGate,
                    reason_code: reason,
                },
            },
        };
    }

    let manifest_gate = verify_manifest_or_fail(verifier, manifest, now_unix_s);
    if let Some(failure_code) = manifest_gate.failure_code() {
        return OnPipelineResult {
            detection_gate,
            manifest_gate: Some(manifest_gate),
            route_precheck: None,
            status: OnPipelineStatus::Blocked {
                failure: OnPipelineFailure {
                    stage: OnPipelineStage::ManifestGate,
                    reason_code: map_manifest_reason(failure_code),
                },
            },
        };
    }

    let (manifest_version, candidates) = match &manifest_gate {
        ManifestGateResult::Passed {
            manifest_version,
            candidates,
        } => (manifest_version.clone(), candidates.clone()),
        ManifestGateResult::Blocked { .. } => unreachable!("blocked branch returned above"),
    };

    let route_precheck = run_route_precheck(&candidates, protocol_order);
    if !route_precheck.ready_for_activation {
        return OnPipelineResult {
            detection_gate,
            manifest_gate: Some(manifest_gate),
            route_precheck: Some(route_precheck.clone()),
            status: OnPipelineStatus::Blocked {
                failure: OnPipelineFailure {
                    stage: OnPipelineStage::RoutePrecheck,
                    reason_code: route_precheck
                        .reason_code
                        .unwrap_or(OnPipelineReasonCode::PrecheckNoCandidatesAvailable),
                },
            },
        };
    }

    OnPipelineResult {
        detection_gate,
        manifest_gate: Some(manifest_gate),
        route_precheck: Some(route_precheck),
        status: OnPipelineStatus::Ready {
            manifest_version,
            candidates,
            protocol_order: protocol_order.to_vec(),
        },
    }
}

fn run_route_precheck(
    candidates: &[RouteCandidate],
    protocol_order: &[RouteProtocol],
) -> RoutePrecheckResult {
    if protocol_order.is_empty() {
        return RoutePrecheckResult {
            ready_for_activation: false,
            protocol_count: 0,
            candidate_count: candidates.len(),
            reason_code: Some(OnPipelineReasonCode::PrecheckNoProtocolsConfigured),
        };
    }

    if candidates.is_empty() {
        return RoutePrecheckResult {
            ready_for_activation: false,
            protocol_count: protocol_order.len(),
            candidate_count: 0,
            reason_code: Some(OnPipelineReasonCode::PrecheckNoCandidatesAvailable),
        };
    }

    RoutePrecheckResult {
        ready_for_activation: true,
        protocol_count: protocol_order.len(),
        candidate_count: candidates.len(),
        reason_code: None,
    }
}

fn map_detection_reason(reason: DetectionGateReasonCode) -> OnPipelineReasonCode {
    match reason {
        DetectionGateReasonCode::DetectionNotFound => OnPipelineReasonCode::DetectionNotFound,
        DetectionGateReasonCode::DetectionStale => OnPipelineReasonCode::DetectionStale,
        DetectionGateReasonCode::DetectionErrorPermissionDenied => {
            OnPipelineReasonCode::DetectionErrorPermissionDenied
        }
        DetectionGateReasonCode::DetectionErrorSnapshotUnavailable => {
            OnPipelineReasonCode::DetectionErrorSnapshotUnavailable
        }
        DetectionGateReasonCode::DetectionErrorEnumerationFailed => {
            OnPipelineReasonCode::DetectionErrorEnumerationFailed
        }
        DetectionGateReasonCode::DetectionErrorUnsupportedPlatform => {
            OnPipelineReasonCode::DetectionErrorUnsupportedPlatform
        }
        DetectionGateReasonCode::DetectionErrorUnknown => {
            OnPipelineReasonCode::DetectionErrorUnknown
        }
    }
}

fn map_manifest_reason(reason: ManifestFailureCode) -> OnPipelineReasonCode {
    match reason {
        ManifestFailureCode::SignatureInvalid => OnPipelineReasonCode::ManifestSignatureInvalid,
        ManifestFailureCode::ManifestExpired => OnPipelineReasonCode::ManifestExpired,
        ManifestFailureCode::NoRelayCandidates => OnPipelineReasonCode::ManifestEmptyCandidates,
    }
}

#[cfg(test)]
mod tests {
    use super::{execute_on_pipeline, OnPipelineReasonCode, OnPipelineStage, OnPipelineStatus};
    use crate::detection::state_resolver::{
        DetectionMetadata, DetectionReasonCode, DetectionResolution, DetectionState,
    };
    use crate::routing::{RelayManifestDto, RelayNodeDto, RouteProtocol};
    use crate::security::signature::AllowlistSignatureVerifier;

    fn detection_resolution(
        state: DetectionState,
        reason_code: Option<DetectionReasonCode>,
    ) -> DetectionResolution {
        DetectionResolution {
            state,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: 1_700_000_000_000,
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

    fn sample_manifest(signature_b64: &str, valid_until_unix_s: u64) -> RelayManifestDto {
        RelayManifestDto {
            version: "2026.08.0".to_string(),
            valid_until_unix_s,
            signature_b64: signature_b64.to_string(),
            relays: vec![RelayNodeDto {
                relay_id: "sin-01".to_string(),
                region: "sin".to_string(),
                hostname: "sin-01.example.net".to_string(),
                priority: 1,
            }],
        }
    }

    fn protocol_order() -> [RouteProtocol; 3] {
        [
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ]
    }

    #[test]
    fn detection_gate_is_required_and_enforced() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let not_found = detection_resolution(DetectionState::NotFound, None);

        let result = execute_on_pipeline(
            &not_found,
            &verifier,
            &manifest,
            1_700_000_000,
            &protocol_order(),
        );

        assert!(!result.detection_gate.can_activate);
        assert!(result.manifest_gate.is_none());
        assert!(result.route_precheck.is_none());
        match result.status {
            OnPipelineStatus::Blocked { failure } => {
                assert_eq!(failure.stage, OnPipelineStage::DetectionGate);
                assert_eq!(failure.reason_code, OnPipelineReasonCode::DetectionNotFound);
                assert_eq!(failure.reason_code.as_str(), "detection_not_found");
            }
            OnPipelineStatus::Ready { .. } => {
                panic!("detection block should not return ready state")
            }
        }
    }

    #[test]
    fn manifest_gate_is_required_and_enforced() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("bad-signature", 1_800_000_000);
        let detected = detection_resolution(DetectionState::Detected, None);

        let result = execute_on_pipeline(
            &detected,
            &verifier,
            &manifest,
            1_700_000_000,
            &protocol_order(),
        );

        assert!(result.detection_gate.can_activate);
        assert!(result.manifest_gate.is_some());
        assert!(result.route_precheck.is_none());
        match result.status {
            OnPipelineStatus::Blocked { failure } => {
                assert_eq!(failure.stage, OnPipelineStage::ManifestGate);
                assert_eq!(
                    failure.reason_code,
                    OnPipelineReasonCode::ManifestSignatureInvalid
                );
                assert_eq!(failure.reason_code.as_str(), "manifest_signature_invalid");
            }
            OnPipelineStatus::Ready { .. } => {
                panic!("manifest block should not return ready state")
            }
        }
    }

    #[test]
    fn route_precheck_runs_before_activation_attempt() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let detected = detection_resolution(DetectionState::Detected, None);

        let result = execute_on_pipeline(&detected, &verifier, &manifest, 1_700_000_000, &[]);

        assert!(result.detection_gate.can_activate);
        assert!(result.manifest_gate.is_some());
        assert!(result.route_precheck.is_some());
        assert_eq!(
            result
                .route_precheck
                .as_ref()
                .expect("route precheck should be present")
                .reason_code,
            Some(OnPipelineReasonCode::PrecheckNoProtocolsConfigured)
        );
        match result.status {
            OnPipelineStatus::Blocked { failure } => {
                assert_eq!(failure.stage, OnPipelineStage::RoutePrecheck);
                assert_eq!(
                    failure.reason_code,
                    OnPipelineReasonCode::PrecheckNoProtocolsConfigured
                );
                assert_eq!(
                    failure.reason_code.as_str(),
                    "precheck_no_protocols_configured"
                );
            }
            OnPipelineStatus::Ready { .. } => {
                panic!("precheck block should not return ready state")
            }
        }
    }

    #[test]
    fn successful_chain_returns_ready_contract_for_activation() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let detected = detection_resolution(DetectionState::Detected, None);

        let result = execute_on_pipeline(
            &detected,
            &verifier,
            &manifest,
            1_700_000_000,
            &protocol_order(),
        );

        assert!(result.detection_gate.can_activate);
        assert!(result.manifest_gate.is_some());
        assert!(result.route_precheck.is_some());
        match result.status {
            OnPipelineStatus::Ready {
                manifest_version,
                candidates,
                protocol_order,
            } => {
                assert_eq!(manifest_version, "2026.08.0");
                assert_eq!(candidates.len(), 1);
                assert_eq!(candidates[0].relay_id, "sin-01");
                assert_eq!(protocol_order.len(), 3);
                assert_eq!(protocol_order[0], RouteProtocol::WireGuard);
            }
            OnPipelineStatus::Blocked { .. } => panic!("valid chain should return ready state"),
        }
    }

    #[test]
    fn detection_error_reason_is_normalized_for_ui() {
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let detection_error = detection_resolution(
            DetectionState::Error,
            Some(DetectionReasonCode::PermissionDenied),
        );

        let result = execute_on_pipeline(
            &detection_error,
            &verifier,
            &manifest,
            1_700_000_000,
            &protocol_order(),
        );

        match result.status {
            OnPipelineStatus::Blocked { failure } => {
                assert_eq!(
                    failure.reason_code,
                    OnPipelineReasonCode::DetectionErrorPermissionDenied
                );
                assert_eq!(
                    failure.reason_code.as_str(),
                    "detection_error_permission_denied"
                );
            }
            OnPipelineStatus::Ready { .. } => panic!("error detection should block activation"),
        }
    }
}
