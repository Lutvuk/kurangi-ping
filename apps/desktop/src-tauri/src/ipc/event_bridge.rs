use std::sync::Mutex;

use client_engine::metrics::{MetricsState, MetricsStateReasonCode, MetricsStateSnapshot};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use super::contracts::{
    DetectionStatusResponse, DetectionStatusState, DetectionStatusUpdatedEventPayload, IpcEvent,
    MetricsEmissionState, MetricsPingSampledEventPayload, RoutingLifecycleResponse,
    RoutingLifecycleState, RoutingStateChangedEventPayload,
};
use super::error_map::{map_ipc_error_reason, map_metrics_reason_kind};

#[derive(Debug, Clone, PartialEq)]
pub struct EmittedIpcEvent {
    pub sequence: u64,
    pub name: String,
    pub payload: Value,
}

#[derive(Debug, Default)]
pub struct EventBridgeState {
    sequence: Mutex<u64>,
}

impl EventBridgeState {
    pub fn next_sequence(&self) -> Result<u64, String> {
        self.sequence
            .lock()
            .map(|mut value| {
                *value = value.saturating_add(1);
                *value
            })
            .map_err(|_| "ipc event sequence state is unavailable".to_string())
    }
}

pub trait IpcEventEmitter {
    fn emit_json(&self, event_name: &str, payload: Value) -> Result<(), String>;
}

impl IpcEventEmitter for AppHandle {
    fn emit_json(&self, event_name: &str, payload: Value) -> Result<(), String> {
        self.emit(event_name, payload)
            .map_err(|_| "failed to emit ipc event".to_string())
    }
}

pub fn emit_routing_state_changed<E: IpcEventEmitter>(
    emitter: &E,
    state: &EventBridgeState,
    previous_state: RoutingLifecycleState,
    response: &RoutingLifecycleResponse,
) -> Result<EmittedIpcEvent, String> {
    let payload = RoutingStateChangedEventPayload {
        previous_state,
        state: response.state,
        reason_code: response.reason_code.clone(),
        message: response.message.clone(),
    };

    emit_event(emitter, state, IpcEvent::RoutingStateChanged, &payload)
}

pub fn emit_metrics_ping_sampled<E: IpcEventEmitter>(
    emitter: &E,
    state: &EventBridgeState,
    sampled_at_unix_ms: u64,
    snapshot: &MetricsStateSnapshot,
) -> Result<EmittedIpcEvent, String> {
    let payload = map_metrics_snapshot(sampled_at_unix_ms, snapshot);
    emit_event(emitter, state, IpcEvent::MetricsPingSampled, &payload)
}

pub fn emit_detection_status_updated<E: IpcEventEmitter>(
    emitter: &E,
    state: &EventBridgeState,
    response: &DetectionStatusResponse,
) -> Result<EmittedIpcEvent, String> {
    let payload = DetectionStatusUpdatedEventPayload {
        state: response.state,
        game_id: response.game_id.clone(),
        process_name: response.process_name.clone(),
        detection_time_ms: response.detection_time_ms,
        reason_code: response.reason_code.clone(),
        message: response.message.clone(),
    };

    emit_event(emitter, state, IpcEvent::DetectionStatusUpdated, &payload)
}

pub fn map_metrics_snapshot(
    sampled_at_unix_ms: u64,
    snapshot: &MetricsStateSnapshot,
) -> MetricsPingSampledEventPayload {
    let (state, reason_code) = map_metrics_state(snapshot.state, snapshot.reason_code);
    let latest = snapshot.latest_metrics.as_ref();

    MetricsPingSampledEventPayload {
        sampled_at_unix_ms,
        state,
        baseline_ping_ms: latest.map(|entry| entry.baseline_ping_ms),
        routed_ping_ms: latest.and_then(|entry| entry.routed_ping_ms),
        jitter_ms: latest.and_then(|entry| entry.jitter_ms),
        packet_loss_pct: latest.and_then(|entry| entry.packet_loss_pct),
        reason_code: reason_code.map(str::to_string),
    }
}

fn map_metrics_state(
    state: MetricsState,
    reason_code: Option<MetricsStateReasonCode>,
) -> (MetricsEmissionState, Option<&'static str>) {
    let normalized_state = match state {
        MetricsState::Live => MetricsEmissionState::Live,
        MetricsState::Idle => MetricsEmissionState::Measuring,
        MetricsState::Degraded | MetricsState::Error => MetricsEmissionState::Degraded,
    };

    let normalized_reason = map_metrics_reason_kind(state, reason_code).map(map_ipc_error_reason);

    (normalized_state, normalized_reason)
}

