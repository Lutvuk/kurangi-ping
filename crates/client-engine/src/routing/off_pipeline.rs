//! OFF pipeline that performs safe route teardown and session finalization.

use super::{
    close_route_session, RouteSessionPersistence, RoutingState, RoutingStateMachine,
    RoutingTransition, RoutingTrigger, SessionHookResult,
};
use crate::db::RouteSessionCloseRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffTeardownErrorCode {
    PermissionDenied,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffTeardownError {
    pub code: OffTeardownErrorCode,
    pub message: String,
}

impl OffTeardownError {
    pub fn new(code: OffTeardownErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffPipelineReasonCode {
    OffRequested,
    OffAlreadyDisabled,
    OffTeardownFailed,
}

impl OffPipelineReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OffRequested => "off_requested",
            Self::OffAlreadyDisabled => "off_already_disabled",
            Self::OffTeardownFailed => "off_teardown_failed",
        }
    }

    pub const fn as_failure_code(self) -> &'static str {
        match self {
            Self::OffRequested => "OFF_REQUESTED",
            Self::OffAlreadyDisabled => "OFF_ALREADY_DISABLED",
            Self::OffTeardownFailed => "OFF_TEARDOWN_FAILED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffPipelineStatus {
    Completed,
    Idempotent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffPipelineResult {
    pub initial_state: RoutingState,
    pub final_state: RoutingState,
    pub status: OffPipelineStatus,
    pub reason_code: OffPipelineReasonCode,
    pub teardown_attempted: bool,
    pub teardown_error: Option<OffTeardownError>,
    pub transitions: Vec<RoutingTransition>,
    pub session_hook: SessionHookResult,
    pub close_record: RouteSessionCloseRecord,
}

pub trait OffRouteTeardown {
    fn teardown_active_route(&mut self) -> Result<(), OffTeardownError>;
}

pub fn execute_off_pipeline<T: OffRouteTeardown, P: RouteSessionPersistence>(
    machine: &mut RoutingStateMachine,
    teardown: &mut T,
    persistence: &P,
    close_record_template: &RouteSessionCloseRecord,
) -> OffPipelineResult {
    let initial_state = machine.state();
    let teardown_attempted = should_attempt_teardown(initial_state);

    if !teardown_attempted {
        let reason_code = OffPipelineReasonCode::OffAlreadyDisabled;
        let close_record = with_end_reason(close_record_template, reason_code);
        let session_hook = close_route_session(persistence, &close_record);

        return OffPipelineResult {
            initial_state,
            final_state: machine.state(),
            status: OffPipelineStatus::Idempotent,
            reason_code,
            teardown_attempted: false,
            teardown_error: None,
            transitions: Vec::new(),
            session_hook,
            close_record,
        };
    }

    let teardown_result = teardown.teardown_active_route();
    match teardown_result {
        Ok(()) => {
            let reason_code = OffPipelineReasonCode::OffRequested;
            let transitions = transition_to_off_if_needed(machine);
            let close_record = with_end_reason(close_record_template, reason_code);
            let session_hook = close_route_session(persistence, &close_record);

            OffPipelineResult {
                initial_state,
                final_state: machine.state(),
                status: OffPipelineStatus::Completed,
                reason_code,
                teardown_attempted: true,
                teardown_error: None,
                transitions,
                session_hook,
                close_record,
            }
        }
        Err(error) => {
            let reason_code = OffPipelineReasonCode::OffTeardownFailed;
            let transitions = force_deterministic_failed_state(machine, reason_code.as_failure_code());
            let close_record = with_end_reason(close_record_template, reason_code);
            let session_hook = close_route_session(persistence, &close_record);

            OffPipelineResult {
                initial_state,
                final_state: machine.state(),
                status: OffPipelineStatus::Failed,
                reason_code,
                teardown_attempted: true,
                teardown_error: Some(error),
                transitions,
                session_hook,
                close_record,
            }
        }
    }
}

fn should_attempt_teardown(state: RoutingState) -> bool {
    matches!(
        state,
        RoutingState::Connecting
            | RoutingState::Connected
            | RoutingState::Degraded
            | RoutingState::Failed
    )
}

fn transition_to_off_if_needed(machine: &mut RoutingStateMachine) -> Vec<RoutingTransition> {
    if machine.state() == RoutingState::Off {
        return Vec::new();
    }

    match machine.transition(RoutingTrigger::DisableRequested, None) {
        Ok(transition) => vec![transition],
        Err(_) => Vec::new(),
    }
}

fn force_deterministic_failed_state(
    machine: &mut RoutingStateMachine,
    failure_code: &str,
) -> Vec<RoutingTransition> {
    let mut transitions = Vec::new();
    match machine.state() {
        RoutingState::Connecting => {
            if let Ok(transition) = machine.transition(
                RoutingTrigger::ConnectionAttemptFailed,
                Some(failure_code.to_string()),
            ) {
                transitions.push(transition);
            }
        }
        RoutingState::Connected => {
            if let Ok(transition) = machine.transition(RoutingTrigger::HealthDegraded, None) {
                transitions.push(transition);
            }
            if let Ok(transition) = machine.transition(
                RoutingTrigger::RelayFailure,
                Some(failure_code.to_string()),
            ) {
                transitions.push(transition);
            }
        }
        RoutingState::Degraded => {
            if let Ok(transition) =
                machine.transition(RoutingTrigger::RelayFailure, Some(failure_code.to_string()))
            {
                transitions.push(transition);
            }
        }
        RoutingState::Failed | RoutingState::Off => {}
    }
    transitions
}

fn with_end_reason(
    template: &RouteSessionCloseRecord,
    reason_code: OffPipelineReasonCode,
) -> RouteSessionCloseRecord {
    RouteSessionCloseRecord {
        session_id: template.session_id.clone(),
        ended_at: template.ended_at.clone(),
        end_reason: Some(reason_code.as_failure_code().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        execute_off_pipeline, OffPipelineReasonCode, OffPipelineStatus, OffRouteTeardown,
        OffTeardownError, OffTeardownErrorCode,
    };
    use crate::db::{RouteSessionCloseRecord, RouteSessionStartRecord};
    use crate::routing::{
        RouteSessionPersistence, RoutingState, RoutingStateMachine, RoutingTrigger,
        SessionHookStatus,
    };
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockTeardown {
        calls: usize,
        fail_with: Option<OffTeardownError>,
    }

    impl OffRouteTeardown for MockTeardown {
        fn teardown_active_route(&mut self) -> Result<(), OffTeardownError> {
            self.calls = self.calls.saturating_add(1);
            match &self.fail_with {
                Some(err) => Err(err.clone()),
                None => Ok(()),
            }
        }
    }

    #[derive(Default)]
    struct MockSessionPersistence {
        close_calls: RefCell<Vec<RouteSessionCloseRecord>>,
    }

    impl RouteSessionPersistence for MockSessionPersistence {
        fn start_route_session(&self, _record: &RouteSessionStartRecord) -> Result<(), String> {
            Ok(())
        }

        fn close_route_session(&self, record: &RouteSessionCloseRecord) -> Result<(), String> {
            self.close_calls.borrow_mut().push(record.clone());
            Ok(())
        }
    }

    fn close_record_template() -> RouteSessionCloseRecord {
        RouteSessionCloseRecord {
            session_id: "sess-1".to_string(),
            ended_at: "2026-08-01T00:00:03Z".to_string(),
            end_reason: None,
        }
    }

    fn connected_machine() -> RoutingStateMachine {
        let mut machine = RoutingStateMachine::new();
        machine
            .transition(RoutingTrigger::EnableRequested, None)
            .expect("off -> connecting should be legal");
        machine
            .transition(RoutingTrigger::ConnectionEstablished, None)
            .expect("connecting -> connected should be legal");
        machine
    }

    #[test]
    fn active_route_teardown_is_always_attempted() {
        let mut machine = connected_machine();
        let mut teardown = MockTeardown::default();
        let persistence = MockSessionPersistence::default();

        let result = execute_off_pipeline(
            &mut machine,
            &mut teardown,
            &persistence,
            &close_record_template(),
        );

        assert_eq!(teardown.calls, 1);
        assert!(result.teardown_attempted);
        assert_eq!(result.status, OffPipelineStatus::Completed);
        assert_eq!(result.final_state, RoutingState::Off);
    }

    #[test]
    fn session_close_hook_is_invoked_with_end_reason() {
        let mut machine = connected_machine();
        let mut teardown = MockTeardown::default();
        let persistence = MockSessionPersistence::default();

        let result = execute_off_pipeline(
            &mut machine,
            &mut teardown,
            &persistence,
            &close_record_template(),
        );

        assert_eq!(result.reason_code, OffPipelineReasonCode::OffRequested);
        assert_eq!(result.close_record.end_reason.as_deref(), Some("OFF_REQUESTED"));
        assert!(matches!(result.session_hook.status, SessionHookStatus::Persisted));

        let close_calls = persistence.close_calls.borrow();
        assert_eq!(close_calls.len(), 1);
        assert_eq!(close_calls[0].end_reason.as_deref(), Some("OFF_REQUESTED"));
    }

    #[test]
    fn teardown_failure_moves_lifecycle_to_deterministic_error_state() {
        let mut machine = connected_machine();
        let mut teardown = MockTeardown {
            calls: 0,
            fail_with: Some(OffTeardownError::new(
                OffTeardownErrorCode::Timeout,
                "teardown timed out",
            )),
        };
        let persistence = MockSessionPersistence::default();

        let result = execute_off_pipeline(
            &mut machine,
            &mut teardown,
            &persistence,
            &close_record_template(),
        );

        assert_eq!(result.status, OffPipelineStatus::Failed);
        assert_eq!(result.reason_code, OffPipelineReasonCode::OffTeardownFailed);
        assert_eq!(result.final_state, RoutingState::Failed);
        assert_eq!(
            machine.ui_snapshot().failure_code.as_deref(),
            Some("OFF_TEARDOWN_FAILED")
        );
        assert_eq!(
            result.close_record.end_reason.as_deref(),
            Some("OFF_TEARDOWN_FAILED")
        );
    }

    #[test]
    fn off_from_non_active_state_is_idempotent() {
        let mut machine = RoutingStateMachine::new();
        let mut teardown = MockTeardown::default();
        let persistence = MockSessionPersistence::default();

        let result = execute_off_pipeline(
            &mut machine,
            &mut teardown,
            &persistence,
            &close_record_template(),
        );

        assert_eq!(result.status, OffPipelineStatus::Idempotent);
        assert_eq!(result.reason_code, OffPipelineReasonCode::OffAlreadyDisabled);
        assert!(!result.teardown_attempted);
        assert_eq!(teardown.calls, 0);
        assert_eq!(result.final_state, RoutingState::Off);
    }
}
