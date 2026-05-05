use super::RelayHealthStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailoverTriggerConfig {
    pub degraded_grace_polls: u32,
    pub poll_failure_streak_threshold: u32,
    pub hysteresis_window_ms: u64,
    pub dead_relay_triggers_immediately: bool,
}

impl Default for FailoverTriggerConfig {
    fn default() -> Self {
        Self {
            degraded_grace_polls: 3,
            poll_failure_streak_threshold: 2,
            hysteresis_window_ms: 15_000,
            dead_relay_triggers_immediately: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailoverTriggerState {
    pub consecutive_warn_polls: u32,
    pub last_evaluated_status: RelayHealthStatus,
    pub last_failover_at_unix_ms: Option<u64>,
}

impl Default for FailoverTriggerState {
    fn default() -> Self {
        Self {
            consecutive_warn_polls: 0,
            last_evaluated_status: RelayHealthStatus::Ok,
            last_failover_at_unix_ms: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailoverEvaluationInput {
    pub active_relay_status: RelayHealthStatus,
    pub poll_failure_streak: u32,
    pub now_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverReasonCode {
    HealthyNoTrigger,
    DegradedWithinGrace,
    DegradedGraceExceeded,
    DeadRelayDetected,
    PollFailureStreakExceeded,
    HysteresisWindowActive,
}

impl FailoverReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FailoverReasonCode::HealthyNoTrigger => "healthy_no_trigger",
            FailoverReasonCode::DegradedWithinGrace => "degraded_within_grace",
            FailoverReasonCode::DegradedGraceExceeded => "degraded_grace_exceeded",
            FailoverReasonCode::DeadRelayDetected => "dead_relay_detected",
            FailoverReasonCode::PollFailureStreakExceeded => "poll_failure_streak_exceeded",
            FailoverReasonCode::HysteresisWindowActive => "hysteresis_window_active",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailoverDecision {
    pub should_failover: bool,
    pub reason_code: FailoverReasonCode,
    pub consecutive_warn_polls: u32,
    pub hysteresis_remaining_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HysteresisWindowResult {
    pub allow: bool,
    pub remaining_ms: Option<u64>,
}

pub fn apply_hysteresis_window(
    last_failover_at_unix_ms: Option<u64>,
    now_unix_ms: u64,
    hysteresis_window_ms: u64,
) -> HysteresisWindowResult {
    let Some(last_failover_at) = last_failover_at_unix_ms else {
        return HysteresisWindowResult {
            allow: true,
            remaining_ms: None,
        };
    };

    let elapsed_ms = now_unix_ms.saturating_sub(last_failover_at);
    if elapsed_ms >= hysteresis_window_ms {
        HysteresisWindowResult {
            allow: true,
            remaining_ms: Some(0),
        }
    } else {
        HysteresisWindowResult {
            allow: false,
            remaining_ms: Some(hysteresis_window_ms - elapsed_ms),
        }
    }
}

pub fn should_failover(
    state: &mut FailoverTriggerState,
    input: FailoverEvaluationInput,
    config: &FailoverTriggerConfig,
) -> FailoverDecision {
    let window = apply_hysteresis_window(
        state.last_failover_at_unix_ms,
        input.now_unix_ms,
        config.hysteresis_window_ms,
    );
    if !window.allow {
        state.last_evaluated_status = input.active_relay_status;
        state.consecutive_warn_polls = match input.active_relay_status {
            RelayHealthStatus::Warn => state.consecutive_warn_polls.saturating_add(1),
            _ => 0,
        };
        return FailoverDecision {
            should_failover: false,
            reason_code: FailoverReasonCode::HysteresisWindowActive,
            consecutive_warn_polls: state.consecutive_warn_polls,
            hysteresis_remaining_ms: window.remaining_ms,
        };
    }

    let (should_failover, reason_code, consecutive_warn_polls) = match input.active_relay_status {
        RelayHealthStatus::Ok => (false, FailoverReasonCode::HealthyNoTrigger, 0),
        RelayHealthStatus::Warn => {
            let next_warn_count = state.consecutive_warn_polls.saturating_add(1);
            if next_warn_count >= config.degraded_grace_polls {
                (
                    true,
                    FailoverReasonCode::DegradedGraceExceeded,
                    next_warn_count,
                )
            } else {
                (false, FailoverReasonCode::DegradedWithinGrace, next_warn_count)
            }
        }
        RelayHealthStatus::Dead => {
            if config.dead_relay_triggers_immediately {
                (true, FailoverReasonCode::DeadRelayDetected, 0)
            } else {
                (false, FailoverReasonCode::DegradedWithinGrace, 0)
            }
        }
    };

    let (should_failover, reason_code) = if input.poll_failure_streak
        >= config.poll_failure_streak_threshold
    {
        (true, FailoverReasonCode::PollFailureStreakExceeded)
    } else {
        (should_failover, reason_code)
    };

    state.consecutive_warn_polls = if should_failover {
        0
    } else {
        consecutive_warn_polls
    };
    state.last_evaluated_status = input.active_relay_status;
    if should_failover {
        state.last_failover_at_unix_ms = Some(input.now_unix_ms);
    }

    FailoverDecision {
        should_failover,
        reason_code,
        consecutive_warn_polls: state.consecutive_warn_polls,
        hysteresis_remaining_ms: window.remaining_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_hysteresis_window, should_failover, FailoverEvaluationInput, FailoverReasonCode,
        FailoverTriggerConfig, FailoverTriggerState,
    };
    use crate::routing::RelayHealthStatus;

    #[test]
    fn degraded_states_do_not_trigger_immediate_oscillation() {
        let mut state = FailoverTriggerState::default();
        let config = FailoverTriggerConfig {
            degraded_grace_polls: 3,
            ..FailoverTriggerConfig::default()
        };

        let first = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 1_000,
            },
            &config,
        );
        let second = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 2_000,
            },
            &config,
        );
        let third = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 3_000,
            },
            &config,
        );

        assert!(!first.should_failover);
        assert!(!second.should_failover);
        assert_eq!(first.reason_code, FailoverReasonCode::DegradedWithinGrace);
        assert_eq!(second.reason_code, FailoverReasonCode::DegradedWithinGrace);
        assert!(third.should_failover);
        assert_eq!(third.reason_code, FailoverReasonCode::DegradedGraceExceeded);
    }

    #[test]
    fn trigger_conditions_are_deterministic_and_explainable() {
        let mut state = FailoverTriggerState::default();
        let config = FailoverTriggerConfig::default();

        let dead = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Dead,
                poll_failure_streak: 0,
                now_unix_ms: 10_000,
            },
            &config,
        );
        assert!(dead.should_failover);
        assert_eq!(dead.reason_code, FailoverReasonCode::DeadRelayDetected);
        assert_eq!(dead.reason_code.as_str(), "dead_relay_detected");

        let blocked = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Dead,
                poll_failure_streak: 0,
                now_unix_ms: 10_001,
            },
            &config,
        );
        assert!(!blocked.should_failover);
        assert_eq!(blocked.reason_code, FailoverReasonCode::HysteresisWindowActive);
    }

    #[test]
    fn grace_window_is_configurable() {
        let mut state = FailoverTriggerState::default();
        let config = FailoverTriggerConfig {
            degraded_grace_polls: 2,
            hysteresis_window_ms: 5_000,
            ..FailoverTriggerConfig::default()
        };

        let first = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 1_000,
            },
            &config,
        );
        let second = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 2_000,
            },
            &config,
        );

        assert!(!first.should_failover);
        assert!(second.should_failover);
        assert_eq!(second.reason_code, FailoverReasonCode::DegradedGraceExceeded);
    }

    #[test]
    fn edge_cases_rapid_fluctuation_are_covered() {
        let mut state = FailoverTriggerState::default();
        let config = FailoverTriggerConfig {
            degraded_grace_polls: 2,
            hysteresis_window_ms: 4_000,
            ..FailoverTriggerConfig::default()
        };

        let warn_a = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 1_000,
            },
            &config,
        );
        let ok_reset = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Ok,
                poll_failure_streak: 0,
                now_unix_ms: 2_000,
            },
            &config,
        );
        let warn_b = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 3_000,
            },
            &config,
        );
        let warn_c = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 0,
                now_unix_ms: 4_000,
            },
            &config,
        );
        let blocked_by_hysteresis = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Dead,
                poll_failure_streak: 0,
                now_unix_ms: 5_000,
            },
            &config,
        );
        let allowed_after_hysteresis = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Dead,
                poll_failure_streak: 0,
                now_unix_ms: 8_001,
            },
            &config,
        );

        assert!(!warn_a.should_failover);
        assert!(!ok_reset.should_failover);
        assert_eq!(ok_reset.reason_code, FailoverReasonCode::HealthyNoTrigger);
        assert!(!warn_b.should_failover);
        assert!(warn_c.should_failover);
        assert!(!blocked_by_hysteresis.should_failover);
        assert_eq!(
            blocked_by_hysteresis.reason_code,
            FailoverReasonCode::HysteresisWindowActive
        );
        assert!(allowed_after_hysteresis.should_failover);
    }

    #[test]
    fn poll_failure_threshold_can_trigger_even_on_warn() {
        let mut state = FailoverTriggerState::default();
        let config = FailoverTriggerConfig {
            degraded_grace_polls: 4,
            poll_failure_streak_threshold: 2,
            ..FailoverTriggerConfig::default()
        };

        let decision = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: RelayHealthStatus::Warn,
                poll_failure_streak: 2,
                now_unix_ms: 11_000,
            },
            &config,
        );
        assert!(decision.should_failover);
        assert_eq!(
            decision.reason_code,
            FailoverReasonCode::PollFailureStreakExceeded
        );
    }

    #[test]
    fn apply_hysteresis_window_reports_remaining_time() {
        let allowed = apply_hysteresis_window(None, 10_000, 4_000);
        assert!(allowed.allow);
        assert_eq!(allowed.remaining_ms, None);

        let blocked = apply_hysteresis_window(Some(10_000), 11_000, 4_000);
        assert!(!blocked.allow);
        assert_eq!(blocked.remaining_ms, Some(3_000));

        let open = apply_hysteresis_window(Some(10_000), 14_000, 4_000);
        assert!(open.allow);
        assert_eq!(open.remaining_ms, Some(0));
    }
}
