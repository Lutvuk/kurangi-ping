//! Route attempt orchestrator across protocol priority and scored candidates.

use super::{CandidateScore, RetryBudget, RetryMetadata, RetryPolicy, RouteCandidate, RouteProtocol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptFailureReason {
    Timeout,
    HandshakeFailed,
    AuthRejected,
    NetworkUnreachable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStepOutcome {
    Success,
    Failed(AttemptFailureReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteAttemptFailureCode {
    NoEligibleCandidates,
    AllAttemptsFailed,
    RetryBudgetExhausted,
}

impl RouteAttemptFailureCode {
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::NoEligibleCandidates => "ROUTE_NO_ELIGIBLE_CANDIDATES",
            Self::AllAttemptsFailed => "ROUTE_ALL_ATTEMPTS_FAILED",
            Self::RetryBudgetExhausted => "ROUTE_RETRY_BUDGET_EXHAUSTED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRecord {
    pub protocol: RouteProtocol,
    pub candidate: RouteCandidate,
    pub outcome: AttemptStepOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptPlan {
    pub protocol_order: Vec<RouteProtocol>,
    pub eligible_candidates: Vec<RouteCandidate>,
    pub max_attempts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptStatus {
    Connected {
        protocol: RouteProtocol,
        candidate: RouteCandidate,
    },
    Exhausted {
        failure_code: RouteAttemptFailureCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptResult {
    pub plan: AttemptPlan,
    pub status: AttemptStatus,
    pub attempts: Vec<AttemptRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryOrchestrationResult {
    pub final_attempt: AttemptResult,
    pub retry_metadata: Vec<RetryMetadata>,
    pub rounds_executed: u8,
}

pub trait RouteDialer {
    fn attempt(&self, protocol: RouteProtocol, candidate: &RouteCandidate) -> AttemptStepOutcome;
}

pub fn attempt_route<D: RouteDialer>(
    protocol_order: &[RouteProtocol],
    scored_candidates: &[CandidateScore],
    dialer: &D,
) -> AttemptResult {
    let eligible_candidates = scored_candidates
        .iter()
        .filter(|entry| entry.is_eligible())
        .map(|entry| entry.candidate.clone())
        .collect::<Vec<_>>();

    let plan = AttemptPlan {
        protocol_order: protocol_order.to_vec(),
        max_attempts: protocol_order.len() * eligible_candidates.len(),
        eligible_candidates,
    };

    if plan.eligible_candidates.is_empty() {
        return AttemptResult {
            plan,
            status: AttemptStatus::Exhausted {
                failure_code: RouteAttemptFailureCode::NoEligibleCandidates,
            },
            attempts: Vec::new(),
        };
    }

    let protocol_order = plan.protocol_order.clone();
    let eligible_candidates = plan.eligible_candidates.clone();

    let mut attempts = Vec::with_capacity(plan.max_attempts);
    for protocol in protocol_order {
        for candidate in &eligible_candidates {
            let outcome = dialer.attempt(protocol, candidate);
            attempts.push(AttemptRecord {
                protocol,
                candidate: candidate.clone(),
                outcome,
            });

            if outcome == AttemptStepOutcome::Success {
                return AttemptResult {
                    plan,
                    status: AttemptStatus::Connected {
                        protocol,
                        candidate: candidate.clone(),
                    },
                    attempts,
                };
            }
        }
    }

    AttemptResult {
        plan,
        status: AttemptStatus::Exhausted {
            failure_code: RouteAttemptFailureCode::AllAttemptsFailed,
        },
        attempts,
    }
}

pub fn attempt_route_with_retry<D: RouteDialer>(
    protocol_order: &[RouteProtocol],
    scored_candidates: &[CandidateScore],
    retry_policy: RetryPolicy,
    dialer: &D,
) -> RetryOrchestrationResult {
    let mut budget = RetryBudget::new(retry_policy.max_attempts_per_protocol);
    let mut retry_metadata = Vec::new();
    let mut rounds_executed: u8 = 0;
    let mut accumulated_attempts = Vec::new();

    loop {
        rounds_executed = rounds_executed.saturating_add(1);
        let round_result = attempt_route(protocol_order, scored_candidates, dialer);
        let round_plan = round_result.plan.clone();
        accumulated_attempts.extend(round_result.attempts.clone());

        match round_result.status {
            AttemptStatus::Connected { .. } => {
                let final_attempt = AttemptResult {
                    plan: round_plan,
                    status: round_result.status,
                    attempts: accumulated_attempts,
                };
                return RetryOrchestrationResult {
                    final_attempt,
                    retry_metadata,
                    rounds_executed,
                };
            }
            AttemptStatus::Exhausted { failure_code } => {
                if failure_code == RouteAttemptFailureCode::NoEligibleCandidates {
                    let final_attempt = AttemptResult {
                        plan: round_plan,
                        status: AttemptStatus::Exhausted { failure_code },
                        attempts: accumulated_attempts,
                    };
                    return RetryOrchestrationResult {
                        final_attempt,
                        retry_metadata,
                        rounds_executed,
                    };
                }

                let Some(metadata) = budget.consume_retry(
                    retry_policy.base_backoff_ms,
                    retry_policy.max_backoff_ms,
                    failure_code.as_code(),
                ) else {
                    let final_attempt = AttemptResult {
                        plan: round_plan,
                        status: AttemptStatus::Exhausted {
                            failure_code: RouteAttemptFailureCode::RetryBudgetExhausted,
                        },
                        attempts: accumulated_attempts,
                    };
                    return RetryOrchestrationResult {
                        final_attempt,
                        retry_metadata,
                        rounds_executed,
                    };
                };

                retry_metadata.push(metadata);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        attempt_route, attempt_route_with_retry, AttemptFailureReason, AttemptStatus,
        AttemptStepOutcome, RetryOrchestrationResult, RouteDialer, RouteAttemptFailureCode,
    };
    use crate::routing::{
        CandidateDisposition, CandidateScore, RelayHealthStatus, RouteCandidate, RouteProtocol,
        RetryPolicy, ScoreBreakdown,
    };
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    struct TestDialer {
        calls: RefCell<Vec<(RouteProtocol, String)>>,
        success_on: Option<(RouteProtocol, String)>,
    }

    impl TestDialer {
        fn with_success_on(protocol: RouteProtocol, relay_id: &str) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                success_on: Some((protocol, relay_id.to_string())),
            }
        }
    }

    impl RouteDialer for TestDialer {
        fn attempt(&self, protocol: RouteProtocol, candidate: &RouteCandidate) -> AttemptStepOutcome {
            self.calls
                .borrow_mut()
                .push((protocol, candidate.relay_id.clone()));
            match &self.success_on {
                Some((target_protocol, target_relay))
                    if *target_protocol == protocol && *target_relay == candidate.relay_id =>
                {
                    AttemptStepOutcome::Success
                }
                _ => AttemptStepOutcome::Failed(AttemptFailureReason::Timeout),
            }
        }
    }

    fn scored_candidate(relay_id: &str, included: bool) -> CandidateScore {
        CandidateScore {
            candidate: RouteCandidate {
                relay_id: relay_id.to_string(),
                region: "sin".to_string(),
                hostname: format!("{relay_id}.example.net"),
                priority: 1,
            },
            health_status: Some(RelayHealthStatus::Ok),
            latency_ms: Some(45),
            disposition: if included {
                CandidateDisposition::Included
            } else {
                CandidateDisposition::Excluded(crate::routing::ScoreExclusionReason::DeadRelay)
            },
            breakdown: ScoreBreakdown {
                health_points: 100,
                region_bonus_points: 0,
                priority_points: -5,
                latency_penalty_points: 45,
                total_points: 50,
            },
        }
    }

    #[test]
    fn orchestrator_attempts_protocols_in_priority_order() {
        let dialer = TestDialer::with_success_on(RouteProtocol::TcpTls, "nrt-01");
        let scored = vec![scored_candidate("sin-01", true), scored_candidate("nrt-01", true)];
        let protocols = [
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ];

        let result = attempt_route(&protocols, &scored, &dialer);
        assert!(matches!(
            result.status,
            AttemptStatus::Connected {
                protocol: RouteProtocol::TcpTls,
                ..
            }
        ));

        let calls = dialer.calls.borrow().clone();
        assert_eq!(
            calls,
            vec![
                (RouteProtocol::WireGuard, "sin-01".to_string()),
                (RouteProtocol::WireGuard, "nrt-01".to_string()),
                (RouteProtocol::TcpTls, "sin-01".to_string()),
                (RouteProtocol::TcpTls, "nrt-01".to_string()),
            ]
        );
    }

    #[test]
    fn candidate_iteration_follows_scoring_output_order() {
        let dialer = TestDialer::default();
        let scored = vec![scored_candidate("preferred-01", true), scored_candidate("backup-02", true)];
        let protocols = [RouteProtocol::WireGuard];

        let _ = attempt_route(&protocols, &scored, &dialer);
        let calls = dialer.calls.borrow().clone();
        assert_eq!(
            calls,
            vec![
                (RouteProtocol::WireGuard, "preferred-01".to_string()),
                (RouteProtocol::WireGuard, "backup-02".to_string()),
            ]
        );
    }

    #[test]
    fn result_is_normalized_for_state_machine_consumption() {
        let dialer = TestDialer::default();
        let scored = vec![scored_candidate("sin-01", true)];
        let protocols = [
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ];

        let result = attempt_route(&protocols, &scored, &dialer);
        assert_eq!(
            result.status,
            AttemptStatus::Exhausted {
                failure_code: RouteAttemptFailureCode::AllAttemptsFailed,
            }
        );
        assert_eq!(result.attempts.len(), 3);
        assert_eq!(result.plan.max_attempts, 3);
        assert!(result.attempts.iter().all(|entry| matches!(
            entry.outcome,
            AttemptStepOutcome::Failed(_)
        )));
    }

    #[test]
    fn empty_eligible_set_short_circuits_without_loop() {
        let dialer = TestDialer::default();
        let scored = vec![scored_candidate("dead-01", false)];
        let protocols = [
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ];

        let result = attempt_route(&protocols, &scored, &dialer);
        assert_eq!(
            result.status,
            AttemptStatus::Exhausted {
                failure_code: RouteAttemptFailureCode::NoEligibleCandidates,
            }
        );
        assert!(result.attempts.is_empty());
        assert_eq!(result.plan.max_attempts, 0);
    }

    #[test]
    fn retry_controller_stops_at_budget_and_returns_terminal_state() {
        let dialer = TestDialer::default();
        let scored = vec![scored_candidate("sin-01", true)];
        let protocols = [
            RouteProtocol::WireGuard,
            RouteProtocol::TcpTls,
            RouteProtocol::Quic,
        ];
        let retry_policy = RetryPolicy {
            max_attempts_per_protocol: 2,
            base_backoff_ms: 100,
            max_backoff_ms: 250,
        };

        let RetryOrchestrationResult {
            final_attempt,
            retry_metadata,
            rounds_executed,
        } = attempt_route_with_retry(&protocols, &scored, retry_policy, &dialer);

        assert_eq!(rounds_executed, 3);
        assert_eq!(
            final_attempt.status,
            AttemptStatus::Exhausted {
                failure_code: RouteAttemptFailureCode::RetryBudgetExhausted,
            }
        );
        assert_eq!(retry_metadata.len(), 2);
        assert_eq!(retry_metadata[0].delay_ms, 100);
        assert_eq!(retry_metadata[1].delay_ms, 200);
        assert_eq!(final_attempt.attempts.len(), 9);
    }

    #[test]
    fn retry_metadata_is_emitted_with_trigger_code_and_remaining_budget() {
        let dialer = TestDialer::default();
        let scored = vec![scored_candidate("sin-01", true)];
        let protocols = [RouteProtocol::WireGuard];
        let retry_policy = RetryPolicy {
            max_attempts_per_protocol: 1,
            base_backoff_ms: 250,
            max_backoff_ms: 3_000,
        };

        let result = attempt_route_with_retry(&protocols, &scored, retry_policy, &dialer);
        assert_eq!(result.retry_metadata.len(), 1);
        assert_eq!(result.retry_metadata[0].trigger_code, "ROUTE_ALL_ATTEMPTS_FAILED");
        assert_eq!(result.retry_metadata[0].retries_remaining, 0);
    }
}
