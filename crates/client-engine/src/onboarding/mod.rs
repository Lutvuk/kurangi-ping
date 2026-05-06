pub mod checks;
pub mod completion_gate;
pub mod state_machine;

pub use checks::{
    apply_check_result_transition, run_environment_check, run_permission_check,
    run_relay_test_check, CheckCondition, CheckOutcomeCode, CheckResult, CheckStatus,
    EnvironmentCheckInput, PermissionCheckInput, RelayReadinessPolicy,
};
pub use completion_gate::{
    evaluate_onboarding_completion, OnboardingCompletionDecision, OnboardingCompletionOutcome,
    OnboardingCompletionReasonCode,
};
pub use state_machine::{
    transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStateTag,
    OnboardingStep, OnboardingTransition, OnboardingTransitionError,
};
