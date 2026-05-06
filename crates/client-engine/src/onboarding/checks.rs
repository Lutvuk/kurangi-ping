use serde::{Deserialize, Serialize};

use crate::onboarding::state_machine::{
    transition_onboarding_state, OnboardingStateMachine, OnboardingStep, OnboardingTransition,
    OnboardingTransitionError,
};
use crate::routing::{RelayHealthSnapshot, RelayHealthStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Warning,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcomeCode {
    PermissionOk,
    PermissionAdminRequired,
    PermissionRouteTableDenied,
    EnvironmentOk,
    EnvironmentNetworkUnavailable,
    EnvironmentUnsupportedOs,
    EnvironmentLowDiskSpace,
    EnvironmentClockSkewDetected,
    RelayReady,
    RelayHealthUnavailable,
    RelayNoHealthyNodes,
    RelayOnlyDegraded,
}

impl CheckOutcomeCode {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::PermissionOk => "permission_ok",
            Self::PermissionAdminRequired => "permission_admin_required",
            Self::PermissionRouteTableDenied => "permission_route_table_denied",
            Self::EnvironmentOk => "environment_ok",
            Self::EnvironmentNetworkUnavailable => "environment_network_unavailable",
            Self::EnvironmentUnsupportedOs => "environment_unsupported_os",
            Self::EnvironmentLowDiskSpace => "environment_low_disk_space",
            Self::EnvironmentClockSkewDetected => "environment_clock_skew_detected",
            Self::RelayReady => "relay_ready",
            Self::RelayHealthUnavailable => "relay_health_unavailable",
            Self::RelayNoHealthyNodes => "relay_no_healthy_nodes",
            Self::RelayOnlyDegraded => "relay_only_degraded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckCondition {
    pub code: CheckOutcomeCode,
    pub is_blocking: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub primary_code: CheckOutcomeCode,
    pub conditions: Vec<CheckCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionCheckInput {
    pub has_admin_privileges: bool,
    pub can_access_route_table: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentCheckInput {
    pub network_available: bool,
    pub supported_os: bool,
    pub disk_space_mb: u64,
    pub minimum_disk_space_mb: u64,
    pub clock_skew_within_tolerance: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayReadinessPolicy {
    pub minimum_healthy_relays: usize,
    pub warn_latency_ceiling_ms: u32,
}

impl Default for RelayReadinessPolicy {
    fn default() -> Self {
        Self {
            minimum_healthy_relays: 1,
            warn_latency_ceiling_ms: 150,
        }
    }
}

pub fn run_permission_check(input: PermissionCheckInput) -> CheckResult {
    if !input.has_admin_privileges {
        return CheckResult {
            status: CheckStatus::Blocked,
            primary_code: CheckOutcomeCode::PermissionAdminRequired,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::PermissionAdminRequired,
                is_blocking: true,
                detail: "Restart app as Administrator to grant route permissions.".to_string(),
            }],
        };
    }

    if !input.can_access_route_table {
        return CheckResult {
            status: CheckStatus::Blocked,
            primary_code: CheckOutcomeCode::PermissionRouteTableDenied,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::PermissionRouteTableDenied,
                is_blocking: true,
                detail: "Route table access denied. Verify firewall/UAC policy then retry."
                    .to_string(),
            }],
        };
    }

    CheckResult {
        status: CheckStatus::Passed,
        primary_code: CheckOutcomeCode::PermissionOk,
        conditions: vec![CheckCondition {
            code: CheckOutcomeCode::PermissionOk,
            is_blocking: false,
            detail: "Permission checks passed.".to_string(),
        }],
    }
}

pub fn run_environment_check(input: EnvironmentCheckInput) -> CheckResult {
    let mut conditions = Vec::new();

    if !input.supported_os {
        conditions.push(CheckCondition {
            code: CheckOutcomeCode::EnvironmentUnsupportedOs,
            is_blocking: true,
            detail: "Current OS build is not supported for onboarding.".to_string(),
        });
    }

    if !input.network_available {
        conditions.push(CheckCondition {
            code: CheckOutcomeCode::EnvironmentNetworkUnavailable,
            is_blocking: true,
            detail: "Internet connection is required for relay probing.".to_string(),
        });
    }

    if input.disk_space_mb < input.minimum_disk_space_mb {
        conditions.push(CheckCondition {
            code: CheckOutcomeCode::EnvironmentLowDiskSpace,
            is_blocking: false,
            detail: format!(
                "Low disk space: {}MB available (recommended >= {}MB).",
                input.disk_space_mb, input.minimum_disk_space_mb
            ),
        });
    }

    if !input.clock_skew_within_tolerance {
        conditions.push(CheckCondition {
            code: CheckOutcomeCode::EnvironmentClockSkewDetected,
            is_blocking: false,
            detail: "System clock drift detected; relay signatures may fail.".to_string(),
        });
    }

    if conditions.is_empty() {
        return CheckResult {
            status: CheckStatus::Passed,
            primary_code: CheckOutcomeCode::EnvironmentOk,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::EnvironmentOk,
                is_blocking: false,
                detail: "Environment checks passed.".to_string(),
            }],
        };
    }

    let primary = conditions[0].code;
    let has_blocking = conditions.iter().any(|condition| condition.is_blocking);
    CheckResult {
        status: if has_blocking {
            CheckStatus::Blocked
        } else {
            CheckStatus::Warning
        },
        primary_code: primary,
        conditions,
    }
}

