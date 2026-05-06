use client_engine::metrics::{
    compute_ping_metrics, evaluate_metric_freshness, resolve_metrics_state, start_probe_loop,
    MetricFreshnessConfig, MetricFreshnessInput, MetricsState, ProbeExecutionCompletion,
    ProbeLoopConfig, ProbeLoopError, ProbeLoopErrorCode, ProbeLoopScheduler, ProbeObservation,
};
use client_engine::routing::RoutingState;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
enum StepAction {
    Success,
    Failure(ProbeLoopErrorCode),
    Tick,
}

#[derive(Debug, Clone, PartialEq)]
struct ProbeRecoveryStep {
    at_unix_ms: u64,
    action: StepAction,
    baseline_ping_ms: Option<f64>,
    routed_ping_ms: Option<f64>,
    routed_probe_attempted: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ProbeRecoveryScenario {
    name: String,
    stale_after_ms: u64,
    probe_interval_ms: u64,
    steps: Vec<ProbeRecoveryStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeRecoveryScenarioResult {
    pub name: String,
    pub trace: Vec<String>,
    pub terminal: String,
}

pub fn run_probe_recovery_suite() -> Vec<ProbeRecoveryScenarioResult> {
    let mut scenarios = load_probe_recovery_scenarios();
    scenarios.sort_by(|left, right| left.name.cmp(&right.name));
    scenarios
        .iter()
        .map(run_probe_recovery_scenario)
        .collect::<Vec<_>>()
}

fn run_probe_recovery_scenario(scenario: &ProbeRecoveryScenario) -> ProbeRecoveryScenarioResult {
    let mut scheduler = ProbeLoopScheduler::new(ProbeLoopConfig {
        interval_ms: scenario.probe_interval_ms,
        min_interval_ms: 100,
        max_interval_ms: 10_000,
    });
    let started = start_probe_loop(
        &mut scheduler,
        RoutingState::Connected,
        scenario
            .steps
            .first()
            .map(|step| step.at_unix_ms)
            .unwrap_or(0),
    );
    assert!(started, "probe loop should start in connected state");

    let mut latest_metrics = None;
    let mut last_success_sample_at = None;
    let mut recovery_count = 0_u32;
    let mut persist_count = 0_u32;
    let mut previous_state = MetricsState::Idle;
    let mut trace = Vec::new();

    for step in &scenario.steps {
        let mut probe_failed = false;
        let mut computation_error_code = None;
        let previous_last_known = latest_metrics.clone();

        match &step.action {
            StepAction::Success => {
                let decision = scheduler.begin_probe(step.at_unix_ms);
                let lease = match decision {
                    client_engine::metrics::ProbeLoopDecision::Started(lease) => lease,
                    other => panic!("expected started probe decision, got {other:?}"),
                };

                let observation = ProbeObservation {
                    baseline_ping_ms: step.baseline_ping_ms,
                    routed_ping_ms: step.routed_ping_ms,
                    routed_probe_attempted: step.routed_probe_attempted,
                };

                match compute_ping_metrics(&[observation]) {
                    Ok(metrics) => {
                        latest_metrics = Some(metrics);
                        last_success_sample_at = Some(step.at_unix_ms);
                        let completion = scheduler.complete_probe(lease, Ok(()));
                        assert_eq!(completion, ProbeExecutionCompletion::Success);
                    }
                    Err(error) => {
                        computation_error_code = Some(error.code);
                        probe_failed = true;
                        let completion = scheduler.complete_probe(
                            lease,
                            Err(ProbeLoopError::new(
                                ProbeLoopErrorCode::InvalidSample,
                                error.message,
                            )),
                        );
                        assert_eq!(
                            completion,
                            ProbeExecutionCompletion::Failed {
                                error_code: ProbeLoopErrorCode::InvalidSample
                            }
                        );
                    }
                }
            }
            StepAction::Failure(code) => {
                let decision = scheduler.begin_probe(step.at_unix_ms);
                let lease = match decision {
                    client_engine::metrics::ProbeLoopDecision::Started(lease) => lease,
                    other => panic!("expected started probe decision, got {other:?}"),
                };
                let completion = scheduler.complete_probe(
                    lease,
                    Err(ProbeLoopError::new(
                        *code,
                        format!("simulated probe failure: {code:?}"),
                    )),
                );
                assert_eq!(
                    completion,
                    ProbeExecutionCompletion::Failed { error_code: *code }
                );
                probe_failed = true;
            }
            StepAction::Tick => {}
        }

        let freshness = evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: step.at_unix_ms,
            last_sample_at_unix_ms: last_success_sample_at,
            config: MetricFreshnessConfig {
                stale_after_ms: scenario.stale_after_ms,
            },
        });
        let snapshot =
            resolve_metrics_state(&client_engine::metrics::MetricsStateResolutionInput {
                freshness,
                latest_metrics: latest_metrics.clone(),
                last_probe_failed: probe_failed,
                computation_error_code,
            });

        if matches!(previous_state, MetricsState::Degraded) && snapshot.state == MetricsState::Live
        {
            recovery_count = recovery_count.saturating_add(1);
        }

        if probe_failed && latest_metrics == previous_last_known && latest_metrics.is_some() {
            persist_count = persist_count.saturating_add(1);
        }

        let action_label = match &step.action {
            StepAction::Success => "success",
            StepAction::Failure(code) => match code {
                ProbeLoopErrorCode::Timeout => "failure_timeout",
                ProbeLoopErrorCode::TransportUnavailable => "failure_transport_unavailable",
                ProbeLoopErrorCode::InvalidSample => "failure_invalid_sample",
            },
            StepAction::Tick => "tick",
        };

        let reason_label = snapshot
            .reason_code
            .map(|reason| reason.as_str().to_string())
            .unwrap_or_else(|| "none".to_string());
        let routed_value = snapshot
            .latest_metrics
            .as_ref()
            .and_then(|metrics| metrics.routed_ping_ms)
            .map(format_metric_value)
            .unwrap_or_else(|| "--".to_string());
        let baseline_value = snapshot
            .latest_metrics
            .as_ref()
            .map(|metrics| format_metric_value(metrics.baseline_ping_ms))
            .unwrap_or_else(|| "--".to_string());

        trace.push(format!(
            "{}:{}:{}:{}:{}:{}",
            step.at_unix_ms,
            action_label,
            snapshot.state.as_str(),
            reason_label,
            baseline_value,
            routed_value
        ));

        previous_state = snapshot.state;
    }

