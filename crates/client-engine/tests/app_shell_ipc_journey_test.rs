use client_engine::detection::commands::{trigger_rescan, RescanCommandConfig, RescanCommandHandler, TimeProvider};
use client_engine::detection::scanner_windows::{
    ProcessEntry, ProcessEnumerator, ScanError, SupportedGame,
};
use client_engine::metrics::{
    compute_ping_metrics, evaluate_metric_freshness, resolve_metrics_state, MetricFreshnessConfig,
    MetricFreshnessInput, MetricsComputationErrorCode, MetricsState, MetricsStateReasonCode,
    MetricsStateResolutionInput, ProbeObservation,
};
use client_engine::routing::{
    handle_toggle_command, RoutingStateMachine, RoutingTrigger, ToggleCommand, ToggleResultCode,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct IpcJourneyResult {
    name: String,
    trace: Vec<String>,
    snapshots: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectionIpcPayload {
    state: String,
    game_id: Option<String>,
    process_name: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct MetricsIpcPayload {
    state: String,
    baseline_ping_ms: Option<f64>,
    routed_ping_ms: Option<f64>,
    jitter_ms: Option<f64>,
    packet_loss_pct: Option<f64>,
    reason_code: Option<String>,
}

#[derive(Clone)]
struct FixedClock {
    now_unix_ms: u64,
}

impl TimeProvider for FixedClock {
    fn now_unix_ms(&self) -> u64 {
        self.now_unix_ms
    }
}

struct StubScanner {
    processes: Vec<ProcessEntry>,
}

impl ProcessEnumerator for StubScanner {
    fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
        Ok(self.processes.clone())
    }
}

fn run_app_shell_ipc_journey_suite() -> Vec<IpcJourneyResult> {
    vec![
        run_nominal_ipc_journey(),
        run_detection_not_detected_journey(),
        run_metrics_degraded_journey(),
    ]
}

fn run_nominal_ipc_journey() -> IpcJourneyResult {
    let mut trace = Vec::new();
    let mut snapshots = BTreeMap::new();

    // 1) Frontend invokes routing ON command and backend state machine moves to connecting.
    let mut machine = RoutingStateMachine::new();
    trace.push("invoke:routing_toggle_on".to_string());
    let toggle_result = handle_toggle_command(&mut machine, ToggleCommand::On);
    assert_eq!(toggle_result.code, ToggleResultCode::Applied);
    let connecting_event_state = map_routing_state_to_ipc(toggle_result.to_state.as_str());
    trace.push(format!(
        "event:routing_state_changed:{}->{}",
        map_routing_state_to_ipc(toggle_result.from_state.as_str()),
        connecting_event_state
    ));

    // 2) Backend lifecycle continues and emits active state when connection is established.
    let established = machine
        .transition(RoutingTrigger::ConnectionEstablished, None)
        .expect("connecting -> connected transition should be legal");
    let active_event_state = map_routing_state_to_ipc(established.to.as_str());
    trace.push(format!(
        "event:routing_state_changed:{}->{}",
        map_routing_state_to_ipc(established.from.as_str()),
        active_event_state
    ));
    snapshots.insert("routing_active_state".to_string(), active_event_state.to_string());

    // 3) Frontend requests detection startup status and backend maps scanner result.
    trace.push("invoke:detection_get_status".to_string());
    let detection_handler = RescanCommandHandler::new(
        StubScanner {
            processes: vec![ProcessEntry {
                process_id: 4242,
                executable_name: "ffxiv_dx11.exe".to_string(),
            }],
        },
        FixedClock {
            now_unix_ms: 1_700_000_100_000,
        },
        RescanCommandConfig::default(),
    );
    let detection_output = trigger_rescan(&detection_handler, &supported_games())
        .expect("detection scan should complete deterministically");
    let detection_payload = map_detection_to_ipc_payload(&detection_output.resolution);
    trace.push(format!(
        "event:detection_status_updated:{}",
        detection_payload.state
    ));
    snapshots.insert(
        "detection_startup_state".to_string(),
        detection_payload.state.clone(),
    );

    // 4) Backend emits continuous metrics stream updates.
    let live_metrics = compute_ping_metrics(&[
        ProbeObservation {
            baseline_ping_ms: Some(212.0),
            routed_ping_ms: Some(154.0),
            routed_probe_attempted: true,
        },
        ProbeObservation {
            baseline_ping_ms: Some(210.0),
            routed_ping_ms: Some(152.0),
            routed_probe_attempted: true,
        },
    ])
    .expect("nominal metrics should compute");

    let live_snapshot = resolve_metrics_state(&MetricsStateResolutionInput {
        freshness: evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: 1_700_000_100_500,
            last_sample_at_unix_ms: Some(1_700_000_100_500),
            config: MetricFreshnessConfig::default(),
        }),
        latest_metrics: Some(live_metrics.clone()),
        last_probe_failed: false,
        computation_error_code: None,
    });
    let first_payload = map_metrics_to_ipc_payload(&live_snapshot);
    trace.push(format!("event:metrics_ping_sampled:{}", first_payload.state));

    let second_snapshot = resolve_metrics_state(&MetricsStateResolutionInput {
        freshness: evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: 1_700_000_101_000,
            last_sample_at_unix_ms: Some(1_700_000_101_000),
            config: MetricFreshnessConfig::default(),
        }),
        latest_metrics: Some(live_metrics),
        last_probe_failed: false,
        computation_error_code: None,
    });
    let second_payload = map_metrics_to_ipc_payload(&second_snapshot);
    trace.push(format!("event:metrics_ping_sampled:{}", second_payload.state));
    snapshots.insert("metrics_stream_state".to_string(), second_payload.state.clone());

    IpcJourneyResult {
        name: "nominal_toggle_detection_metrics".to_string(),
        trace,
        snapshots,
    }
}