pub fn run_relay_test_check(
    snapshots: &[RelayHealthSnapshot],
    policy: RelayReadinessPolicy,
) -> CheckResult {
    if snapshots.is_empty() {
        return CheckResult {
            status: CheckStatus::Blocked,
            primary_code: CheckOutcomeCode::RelayHealthUnavailable,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::RelayHealthUnavailable,
                is_blocking: true,
                detail: "Relay health source returned no snapshots.".to_string(),
            }],
        };
    }

    let healthy_relays = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.status == RelayHealthStatus::Ok
                || (snapshot.status == RelayHealthStatus::Warn
                    && snapshot.latency_ms <= policy.warn_latency_ceiling_ms)
        })
        .count();
    let ok_relays = snapshots
        .iter()
        .filter(|snapshot| snapshot.status == RelayHealthStatus::Ok)
        .count();

    if healthy_relays < policy.minimum_healthy_relays {
        return CheckResult {
            status: CheckStatus::Blocked,
            primary_code: CheckOutcomeCode::RelayNoHealthyNodes,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::RelayNoHealthyNodes,
                is_blocking: true,
                detail: format!(
                    "Healthy relays below minimum: {} < {}.",
                    healthy_relays, policy.minimum_healthy_relays
                ),
            }],
        };
    }

    if ok_relays == 0 {
        return CheckResult {
            status: CheckStatus::Warning,
            primary_code: CheckOutcomeCode::RelayOnlyDegraded,
            conditions: vec![CheckCondition {
                code: CheckOutcomeCode::RelayOnlyDegraded,
                is_blocking: false,
                detail: "Relay test passed with degraded nodes only.".to_string(),
            }],
        };
    }

    CheckResult {
        status: CheckStatus::Passed,
        primary_code: CheckOutcomeCode::RelayReady,
        conditions: vec![CheckCondition {
            code: CheckOutcomeCode::RelayReady,
            is_blocking: false,
            detail: "Relay test found healthy nodes.".to_string(),
        }],
    }
}