    let terminal_state = trace
        .last()
        .and_then(|line| line.split(':').nth(2))
        .unwrap_or("idle");
    let terminal = format!(
        "terminal_state={};recoveries={};last_known_persisted={}",
        terminal_state,
        recovery_count,
        persist_count > 0
    );

    ProbeRecoveryScenarioResult {
        name: scenario.name.clone(),
        trace,
        terminal,
    }
}

fn load_probe_recovery_scenarios() -> Vec<ProbeRecoveryScenario> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("probe_recovery");
    let entries = fs::read_dir(base).expect("probe_recovery fixture directory should be readable");

    let mut scenarios = Vec::new();
    for entry in entries {
        let path = entry.expect("fixture entry should exist").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("scn") {
            continue;
        }

        let raw = fs::read_to_string(&path).expect("probe recovery fixture should be readable");
        scenarios.push(parse_probe_recovery_fixture(&raw));
    }
    scenarios
}

fn parse_probe_recovery_fixture(raw: &str) -> ProbeRecoveryScenario {
    let mut name = String::new();
    let mut stale_after_ms = 3_000_u64;
    let mut probe_interval_ms = 1_000_u64;
    let mut steps_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("fixture key should exist").trim();
        let value = parts.next().expect("fixture value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "stale_after_ms" => {
                stale_after_ms = value.parse().expect("stale_after_ms should parse")
            }
            "probe_interval_ms" => {
                probe_interval_ms = value.parse().expect("probe_interval_ms should parse")
            }
            "steps" => steps_raw = value.to_string(),
            _ => panic!("unknown probe recovery fixture key: {key}"),
        }
    }

    let steps = parse_probe_recovery_steps(&steps_raw);
    ProbeRecoveryScenario {
        name,
        stale_after_ms,
        probe_interval_ms,
        steps,
    }
}

