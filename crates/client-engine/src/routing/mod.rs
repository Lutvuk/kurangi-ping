//! Routing and manifest verification boundaries.

mod config;
mod detection_gate;
mod failover_executor;
mod failover_state;
mod failover_trigger;
mod health_classification;
mod health_scheduler;
mod integration;
mod manifest_gate;
mod orchestrator;
mod policy;
mod retry;
mod scoring;
mod state_machine;
mod toggle_orchestrator;

pub use config::{RoutingConfig, RoutingConfigError};
pub use detection_gate::{can_activate_routing, DetectionGateDecision, DetectionGateReasonCode};
pub use failover_executor::{
    switch_active_relay, ActiveRelaySwitchAdapter, FailoverExecutionContext, FailoverExecutionLog,
    FailoverExecutionState, FailoverFallbackState, FailoverSwitchError, FailoverSwitchErrorCode,
    FailoverSwitchFailureCode, FailoverSwitchResult,
};
pub use failover_state::{
    build_failover_state_payload, build_failover_state_payload_from_switch_result,
    normalize_reason_code, FailoverStatePayload, FailoverUiReasonCode, FailoverUiState,
    FAILOVER_STATE_SCHEMA_VERSION,
};
pub use failover_trigger::{
    apply_hysteresis_window, should_failover, FailoverDecision, FailoverEvaluationInput,
    FailoverReasonCode, FailoverTriggerConfig, FailoverTriggerState, HysteresisWindowResult,
};
pub use health_classification::{
    classify_relay_health, HealthClassificationThresholds, RelayHealthMetrics, RelayHealthStatus,
};
pub use health_scheduler::{
    start_health_polling, stop_health_polling, FailoverEvaluatorSink, HealthPollCompletion,
    HealthPollDecision, HealthPollError, HealthPollErrorCode, HealthPollSkipReason,
    HealthPollingConfig, HealthPollingContext, HealthPollingLease, HealthPollingScheduler,
    RelayHealthPoller, RELAY_HEALTH_ENDPOINT_PATH,
};
pub use integration::{
    close_route_session, start_route_session, transition_with_session_hooks,
    RouteSessionPersistence, SessionHookResult, SessionHookStatus,
};
pub use manifest_gate::{
    verify_manifest_or_fail, ManifestFailureCode, ManifestGateResult, RelayManifestDto,
    RelayNodeDto, RouteCandidate,
};
pub use policy::{
    ProtocolPriority, RetryPolicy, RouteProtocol, RoutingPolicy, DEFAULT_PROTOCOL_ORDER,
};
pub use orchestrator::{
    attempt_route, attempt_route_with_retry, AttemptFailureReason, AttemptPlan, AttemptRecord,
    AttemptResult, AttemptStatus, AttemptStepOutcome, RetryOrchestrationResult,
    RouteAttemptFailureCode, RouteDialer,
};
pub use retry::{next_retry_delay, RetryBudget, RetryMetadata};
pub use scoring::{
    score_candidates, CandidateDisposition, CandidateScore, RelayHealthSnapshot,
    RelayScoringConfig, ScoreBreakdown, ScoreExclusionReason, ScoringWeights,
};
pub use state_machine::{
    IllegalTransitionError, RoutingState, RoutingStateMachine, RoutingStateView,
    RoutingTransition, RoutingTrigger,
};
pub use toggle_orchestrator::{
    handle_toggle_command, ToggleCommand, ToggleCommandResult, ToggleOrchestrator,
    ToggleRejectionKind, ToggleResultCode,
};

use crate::security::signature::ManifestSignatureVerifier;
use crate::detection::state_resolver::DetectionResolution;

/// Public routing service API.
#[derive(Debug, Clone)]
pub struct RoutingService {
    active: bool,
    policy: RoutingPolicy,
}

impl RoutingService {
    pub fn new() -> Self {
        Self {
            active: false,
            policy: RoutingPolicy::default(),
        }
    }

    pub fn from_config(config: RoutingConfig) -> Result<Self, RoutingConfigError> {
        let policy = config.into_policy()?;
        Ok(Self {
            active: false,
            policy,
        })
    }

