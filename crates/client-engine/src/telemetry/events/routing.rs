use crate::routing::{RoutingState, RoutingTransition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingEventPayload {
    pub state: &'static str,
    pub previous_state: &'static str,
    pub failure_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingLifecycleEvent {
    pub name: &'static str,
    pub payload: RoutingEventPayload,
}

pub fn emit_routing_event(transition: &RoutingTransition) -> Option<RoutingLifecycleEvent> {
    let name = match (transition.from, transition.to) {
        (RoutingState::Off, RoutingState::Connecting) => "routing_enabled",
        (_, RoutingState::Off) => "routing_disabled",
        (_, RoutingState::Failed) => "relay_failed",
        _ => return None,
    };

    Some(RoutingLifecycleEvent {
        name,
        payload: RoutingEventPayload {
            state: transition.to.as_str(),
            previous_state: transition.from.as_str(),
            failure_code: transition.failure_code.clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::emit_routing_event;
    use crate::routing::{RoutingState, RoutingTransition, RoutingTrigger};

    fn transition(
        from: RoutingState,
        to: RoutingState,
        trigger: RoutingTrigger,
        failure_code: Option<&str>,
    ) -> RoutingTransition {
        RoutingTransition {
            from,
            to,
            trigger,
            failure_code: failure_code.map(ToString::to_string),
        }
    }

    #[test]
    fn maps_routing_enabled_and_disabled_consistently() {
        let enabled = emit_routing_event(&transition(
            RoutingState::Off,
            RoutingState::Connecting,
            RoutingTrigger::EnableRequested,
            None,
        ))
        .expect("enable transition should emit event");
        assert_eq!(enabled.name, "routing_enabled");
        assert_eq!(enabled.payload.state, "connecting");
        assert_eq!(enabled.payload.previous_state, "off");

        let disabled = emit_routing_event(&transition(
            RoutingState::Connected,
            RoutingState::Off,
            RoutingTrigger::DisableRequested,
            None,
        ))
        .expect("disable transition should emit event");
        assert_eq!(disabled.name, "routing_disabled");
        assert_eq!(disabled.payload.state, "off");
        assert_eq!(disabled.payload.previous_state, "connected");
    }

    #[test]
    fn maps_relay_failed_with_deterministic_payload() {
        let failed = emit_routing_event(&transition(
            RoutingState::Connecting,
            RoutingState::Failed,
            RoutingTrigger::ConnectionAttemptFailed,
            Some("ROUTE_ALL_ATTEMPTS_FAILED"),
        ))
        .expect("failed transition should emit event");

        assert_eq!(failed.name, "relay_failed");
        assert_eq!(failed.payload.state, "failed");
        assert_eq!(
            failed.payload.failure_code.as_deref(),
            Some("ROUTE_ALL_ATTEMPTS_FAILED")
        );
    }

    #[test]
    fn non_lifecycle_transition_can_skip_event() {
        let event = emit_routing_event(&transition(
            RoutingState::Connected,
            RoutingState::Degraded,
            RoutingTrigger::HealthDegraded,
            None,
        ));
        assert!(event.is_none());
    }
}
