use serde::{Deserialize, Serialize};

pub const CANONICAL_ONBOARDING_STEPS: [OnboardingStep; 5] = [
    OnboardingStep::Welcome,
    OnboardingStep::PermissionCheck,
    OnboardingStep::RelayTest,
    OnboardingStep::GameDetectionTest,
    OnboardingStep::FirstConnect,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Welcome,
    PermissionCheck,
    RelayTest,
    GameDetectionTest,
    FirstConnect,
}

impl OnboardingStep {
    fn next(self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::PermissionCheck),
            Self::PermissionCheck => Some(Self::RelayTest),
            Self::RelayTest => Some(Self::GameDetectionTest),
            Self::GameDetectionTest => Some(Self::FirstConnect),
            Self::FirstConnect => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStateTag {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OnboardingLifecycleState {
    NotStarted,
    InProgress {
        current_step: OnboardingStep,
        completed_steps: Vec<OnboardingStep>,
    },
    Completed {
        completed_steps: Vec<OnboardingStep>,
    },
    Blocked {
        blocked_step: OnboardingStep,
        completed_steps: Vec<OnboardingStep>,
        reason_code: String,
    },
    Failed {
        failed_step: OnboardingStep,
        completed_steps: Vec<OnboardingStep>,
        reason_code: String,
    },
}

impl OnboardingLifecycleState {
    pub fn tag(&self) -> OnboardingStateTag {
        match self {
            Self::NotStarted => OnboardingStateTag::NotStarted,
            Self::InProgress { .. } => OnboardingStateTag::InProgress,
            Self::Completed { .. } => OnboardingStateTag::Completed,
            Self::Blocked { .. } => OnboardingStateTag::Blocked,
            Self::Failed { .. } => OnboardingStateTag::Failed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum OnboardingTransition {
    Begin,
    CompleteStep { step: OnboardingStep },
    BlockStep { step: OnboardingStep, reason_code: String },
    FailStep { step: OnboardingStep, reason_code: String },
    ResumeFromBlocked,
    RetryFromFailed,
    Restart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnboardingTransitionError {
    pub from_state: OnboardingStateTag,
    pub transition: OnboardingTransition,
    pub reason_code: String,
}

impl OnboardingTransitionError {
    fn new(
        from_state: OnboardingStateTag,
        transition: OnboardingTransition,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            from_state,
            transition,
            reason_code: reason_code.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnboardingStateMachine {
    pub state: OnboardingLifecycleState,
}

impl OnboardingStateMachine {
    pub fn new() -> Self {
        Self {
            state: OnboardingLifecycleState::NotStarted,
        }
    }

    pub fn canonical_steps(&self) -> &'static [OnboardingStep] {
        &CANONICAL_ONBOARDING_STEPS
    }

    pub fn transition(
        &self,
        transition: OnboardingTransition,
    ) -> Result<Self, OnboardingTransitionError> {
        transition_onboarding_state(self, transition)
    }
}

impl Default for OnboardingStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn transition_onboarding_state(
    machine: &OnboardingStateMachine,
    transition: OnboardingTransition,
) -> Result<OnboardingStateMachine, OnboardingTransitionError> {
    let next_state = match (&machine.state, &transition) {
        (OnboardingLifecycleState::NotStarted, OnboardingTransition::Begin) => {
            OnboardingLifecycleState::InProgress {
                current_step: OnboardingStep::Welcome,
                completed_steps: Vec::new(),
            }
        }
        (OnboardingLifecycleState::InProgress { current_step, completed_steps }, OnboardingTransition::CompleteStep { step }) => {
            if step != current_step {
                return Err(OnboardingTransitionError::new(
                    machine.state.tag(),
                    transition,
                    "step_mismatch",
                ));
            }

            let mut progressed = completed_steps.clone();
            progressed.push(*current_step);

            match current_step.next() {
                Some(next_step) => OnboardingLifecycleState::InProgress {
                    current_step: next_step,
                    completed_steps: progressed,
                },
                None => OnboardingLifecycleState::Completed {
                    completed_steps: progressed,
                },
            }
        }
        (OnboardingLifecycleState::InProgress { current_step, completed_steps }, OnboardingTransition::BlockStep { step, reason_code }) => {
            if step != current_step {
                return Err(OnboardingTransitionError::new(
                    machine.state.tag(),
                    transition,
                    "step_mismatch",
                ));
            }
            if reason_code.trim().is_empty() {
                return Err(OnboardingTransitionError::new(
                    machine.state.tag(),
                    transition,
                    "reason_code_required",
                ));
            }
            OnboardingLifecycleState::Blocked {
                blocked_step: *current_step,
                completed_steps: completed_steps.clone(),
                reason_code: reason_code.clone(),
            }
        }
        (OnboardingLifecycleState::InProgress { current_step, completed_steps }, OnboardingTransition::FailStep { step, reason_code }) => {
            if step != current_step {
                return Err(OnboardingTransitionError::new(
                    machine.state.tag(),
                    transition,
                    "step_mismatch",
                ));
            }
            if reason_code.trim().is_empty() {
                return Err(OnboardingTransitionError::new(
                    machine.state.tag(),
                    transition,
                    "reason_code_required",
                ));
            }
            OnboardingLifecycleState::Failed {
                failed_step: *current_step,
                completed_steps: completed_steps.clone(),
                reason_code: reason_code.clone(),
            }
        }
        (
            OnboardingLifecycleState::Blocked {
                blocked_step,
                completed_steps,
                ..
            },
            OnboardingTransition::ResumeFromBlocked,
        ) => OnboardingLifecycleState::InProgress {
            current_step: *blocked_step,
            completed_steps: completed_steps.clone(),
        },
        (
            OnboardingLifecycleState::Failed {
                failed_step,
                completed_steps,
                ..
            },
            OnboardingTransition::RetryFromFailed,
        ) => OnboardingLifecycleState::InProgress {
            current_step: *failed_step,
            completed_steps: completed_steps.clone(),
        },
        (_, OnboardingTransition::Restart) => OnboardingLifecycleState::NotStarted,
        _ => {
            return Err(OnboardingTransitionError::new(
                machine.state.tag(),
                transition,
                "invalid_transition",
            ))
        }
    };

    Ok(OnboardingStateMachine { state: next_state })
}

#[cfg(test)]
mod tests {
    use super::{
        transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
        OnboardingTransition, OnboardingTransitionError,
    };

    #[test]
    fn canonical_five_step_progression_completes_deterministically() {
        let initial = OnboardingStateMachine::new();
        let started = transition_onboarding_state(&initial, OnboardingTransition::Begin)
            .expect("begin should start onboarding");

        let step_1 = transition_onboarding_state(
            &started,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::Welcome,
            },
        )
        .expect("welcome should complete");
        let step_2 = transition_onboarding_state(
            &step_1,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::PermissionCheck,
            },
        )
        .expect("permission check should complete");
        let step_3 = transition_onboarding_state(
            &step_2,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::RelayTest,
            },
        )
        .expect("relay test should complete");
        let step_4 = transition_onboarding_state(
            &step_3,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::GameDetectionTest,
            },
        )
        .expect("game detection should complete");
        let completed = transition_onboarding_state(
            &step_4,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::FirstConnect,
            },
        )
        .expect("first connect should complete onboarding");

        assert_eq!(
            completed.state,
            OnboardingLifecycleState::Completed {
                completed_steps: vec![
                    OnboardingStep::Welcome,
                    OnboardingStep::PermissionCheck,
                    OnboardingStep::RelayTest,
                    OnboardingStep::GameDetectionTest,
                    OnboardingStep::FirstConnect,
                ],
            }
        );
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let initial = OnboardingStateMachine::new();
        let err = transition_onboarding_state(
            &initial,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::Welcome,
            },
        )
        .expect_err("cannot complete step before begin");
        assert_eq!(
            err,
            OnboardingTransitionError {
                from_state: super::OnboardingStateTag::NotStarted,
                transition: OnboardingTransition::CompleteStep {
                    step: OnboardingStep::Welcome
                },
                reason_code: "invalid_transition".to_string(),
            }
        );

        let started = transition_onboarding_state(&initial, OnboardingTransition::Begin)
            .expect("begin should start onboarding");
        let err = transition_onboarding_state(
            &started,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::RelayTest,
            },
        )
        .expect_err("completing wrong step must fail");
        assert_eq!(err.reason_code, "step_mismatch");
    }

