//! Routing lifecycle state machine.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingState {
    Off,
    Connecting,
    Connected,
    Degraded,
    Failed,
}

impl RoutingState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingTrigger {
    EnableRequested,
    DisableRequested,
    ConnectionEstablished,
    ConnectionAttemptFailed,
    HealthDegraded,
    HealthRecovered,
    RelayFailure,
    RetryRequested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingTransition {
    pub from: RoutingState,
    pub to: RoutingState,
    pub trigger: RoutingTrigger,
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingStateView {
    pub state: &'static str,
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IllegalTransitionError {
    pub from: RoutingState,
    pub trigger: RoutingTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingStateMachine {
    state: RoutingState,
    last_failure_code: Option<String>,
}

impl Default for RoutingStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingStateMachine {
    pub fn new() -> Self {
        Self {
            state: RoutingState::Off,
            last_failure_code: None,
        }
    }

    pub fn state(&self) -> RoutingState {
        self.state
    }

    pub fn transition(
        &mut self,
        trigger: RoutingTrigger,
        failure_code: Option<String>,
    ) -> Result<RoutingTransition, IllegalTransitionError> {
        let from = self.state;
        let to = next_state(from, trigger).ok_or(IllegalTransitionError { from, trigger })?;

        self.state = to;
        self.last_failure_code = match to {
            RoutingState::Failed => failure_code.clone(),
            _ => None,
        };

        Ok(RoutingTransition {
            from,
            to,
            trigger,
            failure_code: self.last_failure_code.clone(),
        })
    }

    pub fn ui_snapshot(&self) -> RoutingStateView {
        RoutingStateView {
            state: self.state.as_str(),
            failure_code: self.last_failure_code.clone(),
        }
    }
}

fn next_state(current: RoutingState, trigger: RoutingTrigger) -> Option<RoutingState> {
    match (current, trigger) {
        (RoutingState::Off, RoutingTrigger::EnableRequested) => Some(RoutingState::Connecting),
        (RoutingState::Connecting, RoutingTrigger::ConnectionEstablished) => {
            Some(RoutingState::Connected)
        }
        (RoutingState::Connecting, RoutingTrigger::ConnectionAttemptFailed) => {
            Some(RoutingState::Failed)
        }
        (RoutingState::Connecting, RoutingTrigger::DisableRequested) => Some(RoutingState::Off),
        (RoutingState::Connected, RoutingTrigger::HealthDegraded) => Some(RoutingState::Degraded),
        (RoutingState::Connected, RoutingTrigger::DisableRequested) => Some(RoutingState::Off),
        (RoutingState::Degraded, RoutingTrigger::HealthRecovered) => Some(RoutingState::Connected),
        (RoutingState::Degraded, RoutingTrigger::RelayFailure) => Some(RoutingState::Failed),
        (RoutingState::Degraded, RoutingTrigger::DisableRequested) => Some(RoutingState::Off),
        (RoutingState::Failed, RoutingTrigger::RetryRequested) => Some(RoutingState::Connecting),
        (RoutingState::Failed, RoutingTrigger::DisableRequested) => Some(RoutingState::Off),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{RoutingState, RoutingStateMachine, RoutingTrigger};

    #[test]
    fn states_include_required_minimum_set() {
        assert_eq!(RoutingState::Off.as_str(), "off");
        assert_eq!(RoutingState::Connecting.as_str(), "connecting");
        assert_eq!(RoutingState::Connected.as_str(), "connected");
        assert_eq!(RoutingState::Degraded.as_str(), "degraded");
        assert_eq!(RoutingState::Failed.as_str(), "failed");
    }

    #[test]
    fn legal_transitions_progress_state() {
        let mut machine = RoutingStateMachine::new();

        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        assert_eq!(machine.state(), RoutingState::Connecting);

        machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");
        assert_eq!(machine.state(), RoutingState::Connected);

        machine
            .transition(RoutingTrigger::HealthDegraded, None)
            .expect("connected -> degraded should be legal");
        assert_eq!(machine.state(), RoutingState::Degraded);

        machine
            .transition(RoutingTrigger::HealthRecovered, None)
            .expect("degraded -> connected should be legal");
        assert_eq!(machine.state(), RoutingState::Connected);
    }

    #[test]
    fn illegal_transition_is_rejected() {
        let mut machine = RoutingStateMachine::new();
        let err = machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect_err("off -> connected direct transition is illegal");
        assert_eq!(err.from, RoutingState::Off);
        assert_eq!(err.trigger, RoutingTrigger::ConnectionEstablished);
    }

    #[test]
    fn failed_state_snapshot_is_serializable_for_ui() {
        let mut machine = RoutingStateMachine::new();
        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        machine
            .transition(
                RoutingTrigger::ConnectionAttemptFailed,
                Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
            )
            .expect("connecting -> failed should be legal");

        let snapshot = machine.ui_snapshot();
        assert_eq!(snapshot.state, "failed");
        assert_eq!(
            snapshot.failure_code.as_deref(),
            Some("ROUTE_ALL_ATTEMPTS_FAILED")
        );
    }
}
