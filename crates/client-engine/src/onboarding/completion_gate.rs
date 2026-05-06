use crate::onboarding::state_machine::{
    transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
    OnboardingTransition, OnboardingTransitionError,
};
use crate::routing::{RoutingTransition, RoutingTrigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingCompletionOutcome {
    Completed,
    Pending,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingCompletionReasonCode {
    CompletedOnFirstSuccessfulConnect,
    AwaitingSuccessfulConnect,
    NotAtFirstConnectStep,
    RoutingLifecycleDidNotReachConnected,
    ConnectAttemptFailed,
    AlreadyCompleted,
}

impl OnboardingCompletionReasonCode {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::CompletedOnFirstSuccessfulConnect => "completed_on_first_successful_connect",
            Self::AwaitingSuccessfulConnect => "awaiting_successful_connect",
            Self::NotAtFirstConnectStep => "not_at_first_connect_step",
            Self::RoutingLifecycleDidNotReachConnected => {
                "routing_lifecycle_did_not_reach_connected"
            }
            Self::ConnectAttemptFailed => "connect_attempt_failed",
            Self::AlreadyCompleted => "already_completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingCompletionDecision {
    pub outcome: OnboardingCompletionOutcome,
    pub reason_code: OnboardingCompletionReasonCode,
    pub next_state_machine: OnboardingStateMachine,
    pub should_emit_completed_event: bool,
}

pub fn evaluate_onboarding_completion(
    machine: &OnboardingStateMachine,
    routing_transition: Option<&RoutingTransition>,
) -> Result<OnboardingCompletionDecision, OnboardingTransitionError> {
    if matches!(machine.state, OnboardingLifecycleState::Completed { .. }) {
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Pending,
            reason_code: OnboardingCompletionReasonCode::AlreadyCompleted,
            next_state_machine: machine.clone(),
            should_emit_completed_event: false,
        });
    }

    let OnboardingLifecycleState::InProgress { current_step, .. } = &machine.state else {
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Pending,
            reason_code: OnboardingCompletionReasonCode::NotAtFirstConnectStep,
            next_state_machine: machine.clone(),
            should_emit_completed_event: false,
        });
    };

    if *current_step != OnboardingStep::FirstConnect {
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Pending,
            reason_code: OnboardingCompletionReasonCode::NotAtFirstConnectStep,
            next_state_machine: machine.clone(),
            should_emit_completed_event: false,
        });
    }

    let Some(transition) = routing_transition else {
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Pending,
            reason_code: OnboardingCompletionReasonCode::AwaitingSuccessfulConnect,
            next_state_machine: machine.clone(),
            should_emit_completed_event: false,
        });
    };

    if transition.trigger == RoutingTrigger::ConnectionAttemptFailed {
        let failed = transition_onboarding_state(
            machine,
            OnboardingTransition::FailStep {
                step: OnboardingStep::FirstConnect,
                reason_code: OnboardingCompletionReasonCode::ConnectAttemptFailed
                    .as_code()
                    .to_string(),
            },
        )?;
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Failed,
            reason_code: OnboardingCompletionReasonCode::ConnectAttemptFailed,
            next_state_machine: failed,
            should_emit_completed_event: false,
        });
    }

    if transition.trigger == RoutingTrigger::ConnectionEstablished {
        let completed = transition_onboarding_state(
            machine,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::FirstConnect,
            },
        )?;
        return Ok(OnboardingCompletionDecision {
            outcome: OnboardingCompletionOutcome::Completed,
            reason_code: OnboardingCompletionReasonCode::CompletedOnFirstSuccessfulConnect,
            next_state_machine: completed,
            should_emit_completed_event: true,
        });
    }

    Ok(OnboardingCompletionDecision {
        outcome: OnboardingCompletionOutcome::Pending,
        reason_code: OnboardingCompletionReasonCode::RoutingLifecycleDidNotReachConnected,
        next_state_machine: machine.clone(),
        should_emit_completed_event: false,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_onboarding_completion, OnboardingCompletionOutcome, OnboardingCompletionReasonCode,
    };
    use crate::onboarding::state_machine::{
        transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
        OnboardingTransition,
    };
    use crate::routing::{RoutingStateMachine, RoutingTrigger};

    fn onboarding_at_first_connect() -> OnboardingStateMachine {
        let begin = transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
            .expect("begin should succeed");
        let welcome = transition_onboarding_state(
            &begin,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::Welcome,
            },
        )
        .expect("welcome should complete");
        let permission = transition_onboarding_state(
            &welcome,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::PermissionCheck,
            },
        )
        .expect("permission should complete");
        transition_onboarding_state(
            &permission,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::RelayTest,
            },
        )
        .and_then(|relay| {
            transition_onboarding_state(
                &relay,
                OnboardingTransition::CompleteStep {
                    step: OnboardingStep::GameDetectionTest,
                },
            )
        })
        .expect("must reach first connect step")
    }

    #[test]
    fn completion_cannot_occur_before_successful_connect_state() {
        let onboarding = onboarding_at_first_connect();

        let mut routing_machine = RoutingStateMachine::new();
        let enable = routing_machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        let decision = evaluate_onboarding_completion(&onboarding, Some(&enable))
            .expect("evaluation should succeed");

        assert_eq!(decision.outcome, OnboardingCompletionOutcome::Pending);
        assert_eq!(
            decision.reason_code,
            OnboardingCompletionReasonCode::RoutingLifecycleDidNotReachConnected
        );
        assert!(matches!(
            decision.next_state_machine.state,
            OnboardingLifecycleState::InProgress { .. }
        ));
        assert!(!decision.should_emit_completed_event);
    }

    #[test]
    fn successful_connect_updates_onboarding_state_to_completed() {
        let onboarding = onboarding_at_first_connect();

        let mut routing_machine = RoutingStateMachine::new();
        routing_machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        let established = routing_machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");

        let decision = evaluate_onboarding_completion(&onboarding, Some(&established))
            .expect("evaluation should succeed");
        assert_eq!(decision.outcome, OnboardingCompletionOutcome::Completed);
        assert_eq!(
            decision.reason_code,
            OnboardingCompletionReasonCode::CompletedOnFirstSuccessfulConnect
        );
        assert!(decision.should_emit_completed_event);
        assert!(matches!(
            decision.next_state_machine.state,
            OnboardingLifecycleState::Completed { .. }
        ));
    }

    #[test]
    fn failed_connect_keeps_onboarding_in_non_complete_state() {
        let onboarding = onboarding_at_first_connect();

        let mut routing_machine = RoutingStateMachine::new();
        routing_machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        let failed = routing_machine
            .transition(
                RoutingTrigger::ConnectionAttemptFailed,
                Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
            )
            .expect("connecting -> failed should be legal");

        let decision = evaluate_onboarding_completion(&onboarding, Some(&failed))
            .expect("evaluation should succeed");
        assert_eq!(decision.outcome, OnboardingCompletionOutcome::Failed);
        assert_eq!(
            decision.reason_code,
            OnboardingCompletionReasonCode::ConnectAttemptFailed
        );
        assert!(!decision.should_emit_completed_event);
        assert!(matches!(
            decision.next_state_machine.state,
            OnboardingLifecycleState::Failed { .. }
        ));
    }

    #[test]
    fn completion_gate_integrates_with_routing_lifecycle_transitions() {
        let onboarding = onboarding_at_first_connect();
        let mut routing_machine = RoutingStateMachine::new();

        let enable = routing_machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        let pending = evaluate_onboarding_completion(&onboarding, Some(&enable))
            .expect("evaluation should succeed");
        assert_eq!(pending.outcome, OnboardingCompletionOutcome::Pending);

        let established = routing_machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");
        let completed = evaluate_onboarding_completion(&onboarding, Some(&established))
            .expect("evaluation should succeed");
        assert_eq!(completed.outcome, OnboardingCompletionOutcome::Completed);
    }
}
