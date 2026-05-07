use std::sync::Mutex;

use client_engine::routing::{
    RoutingState, ToggleCommand, ToggleCommandResult, ToggleOrchestrator, ToggleResultCode,
};
use tauri::{AppHandle, State};

use super::contracts::{
    RoutingLifecycleResponse, RoutingLifecycleState, RoutingToggleOffRequest,
    RoutingToggleOnRequest,
};
use super::error_map::{map_ipc_error_reason, map_routing_rejection_kind, IpcErrorKind};
use super::event_bridge::{emit_routing_state_changed, EventBridgeState};

#[derive(Debug, Default)]
pub struct RoutingCommandState {
    orchestrator: Mutex<ToggleOrchestrator>,
}

#[tauri::command]
pub fn routing_toggle_on(
    _request: RoutingToggleOnRequest,
    state: State<'_, RoutingCommandState>,
    app_handle: AppHandle,
    event_bridge_state: State<'_, EventBridgeState>,
) -> Result<RoutingLifecycleResponse, String> {
    execute_and_emit(
        &state,
        ToggleCommand::On,
        &app_handle,
        event_bridge_state.inner(),
    )
}

#[tauri::command]
pub fn routing_toggle_off(
    _request: RoutingToggleOffRequest,
    state: State<'_, RoutingCommandState>,
    app_handle: AppHandle,
    event_bridge_state: State<'_, EventBridgeState>,
) -> Result<RoutingLifecycleResponse, String> {
    execute_and_emit(
        &state,
        ToggleCommand::Off,
        &app_handle,
        event_bridge_state.inner(),
    )
}

fn execute_and_emit(
    state: &State<'_, RoutingCommandState>,
    command: ToggleCommand,
    app_handle: &AppHandle,
    event_bridge_state: &EventBridgeState,
) -> Result<RoutingLifecycleResponse, String> {
    let (previous_state, response) = match execute_with_state(state, command) {
        Ok(result) => {
            let previous_state = map_routing_state(result.from_state);
            let response = normalize_toggle_result(&result);
            (previous_state, response)
        }
        Err(response) => (RoutingLifecycleState::Error, response),
    };

    emit_routing_state_changed(app_handle, event_bridge_state, previous_state, &response)?;
    Ok(response)
}

fn execute_with_state(
    state: &State<'_, RoutingCommandState>,
    command: ToggleCommand,
) -> Result<ToggleCommandResult, RoutingLifecycleResponse> {
    match state.orchestrator.lock() {
        Ok(mut orchestrator) => {
            let result = orchestrator.handle_toggle_command(command);
            Ok(result)
        }
        Err(_) => Err(RoutingLifecycleResponse {
            state: RoutingLifecycleState::Error,
            reason_code: Some(map_ipc_error_reason(IpcErrorKind::UnknownFailure).to_string()),
            message: Some("routing command state is unavailable".to_string()),
        }),
    }
}

fn map_routing_state(state: RoutingState) -> RoutingLifecycleState {
    match state {
        RoutingState::Off => RoutingLifecycleState::Idle,
        RoutingState::Connecting => RoutingLifecycleState::Connecting,
        RoutingState::Connected => RoutingLifecycleState::Active,
        RoutingState::Degraded => RoutingLifecycleState::Degraded,
        RoutingState::Failed => RoutingLifecycleState::Error,
    }
}

fn normalize_toggle_result(result: &ToggleCommandResult) -> RoutingLifecycleResponse {
    let normalized_state = map_routing_state(result.to_state);

    match result.code {
        ToggleResultCode::Applied => RoutingLifecycleResponse {
            state: normalized_state,
            reason_code: None,
            message: None,
        },
        ToggleResultCode::IllegalTransitionRejected => {
            let reason_code =
                map_ipc_error_reason(map_routing_rejection_kind(result.rejection_kind));

            RoutingLifecycleResponse {
                state: normalized_state,
                reason_code: Some(reason_code.to_string()),
                message: Some(format!(
                    "toggle '{}' rejected while routing state is '{}'",
                    result.command.as_str(),
                    result.from_state.as_str()
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use client_engine::routing::{ToggleCommand, ToggleOrchestrator};

    use super::{map_routing_state, normalize_toggle_result, RoutingLifecycleState};

    #[test]
    fn on_command_invokes_lifecycle_and_returns_connecting_state() {
        let mut orchestrator = ToggleOrchestrator::new();
        let result = orchestrator.handle_toggle_command(ToggleCommand::On);
        let response = normalize_toggle_result(&result);

        assert_eq!(response.state, RoutingLifecycleState::Connecting);
        assert_eq!(response.reason_code, None);
    }

    #[test]
    fn off_command_invokes_lifecycle_and_returns_idle_state() {
        let mut orchestrator = ToggleOrchestrator::new();
        let on = orchestrator.handle_toggle_command(ToggleCommand::On);
        let on_response = normalize_toggle_result(&on);
        assert_eq!(on_response.state, RoutingLifecycleState::Connecting);

        let off = orchestrator.handle_toggle_command(ToggleCommand::Off);
        let off_response = normalize_toggle_result(&off);
        assert_eq!(off_response.state, RoutingLifecycleState::Idle);
        assert_eq!(off_response.reason_code, None);
    }

    #[test]
    fn illegal_transition_is_mapped_to_ui_safe_reason_code() {
        let mut orchestrator = ToggleOrchestrator::new();
        let first = orchestrator.handle_toggle_command(ToggleCommand::On);
        let _ = normalize_toggle_result(&first);

        let rejected = orchestrator.handle_toggle_command(ToggleCommand::On);
        let response = normalize_toggle_result(&rejected);
        assert_eq!(response.state, RoutingLifecycleState::Connecting);
        assert_eq!(response.reason_code.as_deref(), Some("ipc_invalid_state"));
        assert!(response
            .message
            .as_deref()
            .is_some_and(|value| value.contains("rejected")));
    }

    #[test]
    fn routing_state_mapping_to_ui_state_is_stable() {
        use client_engine::routing::RoutingState;

        assert_eq!(
            map_routing_state(RoutingState::Off),
            RoutingLifecycleState::Idle
        );
        assert_eq!(
            map_routing_state(RoutingState::Connecting),
            RoutingLifecycleState::Connecting
        );
        assert_eq!(
            map_routing_state(RoutingState::Connected),
            RoutingLifecycleState::Active
        );
        assert_eq!(
            map_routing_state(RoutingState::Degraded),
            RoutingLifecycleState::Degraded
        );
        assert_eq!(
            map_routing_state(RoutingState::Failed),
            RoutingLifecycleState::Error
        );
    }
}