fn run_detection_not_detected_journey() -> IpcJourneyResult {
    let mut trace = Vec::new();
    let mut snapshots = BTreeMap::new();
    trace.push("invoke:detection_get_status".to_string());

    let handler = RescanCommandHandler::new(
        StubScanner {
            processes: vec![ProcessEntry {
                process_id: 777,
                executable_name: "explorer.exe".to_string(),
            }],
        },
        FixedClock {
            now_unix_ms: 1_700_000_200_000,
        },
        RescanCommandConfig::default(),
    );
    let output = trigger_rescan(&handler, &supported_games()).expect("scan should complete");
    let payload = map_detection_to_ipc_payload(&output.resolution);
    trace.push(format!("event:detection_status_updated:{}", payload.state));
    snapshots.insert("detection_startup_state".to_string(), payload.state);

    IpcJourneyResult {
        name: "detection_not_detected_contract".to_string(),
        trace,
        snapshots,
    }
}

fn run_metrics_degraded_journey() -> IpcJourneyResult {
    let mut trace = Vec::new();
    let mut snapshots = BTreeMap::new();

    let snapshot = resolve_metrics_state(&MetricsStateResolutionInput {
        freshness: evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: 1_700_000_300_000,
            last_sample_at_unix_ms: Some(1_700_000_290_000),
            config: MetricFreshnessConfig::default(),
        }),
        latest_metrics: None,
        last_probe_failed: false,
        computation_error_code: Some(MetricsComputationErrorCode::InvalidObservationWindow),
    });
    let payload = map_metrics_to_ipc_payload(&snapshot);
    trace.push(format!("event:metrics_ping_sampled:{}", payload.state));
    trace.push(format!(
        "event:metrics_ping_sampled_reason:{}",
        payload
            .reason_code
            .clone()
            .unwrap_or_else(|| "none".to_string())
    ));

    snapshots.insert("metrics_stream_state".to_string(), payload.state);
    snapshots.insert(
        "metrics_stream_reason".to_string(),
        payload
            .reason_code
            .unwrap_or_else(|| "none".to_string()),
    );

    IpcJourneyResult {
        name: "metrics_error_to_degraded_event".to_string(),
        trace,
        snapshots,
    }
}

fn supported_games() -> Vec<SupportedGame> {
    vec![SupportedGame {
        game_id: "ffxiv".to_string(),
        executable_name: "ffxiv_dx11.exe".to_string(),
        enabled: true,
    }]
}

fn map_routing_state_to_ipc(state: &str) -> &'static str {
    match state {
        "off" => "idle",
        "connecting" => "connecting",
        "connected" => "active",
        "degraded" => "degraded",
        "failed" => "error",
        _ => "error",
    }
}