pub fn apply_check_result_transition(
    machine: &OnboardingStateMachine,
    step: OnboardingStep,
    result: &CheckResult,
) -> Result<OnboardingStateMachine, OnboardingTransitionError> {
    let transition = match result.status {
        CheckStatus::Passed | CheckStatus::Warning => OnboardingTransition::CompleteStep { step },
        CheckStatus::Blocked => OnboardingTransition::BlockStep {
            step,
            reason_code: result.primary_code.as_code().to_string(),
        },
        CheckStatus::Failed => OnboardingTransition::FailStep {
            step,
            reason_code: result.primary_code.as_code().to_string(),
        },
    };

    transition_onboarding_state(machine, transition)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_check_result_transition, run_environment_check, run_permission_check,
        run_relay_test_check, CheckOutcomeCode, CheckStatus, EnvironmentCheckInput,
        PermissionCheckInput, RelayReadinessPolicy,
    };
    use crate::onboarding::state_machine::{
        transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
        OnboardingTransition,
    };
    use crate::routing::{RelayHealthSnapshot, RelayHealthStatus};

    fn snapshot(relay_id: &str, status: RelayHealthStatus, latency_ms: u32) -> RelayHealthSnapshot {
        RelayHealthSnapshot {
            relay_id: relay_id.to_string(),
            status,
            latency_ms,
        }
    }

    #[test]
    fn permission_checks_produce_actionable_outcome_codes() {
        let blocked_admin = run_permission_check(PermissionCheckInput {
            has_admin_privileges: false,
            can_access_route_table: false,
        });
        assert_eq!(blocked_admin.status, CheckStatus::Blocked);
        assert_eq!(
            blocked_admin.primary_code,
            CheckOutcomeCode::PermissionAdminRequired
        );
        assert!(blocked_admin.conditions[0].is_blocking);

        let blocked_route_table = run_permission_check(PermissionCheckInput {
            has_admin_privileges: true,
            can_access_route_table: false,
        });
        assert_eq!(blocked_route_table.status, CheckStatus::Blocked);
        assert_eq!(
            blocked_route_table.primary_code,
            CheckOutcomeCode::PermissionRouteTableDenied
        );

        let ok = run_permission_check(PermissionCheckInput {
            has_admin_privileges: true,
            can_access_route_table: true,
        });
        assert_eq!(ok.status, CheckStatus::Passed);
        assert_eq!(ok.primary_code, CheckOutcomeCode::PermissionOk);
    }

    #[test]
    fn environment_checks_identify_blocking_and_non_blocking_conditions() {
        let blocked = run_environment_check(EnvironmentCheckInput {
            network_available: false,
            supported_os: true,
            disk_space_mb: 128,
            minimum_disk_space_mb: 256,
            clock_skew_within_tolerance: true,
        });
        assert_eq!(blocked.status, CheckStatus::Blocked);
        assert_eq!(
            blocked.primary_code,
            CheckOutcomeCode::EnvironmentNetworkUnavailable
        );
        assert!(blocked.conditions.iter().any(|condition| condition.is_blocking));
        assert!(blocked
            .conditions
            .iter()
            .any(|condition| condition.code == CheckOutcomeCode::EnvironmentLowDiskSpace));

        let warning_only = run_environment_check(EnvironmentCheckInput {
            network_available: true,
            supported_os: true,
            disk_space_mb: 200,
            minimum_disk_space_mb: 256,
            clock_skew_within_tolerance: false,
        });
        assert_eq!(warning_only.status, CheckStatus::Warning);
        assert!(warning_only
            .conditions
            .iter()
            .all(|condition| !condition.is_blocking));
    }

    #[test]
    fn relay_readiness_checks_integrate_with_relay_health_sources() {
        let policy = RelayReadinessPolicy::default();

        let unavailable = run_relay_test_check(&[], policy);
        assert_eq!(unavailable.status, CheckStatus::Blocked);
        assert_eq!(
            unavailable.primary_code,
            CheckOutcomeCode::RelayHealthUnavailable
        );

        let blocked = run_relay_test_check(
            &[
                snapshot("sin-01", RelayHealthStatus::Dead, 999),
                snapshot("nrt-01", RelayHealthStatus::Warn, 400),
            ],
            policy,
        );
        assert_eq!(blocked.status, CheckStatus::Blocked);
        assert_eq!(blocked.primary_code, CheckOutcomeCode::RelayNoHealthyNodes);

        let warning = run_relay_test_check(
            &[snapshot("sin-01", RelayHealthStatus::Warn, 120)],
            policy,
        );
        assert_eq!(warning.status, CheckStatus::Warning);
        assert_eq!(warning.primary_code, CheckOutcomeCode::RelayOnlyDegraded);

        let passed = run_relay_test_check(
            &[
                snapshot("sin-01", RelayHealthStatus::Warn, 120),
                snapshot("nrt-01", RelayHealthStatus::Ok, 48),
            ],
            policy,
        );
        assert_eq!(passed.status, CheckStatus::Passed);
        assert_eq!(passed.primary_code, CheckOutcomeCode::RelayReady);
    }

    #[test]
    fn check_results_feed_onboarding_state_machine_transitions() {
        let machine = OnboardingStateMachine::new();
        let started = transition_onboarding_state(&machine, OnboardingTransition::Begin)
            .expect("begin should move to in progress");

        let after_welcome = apply_check_result_transition(
            &started,
            OnboardingStep::Welcome,
            &run_environment_check(EnvironmentCheckInput {
                network_available: true,
                supported_os: true,
                disk_space_mb: 512,
                minimum_disk_space_mb: 256,
                clock_skew_within_tolerance: true,
            }),
        )
        .expect("passed check should complete step");

        let blocked = apply_check_result_transition(
            &after_welcome,
            OnboardingStep::PermissionCheck,
            &run_permission_check(PermissionCheckInput {
                has_admin_privileges: false,
                can_access_route_table: false,
            }),
        )
        .expect("blocked check should transition to blocked state");

        match blocked.state {
            OnboardingLifecycleState::Blocked {
                blocked_step,
                reason_code,
                ..
            } => {
                assert_eq!(blocked_step, OnboardingStep::PermissionCheck);
                assert_eq!(reason_code, "permission_admin_required");
            }
            other => panic!("expected blocked state, got: {other:?}"),
        }
    }
}