    pub fn state(&self) -> &'static str {
        if self.active { "enabled" } else { "disabled" }
    }

    pub fn policy(&self) -> RoutingPolicy {
        self.policy
    }

    pub fn protocol_order(&self) -> [RouteProtocol; 3] {
        self.policy.protocol_order()
    }

    pub fn plan_activation<V: ManifestSignatureVerifier>(
        &self,
        verifier: &V,
        manifest: &RelayManifestDto,
        now_unix_s: u64,
    ) -> RoutePlan {
        let manifest_gate = verify_manifest_or_fail(verifier, manifest, now_unix_s);
        let (manifest_valid, attempted_protocols, route_candidates, failure_code) = match &manifest_gate {
            ManifestGateResult::Passed { candidates, .. } => (
                true,
                self.protocol_order().to_vec(),
                candidates.clone(),
                None,
            ),
            ManifestGateResult::Blocked { failure_code } => {
                (false, Vec::new(), Vec::new(), Some(*failure_code))
            }
        };

        RoutePlan {
            manifest_valid,
            manifest_gate,
            attempted_protocols,
            route_candidates,
            failure_code,
        }
    }

    pub fn plan_activation_with_detection<V: ManifestSignatureVerifier>(
        &self,
        detection_resolution: &DetectionResolution,
        verifier: &V,
        manifest: &RelayManifestDto,
        now_unix_s: u64,
    ) -> RouteActivationGateResult {
        let detection_gate = can_activate_routing(detection_resolution);
        if !detection_gate.can_activate {
            return RouteActivationGateResult::Denied(detection_gate);
        }

        RouteActivationGateResult::Allowed(self.plan_activation(verifier, manifest, now_unix_s))
    }
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Dry-run activation plan to support orchestration tests.
#[derive(Debug, Clone)]
pub struct RoutePlan {
    pub manifest_valid: bool,
    pub manifest_gate: ManifestGateResult,
    pub attempted_protocols: Vec<RouteProtocol>,
    pub route_candidates: Vec<RouteCandidate>,
    pub failure_code: Option<ManifestFailureCode>,
}

#[derive(Debug, Clone)]
pub enum RouteActivationGateResult {
    Denied(DetectionGateDecision),
    Allowed(RoutePlan),
}

#[cfg(test)]
mod tests {
    use super::{
        DetectionGateReasonCode, ManifestFailureCode, ManifestGateResult, RelayManifestDto,
        RelayNodeDto, RouteActivationGateResult, RouteProtocol, RoutingConfig, RoutingConfigError,
        RoutingService, DEFAULT_PROTOCOL_ORDER,
    };
    use crate::detection::state_resolver::{
        DetectionMetadata, DetectionReasonCode, DetectionResolution, DetectionState,
    };
    use crate::security::signature::AllowlistSignatureVerifier;

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

    #[test]
    fn service_defaults_to_architecture_protocol_order() {
        let service = RoutingService::new();
        assert_eq!(service.protocol_order(), DEFAULT_PROTOCOL_ORDER);
        assert_eq!(service.state(), "disabled");
    }

    #[test]
    fn service_from_config_rejects_invalid_priority_sets() {
        let invalid_config = RoutingConfig {
            protocol_priority: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::WireGuard,
                RouteProtocol::Quic,
            ],
            ..RoutingConfig::default()
        };

        let err = RoutingService::from_config(invalid_config).expect_err("config must be invalid");
        assert_eq!(
            err,
            RoutingConfigError::DuplicateProtocol {
                protocol: RouteProtocol::WireGuard
            }
        );
    }

    #[test]
    fn plan_activation_uses_policy_protocol_order() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);

        let plan = service.plan_activation(&verifier, &manifest, 1_700_000_000);
        assert!(plan.manifest_valid);
        assert!(matches!(plan.manifest_gate, ManifestGateResult::Passed { .. }));
        assert_eq!(plan.attempted_protocols, DEFAULT_PROTOCOL_ORDER.to_vec());
        assert_eq!(plan.route_candidates.len(), 1);
        assert_eq!(plan.failure_code, None);
    }

    #[test]
    fn plan_activation_blocks_when_manifest_invalid() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("bad-signature", 1_800_000_000);

        let plan = service.plan_activation(&verifier, &manifest, 1_700_000_000);
        assert!(!plan.manifest_valid);
        assert!(plan.attempted_protocols.is_empty());
        assert!(plan.route_candidates.is_empty());
        assert_eq!(plan.failure_code, Some(ManifestFailureCode::SignatureInvalid));
    }

    #[test]
    fn plan_activation_with_detection_denies_when_state_not_detected() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let stale = detection_resolution(
            DetectionState::Stale,
            Some(DetectionReasonCode::StaleWindowExceeded),
        );

        let gated =
            service.plan_activation_with_detection(&stale, &verifier, &manifest, 1_700_000_000);
        match gated {
            RouteActivationGateResult::Denied(decision) => {
                assert_eq!(
                    decision.reason_code,
                    Some(DetectionGateReasonCode::DetectionStale)
                );
            }
            RouteActivationGateResult::Allowed(_) => {
                panic!("stale detection must deny routing activation")
            }
        }
    }

    #[test]
    fn plan_activation_with_detection_allows_and_returns_route_plan() {
        let service = RoutingService::new();
        let verifier = AllowlistSignatureVerifier::new(vec!["known-good".to_string()]);
        let manifest = sample_manifest("known-good", 1_800_000_000);
        let detected = detection_resolution(DetectionState::Detected, None);

        let gated = service.plan_activation_with_detection(
            &detected,
            &verifier,
            &manifest,
            1_700_000_000,
        );
        match gated {
            RouteActivationGateResult::Allowed(plan) => {
                assert!(plan.manifest_valid);
                assert!(!plan.route_candidates.is_empty());
            }
            RouteActivationGateResult::Denied(_) => {
                panic!("detected state should allow routing activation gate")
            }
        }
    }
}

