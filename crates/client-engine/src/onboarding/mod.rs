pub mod state_machine;

pub use state_machine::{
    transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStateTag,
    OnboardingStep, OnboardingTransition, OnboardingTransitionError,
};
