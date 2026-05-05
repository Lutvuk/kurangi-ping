//! Lifecycle timeout policy and deterministic timeout transition handling.

use super::{RoutingState, RoutingStateMachine, RoutingTransition, RoutingTrigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    Arming,
    Disarming,
}

impl LifecyclePhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Arming => "arming",
            Self::Disarming => "disarming",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleTimeoutPolicyConfig {
    pub arming_timeout_ms: u64,
    pub disarming_timeout_ms: u64,
}

impl Default for LifecycleTimeoutPolicyConfig {
    fn default() -> Self {
        Self {
            arming_timeout_ms: 5_000,
            disarming_timeout_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTimeoutReasonCode {
    ArmingTimeout,
    DisarmingTimeout,
}

impl LifecycleTimeoutReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArmingTimeout => "arming_timeout",
            Self::DisarmingTimeout => "disarming_timeout",
        }
    }

    pub const fn as_failure_code(self) -> &'static str {
        match self {
            Self::ArmingTimeout => "LIFECYCLE_ARMING_TIMEOUT",
            Self::DisarmingTimeout => "LIFECYCLE_DISARMING_TIMEOUT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleTimeoutMetrics {
    pub phase: LifecyclePhase,
    pub elapsed_ms: u64,
    pub threshold_ms: u64,
    pub remaining_ms: u64,
    pub overtime_ms: u64,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTimeoutStatus {
    WithinThreshold,
    TimedOut {
        reason_code: LifecycleTimeoutReasonCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTimeoutResult {
    pub state_before: RoutingState,
    pub state_after: RoutingState,
    pub status: LifecycleTimeoutStatus,
    pub transition_trace: Vec<RoutingTransition>,
    pub metrics: LifecycleTimeoutMetrics,
}

pub fn evaluate_lifecycle_timeout(
    machine: &mut RoutingStateMachine,
    phase: LifecyclePhase,
    elapsed_ms: u64,
    policy: &LifecycleTimeoutPolicyConfig,
) -> LifecycleTimeoutResult {
    let state_before = machine.state();
    let threshold_ms = threshold_for_phase(phase, policy);
    let timed_out = elapsed_ms > threshold_ms;
    let remaining_ms = if timed_out {
        0
    } else {
        threshold_ms.saturating_sub(elapsed_ms)
    };
    let overtime_ms = if timed_out {
        elapsed_ms.saturating_sub(threshold_ms)
    } else {
        0
    };

    let metrics = LifecycleTimeoutMetrics {
        phase,
        elapsed_ms,
        threshold_ms,
        remaining_ms,
        overtime_ms,
        timed_out,
    };

    if !timed_out {
        return LifecycleTimeoutResult {
            state_before,
            state_after: machine.state(),
            status: LifecycleTimeoutStatus::WithinThreshold,
            transition_trace: Vec::new(),
            metrics,
        };
    }

    let reason_code = reason_for_phase(phase);
    let transition_trace = force_timeout_failed_state(machine, reason_code.as_failure_code());

    LifecycleTimeoutResult {
        state_before,
        state_after: machine.state(),
        status: LifecycleTimeoutStatus::TimedOut { reason_code },
        transition_trace,
        metrics,
    }
}

fn threshold_for_phase(phase: LifecyclePhase, policy: &LifecycleTimeoutPolicyConfig) -> u64 {
    match phase {
        LifecyclePhase::Arming => policy.arming_timeout_ms,
        LifecyclePhase::Disarming => policy.disarming_timeout_ms,
    }
}

fn reason_for_phase(phase: LifecyclePhase) -> LifecycleTimeoutReasonCode {
    match phase {
        LifecyclePhase::Arming => LifecycleTimeoutReasonCode::ArmingTimeout,
        LifecyclePhase::Disarming => LifecycleTimeoutReasonCode::DisarmingTimeout,
    }
}

fn force_timeout_failed_state(
    machine: &mut RoutingStateMachine,
    failure_code: &str,
) -> Vec<RoutingTransition> {
    let mut trace = Vec::new();
    match machine.state() {
        RoutingState::Connecting => {
            if let Ok(transition) = machine.transition(
                RoutingTrigger::ConnectionAttemptFailed,
                Some(failure_code.to_string()),
            ) {
                trace.push(transition);
            }
        }
        RoutingState::Connected => {
            if let Ok(transition) = machine.transition(RoutingTrigger::HealthDegraded, None) {
                trace.push(transition);
            }
            if let Ok(transition) = machine.transition(
                RoutingTrigger::RelayFailure,
                Some(failure_code.to_string()),
            ) {
                trace.push(transition);
            }
        }
        RoutingState::Degraded => {
            if let Ok(transition) =
                machine.transition(RoutingTrigger::RelayFailure, Some(failure_code.to_string()))
            {
                trace.push(transition);
            }
        }
        RoutingState::Off | RoutingState::Failed => {}
    }
    trace
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_lifecycle_timeout, LifecyclePhase, LifecycleTimeoutPolicyConfig,
        LifecycleTimeoutReasonCode, LifecycleTimeoutStatus,
    };
    use crate::routing::{RoutingState, RoutingStateMachine, RoutingTrigger};

    fn policy() -> LifecycleTimeoutPolicyConfig {
        LifecycleTimeoutPolicyConfig {
            arming_timeout_ms: 2_000,
            disarming_timeout_ms: 1_000,
        }
    }

    fn connecting_machine() -> RoutingStateMachine {
        let mut machine = RoutingStateMachine::new();
        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        machine
    }

    fn connected_machine() -> RoutingStateMachine {
        let mut machine = connecting_machine();
        machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");
        machine
    }

    #[test]
    fn thresholds_are_configurable_per_phase() {
        let mut machine = connecting_machine();
        let configured = LifecycleTimeoutPolicyConfig {
            arming_timeout_ms: 777,
            disarming_timeout_ms: 4_321,
        };

        let arming = evaluate_lifecycle_timeout(
            &mut machine,
            LifecyclePhase::Arming,
            777,
            &configured,
        );
        assert_eq!(arming.metrics.threshold_ms, 777);
        assert_eq!(arming.status, LifecycleTimeoutStatus::WithinThreshold);

        let mut machine = connected_machine();
        let disarming = evaluate_lifecycle_timeout(
            &mut machine,
            LifecyclePhase::Disarming,
            4_321,
            &configured,
        );
        assert_eq!(disarming.metrics.threshold_ms, 4_321);
        assert_eq!(disarming.status, LifecycleTimeoutStatus::WithinThreshold);
    }

    #[test]
    fn arming_timeout_transitions_to_stable_failed_with_reason() {
        let mut machine = connecting_machine();

        let result = evaluate_lifecycle_timeout(&mut machine, LifecyclePhase::Arming, 2_001, &policy());

        assert_eq!(result.state_before, RoutingState::Connecting);
        assert_eq!(result.state_after, RoutingState::Failed);
        assert_eq!(
            result.status,
            LifecycleTimeoutStatus::TimedOut {
                reason_code: LifecycleTimeoutReasonCode::ArmingTimeout
            }
        );
        assert_eq!(machine.state(), RoutingState::Failed);
        assert_eq!(
            machine.ui_snapshot().failure_code.as_deref(),
            Some("LIFECYCLE_ARMING_TIMEOUT")
        );
    }

    #[test]
    fn disarming_timeout_transitions_to_stable_failed_with_reason() {
        let mut machine = connected_machine();

        let result = evaluate_lifecycle_timeout(
            &mut machine,
            LifecyclePhase::Disarming,
            1_500,
            &policy(),
        );

        assert_eq!(result.state_before, RoutingState::Connected);
        assert_eq!(result.state_after, RoutingState::Failed);
        assert_eq!(
            result.status,
            LifecycleTimeoutStatus::TimedOut {
                reason_code: LifecycleTimeoutReasonCode::DisarmingTimeout
            }
        );
        assert_eq!(machine.state(), RoutingState::Failed);
        assert_eq!(
            machine.ui_snapshot().failure_code.as_deref(),
            Some("LIFECYCLE_DISARMING_TIMEOUT")
        );
    }

    #[test]
    fn timeout_metrics_are_emitted_for_telemetry_consumers() {
        let mut machine = connecting_machine();

        let result = evaluate_lifecycle_timeout(&mut machine, LifecyclePhase::Arming, 1_200, &policy());

        assert!(!result.metrics.timed_out);
        assert_eq!(result.metrics.phase, LifecyclePhase::Arming);
        assert_eq!(result.metrics.elapsed_ms, 1_200);
        assert_eq!(result.metrics.threshold_ms, 2_000);
        assert_eq!(result.metrics.remaining_ms, 800);
        assert_eq!(result.metrics.overtime_ms, 0);
    }

    #[test]
    fn behavior_is_deterministic_at_timeout_boundary() {
        let mut machine = connecting_machine();

        let at_boundary = evaluate_lifecycle_timeout(
            &mut machine,
            LifecyclePhase::Arming,
            2_000,
            &policy(),
        );
        assert_eq!(at_boundary.status, LifecycleTimeoutStatus::WithinThreshold);
        assert_eq!(at_boundary.state_after, RoutingState::Connecting);

        let over_boundary = evaluate_lifecycle_timeout(
            &mut machine,
            LifecyclePhase::Arming,
            2_001,
            &policy(),
        );
        assert!(matches!(
            over_boundary.status,
            LifecycleTimeoutStatus::TimedOut { .. }
        ));
        assert_eq!(over_boundary.metrics.overtime_ms, 1);
    }
}