    #[test]
    fn completed_blocked_and_failed_are_explicit_terminal_states() {
        let started = transition_onboarding_state(
            &OnboardingStateMachine::new(),
            OnboardingTransition::Begin,
        )
        .expect("begin should succeed");
        let blocked = transition_onboarding_state(
            &started,
            OnboardingTransition::BlockStep {
                step: OnboardingStep::Welcome,
                reason_code: "permission_denied".to_string(),
            },
        )
        .expect("block transition should succeed");
        assert!(matches!(blocked.state, OnboardingLifecycleState::Blocked { .. }));

        let resumed = transition_onboarding_state(&blocked, OnboardingTransition::ResumeFromBlocked)
            .expect("blocked flow should resume");
        let failed = transition_onboarding_state(
            &resumed,
            OnboardingTransition::FailStep {
                step: OnboardingStep::Welcome,
                reason_code: "unexpected_error".to_string(),
            },
        )
        .expect("fail transition should succeed");
        assert!(matches!(failed.state, OnboardingLifecycleState::Failed { .. }));
    }

    #[test]
    fn state_model_is_serializable_for_ui_sync() {
        let machine = OnboardingStateMachine {
            state: OnboardingLifecycleState::Blocked {
                blocked_step: OnboardingStep::RelayTest,
                completed_steps: vec![
                    OnboardingStep::Welcome,
                    OnboardingStep::PermissionCheck,
                ],
                reason_code: "relay_probe_timeout".to_string(),
            },
        };

        let encoded = serde_json::to_string(&machine).expect("state should serialize");
        let decoded: OnboardingStateMachine =
            serde_json::from_str(&encoded).expect("state should deserialize");
        assert_eq!(decoded, machine);
    }
}
