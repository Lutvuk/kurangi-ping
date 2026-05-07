use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcCommand {
    RoutingToggleOn,
    RoutingToggleOff,
    DetectionGetStatus,
}

impl IpcCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoutingToggleOn => "routing_toggle_on",
            Self::RoutingToggleOff => "routing_toggle_off",
            Self::DetectionGetStatus => "detection_get_status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcEvent {
    RoutingStateChanged,
    MetricsPingSampled,
    DetectionStatusUpdated,
}

impl IpcEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoutingStateChanged => "routing_state_changed",
            Self::MetricsPingSampled => "metrics_ping_sampled",
            Self::DetectionStatusUpdated => "detection_status_updated",
        }
    }
}

pub const IPC_REASON_CODES: [&str; 6] = [
    "ipc_invalid_state",
    "ipc_command_rejected",
    "ipc_detection_scan_failed",
    "ipc_metrics_stream_unavailable",
    "ipc_timeout",
    "ipc_unknown_failure",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingToggleOnTrigger {
    UserToggle,
    StartupResume,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingToggleOnRequest {
    pub trigger: RoutingToggleOnTrigger,
    pub requested_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingToggleOffTrigger {
    UserToggle,
    Shutdown,
    SessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingToggleOffRequest {
    pub trigger: RoutingToggleOffTrigger,
    pub requested_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingLifecycleState {
    Idle,
    Connecting,
    Active,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingLifecycleResponse {
    pub state: RoutingLifecycleState,
    pub reason_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionStatusState {
    Detected,
    NotDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionStatusResponse {
    pub state: DetectionStatusState,
    pub game_id: Option<String>,
    pub process_name: Option<String>,
    pub detection_time_ms: Option<u64>,
    pub reason_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingStateChangedEventPayload {
    pub previous_state: RoutingLifecycleState,
    pub state: RoutingLifecycleState,
    pub reason_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetricsEmissionState {
    Live,
    Measuring,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsPingSampledEventPayload {
    pub sampled_at_unix_ms: u64,
    pub state: MetricsEmissionState,
    pub baseline_ping_ms: Option<f64>,
    pub routed_ping_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetectionStatusUpdatedEventPayload {
    pub state: DetectionStatusState,
    pub game_id: Option<String>,
    pub process_name: Option<String>,
    pub detection_time_ms: Option<u64>,
    pub reason_code: Option<String>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        DetectionStatusResponse, DetectionStatusState, IpcCommand, IpcEvent, IPC_REASON_CODES,
        RoutingLifecycleResponse, RoutingLifecycleState, RoutingToggleOnRequest,
        RoutingToggleOnTrigger,
    };

    #[test]
    fn command_and_event_names_are_deterministic() {
        let commands = [
            IpcCommand::RoutingToggleOn.as_str(),
            IpcCommand::RoutingToggleOff.as_str(),
            IpcCommand::DetectionGetStatus.as_str(),
        ];
        assert_eq!(
            commands,
            [
                "routing_toggle_on",
                "routing_toggle_off",
                "detection_get_status",
            ]
        );

        let events = [
            IpcEvent::RoutingStateChanged.as_str(),
            IpcEvent::MetricsPingSampled.as_str(),
            IpcEvent::DetectionStatusUpdated.as_str(),
        ];
        assert_eq!(
            events,
            [
                "routing_state_changed",
                "metrics_ping_sampled",
                "detection_status_updated",
            ]
        );
    }

    #[test]
    fn reason_codes_are_non_sensitive_and_unique() {
        let unique = IPC_REASON_CODES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), IPC_REASON_CODES.len());

        for code in IPC_REASON_CODES {
            assert!(!code.contains("username"));
            assert!(!code.contains("email"));
            assert!(!code.contains("ip_address"));
            assert!(!code.contains("ipv4"));
            assert!(!code.contains("ipv6"));
            assert!(!code.contains("\\"));
            assert!(!code.contains("/"));
        }
    }

    #[test]
    fn payload_serialization_boundary_is_stable() {
        let request = RoutingToggleOnRequest {
            trigger: RoutingToggleOnTrigger::UserToggle,
            requested_at_unix_ms: 1_700_000_000_000,
        };
        let serialized = serde_json::to_string(&request).expect("request should serialize");
        assert_eq!(
            serialized,
            r#"{"trigger":"user_toggle","requested_at_unix_ms":1700000000000}"#
        );

        let response = DetectionStatusResponse {
            state: DetectionStatusState::Detected,
            game_id: Some("ffxiv".to_string()),
            process_name: Some("ffxiv_dx11.exe".to_string()),
            detection_time_ms: Some(742),
            reason_code: None,
            message: None,
        };
        let json = serde_json::to_value(&response).expect("response should serialize");
        assert_eq!(json["state"], "detected");
        assert_eq!(json["game_id"], "ffxiv");
    }

    #[test]
    fn lifecycle_response_uses_normalized_state_and_reason_shape() {
        let response = RoutingLifecycleResponse {
            state: RoutingLifecycleState::Error,
            reason_code: Some("ipc_command_rejected".to_string()),
            message: Some("toggle action rejected".to_string()),
        };
        let json = serde_json::to_value(&response).expect("response should serialize");
        assert_eq!(json["state"], "error");
        assert_eq!(json["reason_code"], "ipc_command_rejected");
        assert_eq!(json["message"], "toggle action rejected");
    }
}