fn map_detection_to_ipc_payload(
    resolution: &client_engine::detection::state_resolver::DetectionResolution,
) -> DetectionIpcPayload {
    match resolution.state {
        client_engine::detection::state_resolver::DetectionState::Detected => DetectionIpcPayload {
            state: "detected".to_string(),
            game_id: resolution.metadata.matched_game_id.clone(),
            process_name: resolution.metadata.matched_executable_name.clone(),
            reason_code: None,
        },
        client_engine::detection::state_resolver::DetectionState::NotFound
        | client_engine::detection::state_resolver::DetectionState::Stale => DetectionIpcPayload {
            state: "not_detected".to_string(),
            game_id: None,
            process_name: None,
            reason_code: None,
        },
        client_engine::detection::state_resolver::DetectionState::Error => DetectionIpcPayload {
            state: "not_detected".to_string(),
            game_id: None,
            process_name: None,
            reason_code: Some("ipc_detection_scan_failed".to_string()),
        },
    }
}

fn map_metrics_to_ipc_payload(
    snapshot: &client_engine::metrics::MetricsStateSnapshot,
) -> MetricsIpcPayload {
    let state = match snapshot.state {
        MetricsState::Live => "live",
        MetricsState::Idle => "measuring",
        MetricsState::Degraded | MetricsState::Error => "degraded",
    };
    let reason_code = match snapshot.reason_code {
        Some(MetricsStateReasonCode::FreshnessTimeout) => Some("ipc_timeout".to_string()),
        Some(_) => Some("ipc_metrics_stream_unavailable".to_string()),
        None if matches!(snapshot.state, MetricsState::Error) => {
            Some("ipc_metrics_stream_unavailable".to_string())
        }
        None => None,
    };
    let metrics = snapshot.latest_metrics.as_ref();

    MetricsIpcPayload {
        state: state.to_string(),
        baseline_ping_ms: metrics.map(|entry| entry.baseline_ping_ms),
        routed_ping_ms: metrics.and_then(|entry| entry.routed_ping_ms),
        jitter_ms: metrics.and_then(|entry| entry.jitter_ms),
        packet_loss_pct: metrics.and_then(|entry| entry.packet_loss_pct),
        reason_code,
    }
}

#[test]
fn success_path_validates_toggle_on_to_active_state_event() {
    let suite = run_app_shell_ipc_journey_suite();
    let nominal = suite
        .iter()
        .find(|entry| entry.name == "nominal_toggle_detection_metrics")
        .expect("nominal scenario should exist");

    assert!(nominal
        .trace
        .iter()
        .any(|line| line == "invoke:routing_toggle_on"));
    assert!(nominal
        .trace
        .iter()
        .any(|line| line == "event:routing_state_changed:idle->connecting"));
    assert!(nominal
        .trace
        .iter()
        .any(|line| line == "event:routing_state_changed:connecting->active"));
    assert_eq!(
        nominal.snapshots.get("routing_active_state"),
        Some(&"active".to_string())
    );
}

#[test]
fn detection_startup_query_path_matches_frontend_contract_shape() {
    let suite = run_app_shell_ipc_journey_suite();
    let nominal = suite
        .iter()
        .find(|entry| entry.name == "nominal_toggle_detection_metrics")
        .expect("nominal scenario should exist");
    let not_detected = suite
        .iter()
        .find(|entry| entry.name == "detection_not_detected_contract")
        .expect("not-detected scenario should exist");

    assert_eq!(
        nominal.snapshots.get("detection_startup_state"),
        Some(&"detected".to_string())
    );
    assert_eq!(
        not_detected.snapshots.get("detection_startup_state"),
        Some(&"not_detected".to_string())
    );
}

#[test]
fn metrics_event_path_validates_continuous_live_updates() {
    let suite = run_app_shell_ipc_journey_suite();
    let nominal = suite
        .iter()
        .find(|entry| entry.name == "nominal_toggle_detection_metrics")
        .expect("nominal scenario should exist");
    let live_events = nominal
        .trace
        .iter()
        .filter(|line| line.as_str() == "event:metrics_ping_sampled:live")
        .count();
    assert_eq!(live_events, 2, "continuous metrics should emit repeatable live updates");

    let degraded = suite
        .iter()
        .find(|entry| entry.name == "metrics_error_to_degraded_event")
        .expect("degraded scenario should exist");
    assert_eq!(
        degraded.snapshots.get("metrics_stream_state"),
        Some(&"degraded".to_string())
    );
    assert_eq!(
        degraded.snapshots.get("metrics_stream_reason"),
        Some(&"ipc_metrics_stream_unavailable".to_string())
    );
}

#[test]
fn journey_traces_are_deterministic_across_repeated_runs() {
    let first = run_app_shell_ipc_journey_suite();
    let second = run_app_shell_ipc_journey_suite();
    assert_eq!(first, second);
}