fn parse_probe_recovery_steps(raw: &str) -> Vec<ProbeRecoveryStep> {
    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 6 {
                panic!("invalid probe recovery step token: {token}");
            }

            let action = match fields[1] {
                "success" => StepAction::Success,
                "failure_timeout" => StepAction::Failure(ProbeLoopErrorCode::Timeout),
                "failure_transport_unavailable" => {
                    StepAction::Failure(ProbeLoopErrorCode::TransportUnavailable)
                }
                "failure_invalid_sample" => StepAction::Failure(ProbeLoopErrorCode::InvalidSample),
                "tick" => StepAction::Tick,
                other => panic!("unknown probe recovery action: {other}"),
            };

            ProbeRecoveryStep {
                at_unix_ms: fields[0].parse().expect("step timestamp should parse"),
                action,
                baseline_ping_ms: parse_optional_f64(fields[2]),
                routed_ping_ms: parse_optional_f64(fields[3]),
                routed_probe_attempted: parse_optional_bool(fields[4]).unwrap_or(false),
            }
        })
        .collect()
}

fn parse_optional_f64(raw: &str) -> Option<f64> {
    if raw == "_" {
        return None;
    }
    Some(raw.parse().expect("optional float should parse"))
}

fn parse_optional_bool(raw: &str) -> Option<bool> {
    if raw == "_" {
        return None;
    }
    Some(raw.parse().expect("optional bool should parse"))
}

fn format_metric_value(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }
    format!("{value:.1}")
}

#[test]
fn harness_simulates_intermittent_probe_failure_and_recovery() {
    let results = run_probe_recovery_suite();
    let scenario = results
        .iter()
        .find(|result| result.name == "intermittent_probe_failure_recovery")
        .expect("intermittent recovery scenario should exist");

    assert!(scenario
        .trace
        .iter()
        .any(|line| line.contains("failure_timeout:degraded:probe_failed")));
    assert!(scenario
        .trace
        .iter()
        .any(|line| line.contains("failure_transport_unavailable:degraded:probe_failed")));
    assert!(scenario
        .trace
        .iter()
        .any(|line| line.contains(":success:live:none:")));
}

#[test]
fn degraded_to_live_recovery_path_is_deterministic() {
    let first = run_probe_recovery_suite();
    let second = run_probe_recovery_suite();
    assert_eq!(first, second);

    let scenario = first
        .iter()
        .find(|result| result.name == "intermittent_probe_failure_recovery")
        .expect("intermittent recovery scenario should exist");
    assert!(scenario.terminal.contains("recoveries=1"));
    assert!(scenario.terminal.contains("terminal_state=live"));
}

#[test]
fn last_known_values_persist_during_failure_window() {
    let results = run_probe_recovery_suite();
    let scenario = results
        .iter()
        .find(|result| result.name == "stale_window_with_last_known_persistence")
        .expect("last-known scenario should exist");

    let first_failure = scenario
        .trace
        .iter()
        .find(|line| line.contains("failure_timeout"))
        .expect("scenario should include failure_timeout step");
    let stale_tick = scenario
        .trace
        .iter()
        .find(|line| line.contains(":tick:degraded:freshness_timeout"))
        .expect("scenario should include stale degraded tick");

    // Value columns preserve the last successful sample while failures happen.
    assert!(first_failure.ends_with(":208:150"));
    assert!(stale_tick.ends_with(":208:150"));
    assert!(scenario.terminal.contains("last_known_persisted=true"));
}

#[test]
fn recovery_trace_output_is_reproducible() {
    let first = run_probe_recovery_suite();
    let second = run_probe_recovery_suite();
    assert_eq!(first, second);
}
