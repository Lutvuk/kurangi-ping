use super::{FailoverExecutionState, FailoverSwitchFailureCode, FailoverSwitchResult};
use serde::{Deserialize, Serialize};

pub const FAILOVER_STATE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailoverUiState {
    Stable,
    Switching,
    Recovered,
    Failed,
}

impl FailoverUiState {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailoverUiState::Stable => "stable",
            FailoverUiState::Switching => "switching",
            FailoverUiState::Recovered => "recovered",
            FailoverUiState::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailoverUiReasonCode {
    SwitchSuccessful,
    RelayUnreachable,
    RelayUnstable,
    RelayProbeFailed,
    FailoverCooldown,
    NoAlternativeRelay,
    SwitchPreparationFailed,
    SwitchApplyFailed,
    SwitchRetryExhausted,
    PermissionRequired,
    NetworkTimeout,
    SafeGenericIssue,
}

impl FailoverUiReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailoverUiReasonCode::SwitchSuccessful => "switch_successful",
            FailoverUiReasonCode::RelayUnreachable => "relay_unreachable",
            FailoverUiReasonCode::RelayUnstable => "relay_unstable",
            FailoverUiReasonCode::RelayProbeFailed => "relay_probe_failed",
            FailoverUiReasonCode::FailoverCooldown => "failover_cooldown",
            FailoverUiReasonCode::NoAlternativeRelay => "no_alternative_relay",
            FailoverUiReasonCode::SwitchPreparationFailed => "switch_preparation_failed",
            FailoverUiReasonCode::SwitchApplyFailed => "switch_apply_failed",
            FailoverUiReasonCode::SwitchRetryExhausted => "switch_retry_exhausted",
            FailoverUiReasonCode::PermissionRequired => "permission_required",
            FailoverUiReasonCode::NetworkTimeout => "network_timeout",
            FailoverUiReasonCode::SafeGenericIssue => "safe_generic_issue",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailoverStatePayload {
    pub schema_version: u16,
    pub current_state: String,
    pub previous_relay_id: Option<String>,
    pub next_relay_id: Option<String>,
    pub reason_code: String,
}

pub fn build_failover_state_payload(
    current_state: FailoverUiState,
    previous_relay_id: Option<&str>,
    next_relay_id: Option<&str>,
    raw_reason_code: Option<&str>,
) -> FailoverStatePayload {
    let reason = normalize_reason_code(raw_reason_code);
    FailoverStatePayload {
        schema_version: FAILOVER_STATE_SCHEMA_VERSION,
        current_state: current_state.as_str().to_string(),
        previous_relay_id: previous_relay_id.map(ToString::to_string),
        next_relay_id: next_relay_id.map(ToString::to_string),
        reason_code: reason.as_str().to_string(),
    }
}

pub fn build_failover_state_payload_from_switch_result(
    result: &FailoverSwitchResult,
) -> FailoverStatePayload {
    match result.state {
        FailoverExecutionState::Switched { .. } => build_failover_state_payload(
            FailoverUiState::Recovered,
            Some(&result.active_relay_id_before),
            result.active_relay_id_after.as_deref(),
            Some(FailoverUiReasonCode::SwitchSuccessful.as_str()),
        ),
        FailoverExecutionState::Failed { failure_code, .. } => {
            let reason = map_switch_failure_code(failure_code);
            build_failover_state_payload(
                FailoverUiState::Failed,
                Some(&result.active_relay_id_before),
                result.active_relay_id_after.as_deref(),
                Some(reason.as_str()),
            )
        }
    }
}

pub fn normalize_reason_code(raw_reason_code: Option<&str>) -> FailoverUiReasonCode {
    match raw_reason_code.unwrap_or_default() {
        "dead_relay_detected" => FailoverUiReasonCode::RelayUnreachable,
        "degraded_grace_exceeded" => FailoverUiReasonCode::RelayUnstable,
        "poll_failure_streak_exceeded" => FailoverUiReasonCode::RelayProbeFailed,
        "hysteresis_window_active" => FailoverUiReasonCode::FailoverCooldown,
        "FAILOVER_NO_CANDIDATES" => FailoverUiReasonCode::NoAlternativeRelay,
        "FAILOVER_TEARDOWN_FAILED" => FailoverUiReasonCode::SwitchPreparationFailed,
        "FAILOVER_PROMOTE_FAILED" => FailoverUiReasonCode::SwitchApplyFailed,
        "FAILOVER_RETRY_BUDGET_EXHAUSTED" => FailoverUiReasonCode::SwitchRetryExhausted,
        "FAILOVER_ERR_PERMISSION_DENIED" => FailoverUiReasonCode::PermissionRequired,
        "FAILOVER_ERR_TIMEOUT" => FailoverUiReasonCode::NetworkTimeout,
        "switch_successful" => FailoverUiReasonCode::SwitchSuccessful,
        "relay_unreachable" => FailoverUiReasonCode::RelayUnreachable,
        "relay_unstable" => FailoverUiReasonCode::RelayUnstable,
        "relay_probe_failed" => FailoverUiReasonCode::RelayProbeFailed,
        "failover_cooldown" => FailoverUiReasonCode::FailoverCooldown,
        "no_alternative_relay" => FailoverUiReasonCode::NoAlternativeRelay,
        "switch_preparation_failed" => FailoverUiReasonCode::SwitchPreparationFailed,
        "switch_apply_failed" => FailoverUiReasonCode::SwitchApplyFailed,
        "switch_retry_exhausted" => FailoverUiReasonCode::SwitchRetryExhausted,
        "permission_required" => FailoverUiReasonCode::PermissionRequired,
        "network_timeout" => FailoverUiReasonCode::NetworkTimeout,
        "safe_generic_issue" => FailoverUiReasonCode::SafeGenericIssue,
        _ => FailoverUiReasonCode::SafeGenericIssue,
    }
}

fn map_switch_failure_code(code: FailoverSwitchFailureCode) -> FailoverUiReasonCode {
    match code {
        FailoverSwitchFailureCode::NoCandidates => FailoverUiReasonCode::NoAlternativeRelay,
        FailoverSwitchFailureCode::TeardownActivePathFailed => {
            FailoverUiReasonCode::SwitchPreparationFailed
        }
        FailoverSwitchFailureCode::PromoteNewPathFailed => FailoverUiReasonCode::SwitchApplyFailed,
        FailoverSwitchFailureCode::RetryBudgetExhausted => FailoverUiReasonCode::SwitchRetryExhausted,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_failover_state_payload, build_failover_state_payload_from_switch_result,
        normalize_reason_code, FailoverStatePayload, FailoverUiReasonCode, FailoverUiState,
        FAILOVER_STATE_SCHEMA_VERSION,
    };
    use crate::routing::{
        FailoverExecutionLog, FailoverExecutionState, FailoverFallbackState,
        FailoverSwitchFailureCode, FailoverSwitchResult, RetryMetadata, RouteProtocol,
    };

    #[test]
    fn payload_includes_state_relays_and_reason_code() {
        let payload = build_failover_state_payload(
            FailoverUiState::Switching,
            Some("sin-01"),
            Some("nrt-01"),
            Some("dead_relay_detected"),
        );

        assert_eq!(payload.current_state, "switching");
        assert_eq!(payload.previous_relay_id.as_deref(), Some("sin-01"));
        assert_eq!(payload.next_relay_id.as_deref(), Some("nrt-01"));
        assert_eq!(payload.reason_code, "relay_unreachable");
    }

    #[test]
    fn unknown_reasons_map_to_safe_default_code() {
        let reason = normalize_reason_code(Some("INTERNAL_KERNEL_ERR_0xC000"));
        assert_eq!(reason, FailoverUiReasonCode::SafeGenericIssue);

        let payload =
            build_failover_state_payload(FailoverUiState::Failed, Some("sin-01"), None, Some("???"));
        assert_eq!(payload.reason_code, "safe_generic_issue");
    }

    #[test]
    fn payload_is_serializable_and_versionable() {
        let payload = FailoverStatePayload {
            schema_version: FAILOVER_STATE_SCHEMA_VERSION,
            current_state: "failed".to_string(),
            previous_relay_id: Some("sin-01".to_string()),
            next_relay_id: None,
            reason_code: "switch_retry_exhausted".to_string(),
        };

        let json = serde_json::to_string(&payload).expect("payload should serialize");
        let decoded: FailoverStatePayload =
            serde_json::from_str(&json).expect("payload should deserialize");

        assert_eq!(decoded.schema_version, 1);
        assert_eq!(decoded.reason_code, "switch_retry_exhausted");
    }

    #[test]
    fn mapping_consistency_is_unit_tested() {
        let cases = [
            ("dead_relay_detected", "relay_unreachable"),
            ("degraded_grace_exceeded", "relay_unstable"),
            ("poll_failure_streak_exceeded", "relay_probe_failed"),
            ("hysteresis_window_active", "failover_cooldown"),
            ("FAILOVER_NO_CANDIDATES", "no_alternative_relay"),
            ("FAILOVER_TEARDOWN_FAILED", "switch_preparation_failed"),
            ("FAILOVER_PROMOTE_FAILED", "switch_apply_failed"),
            ("FAILOVER_RETRY_BUDGET_EXHAUSTED", "switch_retry_exhausted"),
            ("FAILOVER_ERR_PERMISSION_DENIED", "permission_required"),
            ("FAILOVER_ERR_TIMEOUT", "network_timeout"),
            ("unknown", "safe_generic_issue"),
        ];

        for (input, expected) in cases {
            let normalized = normalize_reason_code(Some(input));
            assert_eq!(normalized.as_str(), expected);
        }
    }

    #[test]
    fn switch_result_mapping_builds_expected_payload_shape() {
        let switched = FailoverSwitchResult {
            state: FailoverExecutionState::Switched {
                protocol: RouteProtocol::TcpTls,
            },
            active_relay_id_before: "sin-01".to_string(),
            active_relay_id_after: Some("nrt-01".to_string()),
            retry_metadata: vec![RetryMetadata {
                retry_index: 1,
                delay_ms: 250,
                retries_remaining: 0,
                trigger_code: "x".to_string(),
            }],
            attempts: 3,
            logs: vec![FailoverExecutionLog {
                step: "promote_candidate",
                code: "FAILOVER_PROMOTE_OK",
                relay_id: Some("nrt-01".to_string()),
                protocol: Some(RouteProtocol::TcpTls),
            }],
        };
        let switched_payload = build_failover_state_payload_from_switch_result(&switched);
        assert_eq!(switched_payload.current_state, "recovered");
        assert_eq!(switched_payload.reason_code, "switch_successful");

        let failed = FailoverSwitchResult {
            state: FailoverExecutionState::Failed {
                failure_code: FailoverSwitchFailureCode::RetryBudgetExhausted,
                fallback_state: FailoverFallbackState::Disconnected,
            },
            active_relay_id_before: "sin-01".to_string(),
            active_relay_id_after: None,
            retry_metadata: Vec::new(),
            attempts: 10,
            logs: Vec::new(),
        };
        let failed_payload = build_failover_state_payload_from_switch_result(&failed);
        assert_eq!(failed_payload.current_state, "failed");
        assert_eq!(failed_payload.reason_code, "switch_retry_exhausted");
    }
}
