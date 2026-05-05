//! Toggle command lifecycle orchestrator for one-click ON/OFF flows.

use super::{
    IllegalTransitionError, RoutingState, RoutingStateMachine, RoutingStateView, RoutingTransition,
    RoutingTrigger,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleCommand {
    On,
    Off,
}

impl ToggleCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleResultCode {
    Applied,
    IllegalTransitionRejected,
}

impl ToggleResultCode {
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::Applied => "TOGGLE_APPLIED",
            Self::IllegalTransitionRejected => "TOGGLE_ILLEGAL_TRANSITION_REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleRejectionKind {
    UnsupportedStateForCommand,
    StateMachineRejectedTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleCommandResult {
    pub command: ToggleCommand,
    pub from_state: RoutingState,
    pub to_state: RoutingState,
    pub trigger: Option<RoutingTrigger>,
    pub code: ToggleResultCode,
    pub rejection_kind: Option<ToggleRejectionKind>,
    pub transition: Option<RoutingTransition>,
    pub state_machine_error: Option<IllegalTransitionError>,
}

impl ToggleCommandResult {
    pub const fn result_code(&self) -> &'static str {
        self.code.as_code()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleOrchestrator {
    machine: RoutingStateMachine,
}

impl ToggleOrchestrator {
    pub fn new() -> Self {
        Self {
            machine: RoutingStateMachine::new(),
        }
    }

    pub fn from_machine(machine: RoutingStateMachine) -> Self {
        Self { machine }
    }

    pub fn state(&self) -> RoutingState {
        self.machine.state()
    }

    pub fn state_view(&self) -> RoutingStateView {
        self.machine.ui_snapshot()
    }

    pub fn handle_toggle_command(&mut self, command: ToggleCommand) -> ToggleCommandResult {
        handle_toggle_command(&mut self.machine, command)
    }

    pub fn into_state_machine(self) -> RoutingStateMachine {
        self.machine
    }
}

impl Default for ToggleOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_toggle_command(
    machine: &mut RoutingStateMachine,
    command: ToggleCommand,
) -> ToggleCommandResult {
    let from_state = machine.state();
    let Some(trigger) = resolve_trigger_for_command(command, from_state) else {
        return ToggleCommandResult {
            command,
            from_state,
            to_state: from_state,
            trigger: None,
            code: ToggleResultCode::IllegalTransitionRejected,
            rejection_kind: Some(ToggleRejectionKind::UnsupportedStateForCommand),
            transition: None,
            state_machine_error: None,
        };
    };

    match machine.transition(trigger, None) {
        Ok(transition) => ToggleCommandResult {
            command,
            from_state,
            to_state: transition.to,
            trigger: Some(trigger),
            code: ToggleResultCode::Applied,
            rejection_kind: None,
            transition: Some(transition),
            state_machine_error: None,
        },
        Err(err) => ToggleCommandResult {
            command,
            from_state,
            to_state: from_state,
            trigger: Some(trigger),
            code: ToggleResultCode::IllegalTransitionRejected,
            rejection_kind: Some(ToggleRejectionKind::StateMachineRejectedTransition),
            transition: None,
            state_machine_error: Some(err),
        },
    }
}

fn resolve_trigger_for_command(
    command: ToggleCommand,
    state: RoutingState,
) -> Option<RoutingTrigger> {
    match (command, state) {
        (ToggleCommand::On, RoutingState::Off) => Some(RoutingTrigger::EnableRequested),
        (ToggleCommand::On, RoutingState::Failed) => Some(RoutingTrigger::RetryRequested),
        (ToggleCommand::Off, RoutingState::Connecting) => Some(RoutingTrigger::DisableRequested),
        (ToggleCommand::Off, RoutingState::Connected) => Some(RoutingTrigger::DisableRequested),
        (ToggleCommand::Off, RoutingState::Degraded) => Some(RoutingTrigger::DisableRequested),
        (ToggleCommand::Off, RoutingState::Failed) => Some(RoutingTrigger::DisableRequested),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        handle_toggle_command, ToggleCommand, ToggleOrchestrator, ToggleRejectionKind,
        ToggleResultCode,
    };
    use crate::routing::{RoutingState, RoutingStateMachine, RoutingTrigger};

    #[test]
    fn on_command_moves_off_to_connecting() {
        let mut machine = RoutingStateMachine::new();

        let result = handle_toggle_command(&mut machine, ToggleCommand::On);

        assert_eq!(result.command, ToggleCommand::On);
        assert_eq!(result.from_state, RoutingState::Off);
        assert_eq!(result.to_state, RoutingState::Connecting);
        assert_eq!(result.code, ToggleResultCode::Applied);
        assert_eq!(result.result_code(), "TOGGLE_APPLIED");
        assert_eq!(result.trigger, Some(RoutingTrigger::EnableRequested));
        assert!(result.rejection_kind.is_none());
        assert!(result.state_machine_error.is_none());
        assert_eq!(machine.state(), RoutingState::Connecting);
    }

    #[test]
    fn off_command_moves_connected_to_off() {
        let mut machine = RoutingStateMachine::new();
        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");

        let result = handle_toggle_command(&mut machine, ToggleCommand::Off);

        assert_eq!(result.command, ToggleCommand::Off);
        assert_eq!(result.from_state, RoutingState::Connected);
        assert_eq!(result.to_state, RoutingState::Off);
        assert_eq!(result.code, ToggleResultCode::Applied);
        assert_eq!(result.trigger, Some(RoutingTrigger::DisableRequested));
        assert!(result.rejection_kind.is_none());
        assert!(result.state_machine_error.is_none());
        assert_eq!(machine.state(), RoutingState::Off);
    }

    #[test]
    fn unsupported_transition_is_rejected_safely() {
        let mut machine = RoutingStateMachine::new();
        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");

        let result = handle_toggle_command(&mut machine, ToggleCommand::On);

        assert_eq!(result.command, ToggleCommand::On);
        assert_eq!(result.from_state, RoutingState::Connected);
        assert_eq!(result.to_state, RoutingState::Connected);
        assert_eq!(result.code, ToggleResultCode::IllegalTransitionRejected);
        assert_eq!(
            result.rejection_kind,
            Some(ToggleRejectionKind::UnsupportedStateForCommand)
        );
        assert!(result.transition.is_none());
        assert!(result.state_machine_error.is_none());
        assert_eq!(machine.state(), RoutingState::Connected);
    }

    #[test]
    fn orchestrator_returns_deterministic_contract() {
        let mut orchestrator = ToggleOrchestrator::new();

        let first = orchestrator.handle_toggle_command(ToggleCommand::Off);
        let second = orchestrator.handle_toggle_command(ToggleCommand::Off);

        assert_eq!(first.command, ToggleCommand::Off);
        assert_eq!(first.code, ToggleResultCode::IllegalTransitionRejected);
        assert_eq!(first.from_state, RoutingState::Off);
        assert_eq!(first.to_state, RoutingState::Off);
        assert_eq!(first.result_code(), "TOGGLE_ILLEGAL_TRANSITION_REJECTED");

        assert_eq!(second.command, ToggleCommand::Off);
        assert_eq!(second.code, ToggleResultCode::IllegalTransitionRejected);
        assert_eq!(second.from_state, RoutingState::Off);
        assert_eq!(second.to_state, RoutingState::Off);
        assert_eq!(
            second.rejection_kind,
            Some(ToggleRejectionKind::UnsupportedStateForCommand)
        );
        assert_eq!(second.result_code(), "TOGGLE_ILLEGAL_TRANSITION_REJECTED");
    }
}
