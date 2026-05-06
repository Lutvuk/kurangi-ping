use super::{
    next_retry_delay, AttemptFailureReason, AttemptStepOutcome, RetryBudget, RetryMetadata,
    RetryPolicy, RouteCandidate, RouteProtocol,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverSwitchErrorCode {
    PermissionDenied,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverSwitchError {
    pub code: FailoverSwitchErrorCode,
    pub message: String,
}

impl FailoverSwitchError {
    pub fn new(code: FailoverSwitchErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverExecutionContext {
    pub active_relay_id: String,
    pub candidates: Vec<RouteCandidate>,
    pub protocol_order: Vec<RouteProtocol>,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverSwitchFailureCode {
    NoCandidates,
    TeardownActivePathFailed,
    PromoteNewPathFailed,
    RetryBudgetExhausted,
}

impl FailoverSwitchFailureCode {
    pub fn as_code(&self) -> &'static str {
        match self {
            FailoverSwitchFailureCode::NoCandidates => "FAILOVER_NO_CANDIDATES",
            FailoverSwitchFailureCode::TeardownActivePathFailed => "FAILOVER_TEARDOWN_FAILED",
            FailoverSwitchFailureCode::PromoteNewPathFailed => "FAILOVER_PROMOTE_FAILED",
            FailoverSwitchFailureCode::RetryBudgetExhausted => "FAILOVER_RETRY_BUDGET_EXHAUSTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverFallbackState {
    OldPathPreserved,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverExecutionState {
    Switched {
        protocol: RouteProtocol,
    },
    Failed {
        failure_code: FailoverSwitchFailureCode,
        fallback_state: FailoverFallbackState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverExecutionLog {
    pub step: &'static str,
    pub code: &'static str,
    pub relay_id: Option<String>,
    pub protocol: Option<RouteProtocol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailoverSwitchResult {
    pub state: FailoverExecutionState,
    pub active_relay_id_before: String,
    pub active_relay_id_after: Option<String>,
    pub retry_metadata: Vec<RetryMetadata>,
    pub attempts: usize,
    pub logs: Vec<FailoverExecutionLog>,
}

pub trait ActiveRelaySwitchAdapter {
    fn teardown_active_path(&mut self, active_relay_id: &str) -> Result<(), FailoverSwitchError>;
    fn establish_candidate_path(
        &mut self,
        protocol: RouteProtocol,
        candidate: &RouteCandidate,
    ) -> AttemptStepOutcome;
    fn promote_candidate_path(
        &mut self,
        candidate: &RouteCandidate,
        protocol: RouteProtocol,
    ) -> Result<(), FailoverSwitchError>;
    fn rollback_candidate_path(&mut self, candidate: &RouteCandidate);
    fn wait_backoff(&mut self, delay_ms: u64);
}

pub fn switch_active_relay<A: ActiveRelaySwitchAdapter>(
    context: &FailoverExecutionContext,
    adapter: &mut A,
) -> FailoverSwitchResult {
    let mut logs = Vec::new();

    if context.candidates.is_empty() || context.protocol_order.is_empty() {
        logs.push(FailoverExecutionLog {
            step: "precheck",
            code: FailoverSwitchFailureCode::NoCandidates.as_code(),
            relay_id: None,
            protocol: None,
        });
        return FailoverSwitchResult {
            state: FailoverExecutionState::Failed {
                failure_code: FailoverSwitchFailureCode::NoCandidates,
                fallback_state: FailoverFallbackState::OldPathPreserved,
            },
            active_relay_id_before: context.active_relay_id.clone(),
            active_relay_id_after: Some(context.active_relay_id.clone()),
            retry_metadata: Vec::new(),
            attempts: 0,
            logs,
        };
    }

    logs.push(FailoverExecutionLog {
        step: "teardown_active",
        code: "FAILOVER_TEARDOWN_STARTED",
        relay_id: Some(context.active_relay_id.clone()),
        protocol: None,
    });

    if let Err(error) = adapter.teardown_active_path(&context.active_relay_id) {
        logs.push(FailoverExecutionLog {
            step: "teardown_active",
            code: map_switch_error_to_log_code(error.code),
            relay_id: Some(context.active_relay_id.clone()),
            protocol: None,
        });
        return FailoverSwitchResult {
            state: FailoverExecutionState::Failed {
                failure_code: FailoverSwitchFailureCode::TeardownActivePathFailed,
                fallback_state: FailoverFallbackState::OldPathPreserved,
            },
            active_relay_id_before: context.active_relay_id.clone(),
            active_relay_id_after: Some(context.active_relay_id.clone()),
            retry_metadata: Vec::new(),
            attempts: 0,
            logs,
        };
    }

    logs.push(FailoverExecutionLog {
        step: "teardown_active",
        code: "FAILOVER_TEARDOWN_OK",
        relay_id: Some(context.active_relay_id.clone()),
        protocol: None,
    });

    let mut retry_budget = RetryBudget::new(context.retry_policy.max_attempts_per_protocol);
    let mut retry_metadata = Vec::new();
    let mut attempts = 0_usize;

    loop {
        for protocol in &context.protocol_order {
            for candidate in &context.candidates {
                attempts = attempts.saturating_add(1);
                let outcome = adapter.establish_candidate_path(*protocol, candidate);
                logs.push(FailoverExecutionLog {
                    step: "establish_candidate",
                    code: map_attempt_outcome_to_log_code(outcome),
                    relay_id: Some(candidate.relay_id.clone()),
                    protocol: Some(*protocol),
                });

                if outcome == AttemptStepOutcome::Success {
                    let promote_result = adapter.promote_candidate_path(candidate, *protocol);
                    match promote_result {
                        Ok(()) => {
                            logs.push(FailoverExecutionLog {
                                step: "promote_candidate",
                                code: "FAILOVER_PROMOTE_OK",
                                relay_id: Some(candidate.relay_id.clone()),
                                protocol: Some(*protocol),
                            });
                            return FailoverSwitchResult {
                                state: FailoverExecutionState::Switched {
                                    protocol: *protocol,
                                },
                                active_relay_id_before: context.active_relay_id.clone(),
                                active_relay_id_after: Some(candidate.relay_id.clone()),
                                retry_metadata,
                                attempts,
                                logs,
                            };
                        }
                        Err(error) => {
                            adapter.rollback_candidate_path(candidate);
                            logs.push(FailoverExecutionLog {
                                step: "promote_candidate",
                                code: map_switch_error_to_log_code(error.code),
                                relay_id: Some(candidate.relay_id.clone()),
                                protocol: Some(*protocol),
                            });
                            logs.push(FailoverExecutionLog {
                                step: "rollback_candidate",
                                code: "FAILOVER_ROLLBACK_DONE",
                                relay_id: Some(candidate.relay_id.clone()),
                                protocol: Some(*protocol),
                            });
                            return FailoverSwitchResult {
                                state: FailoverExecutionState::Failed {
                                    failure_code: FailoverSwitchFailureCode::PromoteNewPathFailed,
                                    fallback_state: FailoverFallbackState::Disconnected,
                                },
                                active_relay_id_before: context.active_relay_id.clone(),
                                active_relay_id_after: None,
                                retry_metadata,
                                attempts,
                                logs,
                            };
                        }
                    }
                }
            }
        }

        let Some(metadata) = retry_budget.consume_retry(
            context.retry_policy.base_backoff_ms,
            context.retry_policy.max_backoff_ms,
            FailoverSwitchFailureCode::RetryBudgetExhausted.as_code(),
        ) else {
            logs.push(FailoverExecutionLog {
                step: "retry_budget",
                code: FailoverSwitchFailureCode::RetryBudgetExhausted.as_code(),
                relay_id: None,
                protocol: None,
            });
            return FailoverSwitchResult {
                state: FailoverExecutionState::Failed {
                    failure_code: FailoverSwitchFailureCode::RetryBudgetExhausted,
                    fallback_state: FailoverFallbackState::Disconnected,
                },
                active_relay_id_before: context.active_relay_id.clone(),
                active_relay_id_after: None,
                retry_metadata,
                attempts,
                logs,
            };
        };

        let delay_ms = next_retry_delay(
            context.retry_policy.base_backoff_ms,
            context.retry_policy.max_backoff_ms,
            metadata.retry_index,
        );
        adapter.wait_backoff(delay_ms);
        logs.push(FailoverExecutionLog {
            step: "retry_backoff",
            code: "FAILOVER_RETRY_SCHEDULED",
            relay_id: None,
            protocol: None,
        });
        retry_metadata.push(metadata);
    }
}

fn map_switch_error_to_log_code(error_code: FailoverSwitchErrorCode) -> &'static str {
    match error_code {
        FailoverSwitchErrorCode::PermissionDenied => "FAILOVER_ERR_PERMISSION_DENIED",
        FailoverSwitchErrorCode::Timeout => "FAILOVER_ERR_TIMEOUT",
        FailoverSwitchErrorCode::Unknown => "FAILOVER_ERR_UNKNOWN",
    }
}

fn map_attempt_outcome_to_log_code(outcome: AttemptStepOutcome) -> &'static str {
    match outcome {
        AttemptStepOutcome::Success => "FAILOVER_ESTABLISH_OK",
        AttemptStepOutcome::Failed(AttemptFailureReason::Timeout) => "FAILOVER_ESTABLISH_TIMEOUT",
        AttemptStepOutcome::Failed(AttemptFailureReason::HandshakeFailed) => {
            "FAILOVER_ESTABLISH_HANDSHAKE_FAILED"
        }
        AttemptStepOutcome::Failed(AttemptFailureReason::AuthRejected) => {
            "FAILOVER_ESTABLISH_AUTH_REJECTED"
        }
        AttemptStepOutcome::Failed(AttemptFailureReason::NetworkUnreachable) => {
            "FAILOVER_ESTABLISH_NETWORK_UNREACHABLE"
        }
        AttemptStepOutcome::Failed(AttemptFailureReason::Unknown) => "FAILOVER_ESTABLISH_UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        switch_active_relay, ActiveRelaySwitchAdapter, FailoverExecutionContext,
        FailoverExecutionState, FailoverFallbackState, FailoverSwitchError,
        FailoverSwitchErrorCode, FailoverSwitchFailureCode,
    };
    use crate::routing::{
        AttemptFailureReason, AttemptStepOutcome, RetryPolicy, RouteCandidate, RouteProtocol,
    };
    use std::collections::VecDeque;

    #[derive(Default)]
    struct MockSwitchAdapter {
        operations: Vec<String>,
        establish_outcomes: VecDeque<AttemptStepOutcome>,
        teardown_result: Option<Result<(), FailoverSwitchError>>,
        promote_result: Option<Result<(), FailoverSwitchError>>,
        backoff_delays: Vec<u64>,
    }

    impl ActiveRelaySwitchAdapter for MockSwitchAdapter {
        fn teardown_active_path(
            &mut self,
            active_relay_id: &str,
        ) -> Result<(), FailoverSwitchError> {
            self.operations.push(format!("teardown:{active_relay_id}"));
            self.teardown_result.clone().unwrap_or(Ok(()))
        }

        fn establish_candidate_path(
            &mut self,
            protocol: RouteProtocol,
            candidate: &RouteCandidate,
        ) -> AttemptStepOutcome {
            self.operations
                .push(format!("establish:{:?}:{}", protocol, candidate.relay_id));
            self.establish_outcomes
                .pop_front()
                .unwrap_or(AttemptStepOutcome::Failed(AttemptFailureReason::Timeout))
        }

        fn promote_candidate_path(
            &mut self,
            candidate: &RouteCandidate,
            protocol: RouteProtocol,
        ) -> Result<(), FailoverSwitchError> {
            self.operations
                .push(format!("promote:{:?}:{}", protocol, candidate.relay_id));
            self.promote_result.clone().unwrap_or(Ok(()))
        }

        fn rollback_candidate_path(&mut self, candidate: &RouteCandidate) {
            self.operations
                .push(format!("rollback:{}", candidate.relay_id));
        }

        fn wait_backoff(&mut self, delay_ms: u64) {
            self.backoff_delays.push(delay_ms);
            self.operations.push(format!("backoff:{delay_ms}"));
        }
    }

    fn context() -> FailoverExecutionContext {
        FailoverExecutionContext {
            active_relay_id: "sin-01".to_string(),
            candidates: vec![RouteCandidate {
                relay_id: "nrt-01".to_string(),
                region: "nrt".to_string(),
                hostname: "sensitive-host.internal.example".to_string(),
                priority: 1,
            }],
            protocol_order: vec![
                RouteProtocol::WireGuard,
                RouteProtocol::TcpTls,
                RouteProtocol::Quic,
            ],
            retry_policy: RetryPolicy {
                max_attempts_per_protocol: 2,
                base_backoff_ms: 250,
                max_backoff_ms: 3_000,
            },
        }
    }

    #[test]
    fn switch_sequence_tears_down_old_path_before_promote_new_route() {
        let mut adapter = MockSwitchAdapter {
            establish_outcomes: VecDeque::from(vec![AttemptStepOutcome::Success]),
            ..MockSwitchAdapter::default()
        };

        let result = switch_active_relay(&context(), &mut adapter);
        assert!(matches!(
            result.state,
            FailoverExecutionState::Switched {
                protocol: RouteProtocol::WireGuard
            }
        ));

        assert_eq!(
            adapter.operations,
            vec![
                "teardown:sin-01".to_string(),
                "establish:WireGuard:nrt-01".to_string(),
                "promote:WireGuard:nrt-01".to_string()
            ]
        );
    }

    #[test]
    fn failover_switch_respects_retry_budget_and_backoff() {
        let mut adapter = MockSwitchAdapter::default();
        let result = switch_active_relay(&context(), &mut adapter);

        assert!(matches!(
            result.state,
            FailoverExecutionState::Failed {
                failure_code: FailoverSwitchFailureCode::RetryBudgetExhausted,
                fallback_state: FailoverFallbackState::Disconnected
            }
        ));
        assert_eq!(result.retry_metadata.len(), 2);
        assert_eq!(result.retry_metadata[0].delay_ms, 250);
        assert_eq!(result.retry_metadata[1].delay_ms, 500);
        assert_eq!(adapter.backoff_delays, vec![250, 500]);
    }

    #[test]
    fn partial_switch_failure_returns_deterministic_terminal_state() {
        let mut adapter = MockSwitchAdapter {
            establish_outcomes: VecDeque::from(vec![AttemptStepOutcome::Success]),
            promote_result: Some(Err(FailoverSwitchError::new(
                FailoverSwitchErrorCode::PermissionDenied,
                "access denied to route promotion",
            ))),
            ..MockSwitchAdapter::default()
        };

        let result = switch_active_relay(&context(), &mut adapter);
        assert!(matches!(
            result.state,
            FailoverExecutionState::Failed {
                failure_code: FailoverSwitchFailureCode::PromoteNewPathFailed,
                fallback_state: FailoverFallbackState::Disconnected
            }
        ));
        assert_eq!(result.active_relay_id_after, None);
        assert!(adapter.operations.contains(&"rollback:nrt-01".to_string()));
    }

    #[test]
    fn execution_logs_produce_non_sensitive_diagnostics() {
        let mut adapter = MockSwitchAdapter {
            establish_outcomes: VecDeque::from(vec![AttemptStepOutcome::Success]),
            ..MockSwitchAdapter::default()
        };
        let result = switch_active_relay(&context(), &mut adapter);

        let flattened = result
            .logs
            .iter()
            .map(|entry| format!("{}:{}:{:?}", entry.step, entry.code, entry.relay_id))
            .collect::<Vec<_>>()
            .join("|");

        assert!(!flattened.contains("sensitive-host.internal.example"));
        assert!(flattened.contains("FAILOVER_TEARDOWN_STARTED"));
        assert!(flattened.contains("FAILOVER_PROMOTE_OK"));
    }
}