fn emit_event<E: IpcEventEmitter, P: Serialize>(
    emitter: &E,
    state: &EventBridgeState,
    event: IpcEvent,
    payload: &P,
) -> Result<EmittedIpcEvent, String> {
    let sequence = state.next_sequence()?;
    let payload_json = serde_json::to_value(payload)
        .map_err(|_| "failed to serialize ipc event payload".to_string())?;
    emitter.emit_json(event.as_str(), payload_json.clone())?;

    Ok(EmittedIpcEvent {
        sequence,
        name: event.as_str().to_string(),
        payload: payload_json,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        emit_detection_status_updated, emit_metrics_ping_sampled, emit_routing_state_changed,
        map_metrics_snapshot, DetectionStatusResponse, DetectionStatusState, EmittedIpcEvent,
        EventBridgeState, IpcEventEmitter, RoutingLifecycleResponse, RoutingLifecycleState,
    };
    use client_engine::metrics::{
        MetricFreshnessResult, MetricFreshnessStatus, MetricsState, MetricsStateReasonCode,
        MetricsStateSnapshot, PingMetrics,
    };
    use serde_json::Value;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct RecorderEmitter {
        entries: Mutex<Vec<EmittedIpcEvent>>,
    }

    impl RecorderEmitter {
        fn snapshot(&self) -> Vec<EmittedIpcEvent> {
            self.entries
                .lock()
                .expect("entries lock should be readable")
                .clone()
        }
    }

    impl IpcEventEmitter for RecorderEmitter {
        fn emit_json(&self, event_name: &str, payload: Value) -> Result<(), String> {
            self.entries
                .lock()
                .map_err(|_| "entries lock unavailable".to_string())?
                .push(EmittedIpcEvent {
                    sequence: 0,
                    name: event_name.to_string(),
                    payload,
                });
            Ok(())
        }
    }

    fn sample_snapshot(
        state: MetricsState,
        reason: Option<MetricsStateReasonCode>,
    ) -> MetricsStateSnapshot {
        MetricsStateSnapshot {
            state,
            reason_code: reason,
            freshness: MetricFreshnessResult {
                status: MetricFreshnessStatus::Fresh,
                stale_after_ms: 5_000,
                sample_age_ms: Some(250),
            },
            latest_metrics: Some(PingMetrics {
                baseline_ping_ms: 200.0,
                routed_ping_ms: Some(150.0),
                jitter_ms: Some(3.2),
                packet_loss_pct: Some(0.5),
                sample_count: 8,
            }),
        }
    }

    #[test]
    fn routing_and_detection_events_emit_in_deterministic_order() {
        let emitter = RecorderEmitter::default();
        let sequence = EventBridgeState::default();

        let first = emit_routing_state_changed(
            &emitter,
            &sequence,
            RoutingLifecycleState::Idle,
            &RoutingLifecycleResponse {
                state: RoutingLifecycleState::Connecting,
                reason_code: None,
                message: None,
            },
        )
        .expect("routing event should emit");

        let second = emit_detection_status_updated(
            &emitter,
            &sequence,
            &DetectionStatusResponse {
                state: DetectionStatusState::Detected,
                game_id: Some("ffxiv".to_string()),
                process_name: Some("ffxiv_dx11.exe".to_string()),
                detection_time_ms: Some(170_000),
                reason_code: None,
                message: None,
            },
        )
        .expect("detection event should emit");

        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);

        let events = emitter.snapshot();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name, "routing_state_changed");
        assert_eq!(events[1].name, "detection_status_updated");
    }

    #[test]
    fn metrics_snapshot_maps_to_normalized_payload_fields() {
        let snapshot = sample_snapshot(
            MetricsState::Degraded,
            Some(MetricsStateReasonCode::FreshnessTimeout),
        );

        let payload = map_metrics_snapshot(1_700_000_010_000, &snapshot);
        assert_eq!(payload.sampled_at_unix_ms, 1_700_000_010_000);
        assert_eq!(payload.state, super::MetricsEmissionState::Degraded);
        assert_eq!(payload.baseline_ping_ms, Some(200.0));
        assert_eq!(payload.routed_ping_ms, Some(150.0));
        assert_eq!(payload.reason_code.as_deref(), Some("ipc_timeout"));
    }

    #[test]
    fn metrics_event_emits_with_stable_reason_fallback() {
        let emitter = RecorderEmitter::default();
        let sequence = EventBridgeState::default();
        let snapshot = sample_snapshot(
            MetricsState::Error,
            Some(MetricsStateReasonCode::InvalidComputationWindow),
        );

        let emitted = emit_metrics_ping_sampled(&emitter, &sequence, 1_700_000_020_000, &snapshot)
            .expect("metrics event should emit");
        assert_eq!(emitted.sequence, 1);
        assert_eq!(emitted.name, "metrics_ping_sampled");
        assert_eq!(
            emitted.payload["reason_code"].as_str(),
            Some("ipc_metrics_stream_unavailable")
        );
    }

    #[test]
    fn repeated_emission_runs_keep_deterministic_shapes() {
        fn run() -> Vec<(String, Option<String>)> {
            let emitter = RecorderEmitter::default();
            let sequence = EventBridgeState::default();

            let _ = emit_routing_state_changed(
                &emitter,
                &sequence,
                RoutingLifecycleState::Connecting,
                &RoutingLifecycleResponse {
                    state: RoutingLifecycleState::Active,
                    reason_code: None,
                    message: None,
                },
            )
            .expect("routing event should emit");

            let _ = emit_metrics_ping_sampled(
                &emitter,
                &sequence,
                1_700_000_030_000,
                &sample_snapshot(
                    MetricsState::Idle,
                    Some(MetricsStateReasonCode::NoSamplesYet),
                ),
            )
            .expect("metrics event should emit");

            let _ = emit_detection_status_updated(
                &emitter,
                &sequence,
                &DetectionStatusResponse {
                    state: DetectionStatusState::NotDetected,
                    game_id: None,
                    process_name: None,
                    detection_time_ms: Some(1_700_000_040_000),
                    reason_code: None,
                    message: None,
                },
            )
            .expect("detection event should emit");

            emitter
                .snapshot()
                .iter()
                .map(|entry| {
                    (
                        entry.name.clone(),
                        entry
                            .payload
                            .get("reason_code")
                            .and_then(Value::as_str)
                            .map(ToString::to_string),
                    )
                })
                .collect()
        }

        let first = run();
        let second = run();
        assert_eq!(first, second);
    }
}
