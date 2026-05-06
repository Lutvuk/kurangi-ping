use client_engine::routing::{
    should_failover, switch_active_relay, ActiveRelaySwitchAdapter, AttemptFailureReason,
    AttemptStepOutcome, FailoverEvaluationInput, FailoverExecutionContext, FailoverExecutionState,
    FailoverReasonCode, FailoverSwitchError, FailoverTriggerConfig, FailoverTriggerState,
    RelayHealthStatus, RetryPolicy, RouteCandidate, RouteProtocol,
};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaosScenarioResult {
    pub name: String,
    pub trace: Vec<String>,
    pub terminal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChaosScenarioKind {
    Trigger,
    Executor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriggerStep {
    now_unix_ms: u64,
    status: RelayHealthStatus,
    poll_failure_streak: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TriggerScenario {
    name: String,
    config: FailoverTriggerConfig,
    steps: Vec<TriggerStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutorScenario {
    name: String,
    context: FailoverExecutionContext,
    outcomes: Vec<AttemptStepOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChaosScenario {
    Trigger(TriggerScenario),
    Executor(ExecutorScenario),
}

#[derive(Debug, Default)]
struct HarnessSwitchAdapter {
    outcomes: VecDeque<AttemptStepOutcome>,
    backoff_delays: Vec<u64>,
    operations: Vec<String>,
}

impl ActiveRelaySwitchAdapter for HarnessSwitchAdapter {
    fn teardown_active_path(&mut self, active_relay_id: &str) -> Result<(), FailoverSwitchError> {
        self.operations.push(format!("teardown:{active_relay_id}"));
        Ok(())
    }

    fn establish_candidate_path(
        &mut self,
        protocol: RouteProtocol,
        candidate: &RouteCandidate,
    ) -> AttemptStepOutcome {
        self.operations.push(format!(
            "establish:{}:{}",
            protocol_as_str(protocol),
            candidate.relay_id
        ));
        self.outcomes
            .pop_front()
            .unwrap_or(AttemptStepOutcome::Failed(AttemptFailureReason::Timeout))
    }

    fn promote_candidate_path(
        &mut self,
        candidate: &RouteCandidate,
        protocol: RouteProtocol,
    ) -> Result<(), FailoverSwitchError> {
        self.operations.push(format!(
            "promote:{}:{}",
            protocol_as_str(protocol),
            candidate.relay_id
        ));
        Ok(())
    }

    fn rollback_candidate_path(&mut self, candidate: &RouteCandidate) {
        self.operations
            .push(format!("rollback:{}", candidate.relay_id));
    }

    fn wait_backoff(&mut self, delay_ms: u64) {
        self.backoff_delays.push(delay_ms);
        self.operations.push(format!("backoff:{delay_ms}"));
    }
}

pub fn run_failover_chaos_suite() -> Vec<ChaosScenarioResult> {
    let mut scenarios = load_chaos_scenarios();
    scenarios.sort_by(|left, right| scenario_name(left).cmp(scenario_name(right)));

    scenarios
        .iter()
        .map(|scenario| match scenario {
            ChaosScenario::Trigger(definition) => run_trigger_scenario(definition),
            ChaosScenario::Executor(definition) => run_executor_scenario(definition),
        })
        .collect()
}

fn run_trigger_scenario(scenario: &TriggerScenario) -> ChaosScenarioResult {
    let mut state = FailoverTriggerState::default();
    let mut failover_count = 0_u32;
    let mut hysteresis_blocks = 0_u32;
    let mut recovered_after_failover = false;
    let mut trace = Vec::new();

    for step in &scenario.steps {
        let decision = should_failover(
            &mut state,
            FailoverEvaluationInput {
                active_relay_status: step.status,
                poll_failure_streak: step.poll_failure_streak,
                now_unix_ms: step.now_unix_ms,
            },
            &scenario.config,
        );

        if decision.should_failover {
            failover_count = failover_count.saturating_add(1);
        }
        if decision.reason_code == FailoverReasonCode::HysteresisWindowActive {
            hysteresis_blocks = hysteresis_blocks.saturating_add(1);
        }
        if failover_count > 0
            && step.status == RelayHealthStatus::Ok
            && decision.reason_code == FailoverReasonCode::HealthyNoTrigger
        {
            recovered_after_failover = true;
        }

        trace.push(format!(
            "{}:{}:{}:{}:{}",
            step.now_unix_ms,
            health_as_str(step.status),
            step.poll_failure_streak,
            decision.reason_code.as_str(),
            decision.should_failover
        ));
    }

    let terminal = format!(
        "kind=trigger;failovers={};hysteresis_blocks={};recovered_after_failover={}",
        failover_count, hysteresis_blocks, recovered_after_failover
    );

    ChaosScenarioResult {
        name: scenario.name.clone(),
        trace,
        terminal,
    }
}

fn run_executor_scenario(scenario: &ExecutorScenario) -> ChaosScenarioResult {
    let mut adapter = HarnessSwitchAdapter {
        outcomes: scenario.outcomes.clone().into(),
        ..HarnessSwitchAdapter::default()
    };
    let result = switch_active_relay(&scenario.context, &mut adapter);
    let mut trace = adapter.operations;
    trace.extend(
        adapter
            .backoff_delays
            .iter()
            .map(|delay| format!("delay:{delay}")),
    );

    let terminal = match result.state {
        FailoverExecutionState::Switched { protocol } => format!(
            "kind=executor;terminal=switched;protocol={};attempts={}",
            protocol_as_str(protocol),
            result.attempts
        ),
        FailoverExecutionState::Failed { failure_code, .. } => format!(
            "kind=executor;terminal=failed;failure_code={};attempts={};retries={}",
            failure_code.as_code(),
            result.attempts,
            result.retry_metadata.len()
        ),
    };

    ChaosScenarioResult {
        name: scenario.name.clone(),
        trace,
        terminal,
    }
}

fn load_chaos_scenarios() -> Vec<ChaosScenario> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("relay_chaos");
    let entries = fs::read_dir(base).expect("relay_chaos fixture directory should be readable");

    let mut scenarios = Vec::new();
    for entry in entries {
        let path = entry.expect("fixture entry should exist").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("scn") {
            continue;
        }

        let raw = fs::read_to_string(&path).expect("chaos fixture should be readable");
        scenarios.push(parse_chaos_fixture(&raw));
    }
    scenarios
}

fn parse_chaos_fixture(raw: &str) -> ChaosScenario {
    let mut name = String::new();
    let mut kind = ChaosScenarioKind::Trigger;
    let mut grace_polls = 3_u32;
    let mut poll_failure_threshold = 2_u32;
    let mut hysteresis_ms = 15_000_u64;
    let mut dead_immediate = true;
    let mut steps_raw = String::new();

    let mut active_relay_id = String::new();
    let mut candidates_raw = String::new();
    let mut protocols_raw = String::new();
    let mut max_attempts_per_protocol = 2_u8;
    let mut base_backoff_ms = 250_u64;
    let mut max_backoff_ms = 3_000_u64;
    let mut outcomes_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("fixture key should exist").trim();
        let value = parts.next().expect("fixture value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "kind" => {
                kind = match value {
                    "trigger" => ChaosScenarioKind::Trigger,
                    "executor" => ChaosScenarioKind::Executor,
                    _ => panic!("unknown chaos kind: {value}"),
                }
            }
            "grace_polls" => grace_polls = value.parse().expect("grace_polls should parse"),
            "poll_failure_threshold" => {
                poll_failure_threshold = value.parse().expect("poll_failure_threshold should parse")
            }
            "hysteresis_ms" => hysteresis_ms = value.parse().expect("hysteresis_ms should parse"),
            "dead_immediate" => {
                dead_immediate = value.parse().expect("dead_immediate should parse")
            }
            "steps" => steps_raw = value.to_string(),
            "active_relay_id" => active_relay_id = value.to_string(),
            "candidates" => candidates_raw = value.to_string(),
            "protocols" => protocols_raw = value.to_string(),
            "max_attempts_per_protocol" => {
                max_attempts_per_protocol = value
                    .parse()
                    .expect("max_attempts_per_protocol should parse")
            }
            "base_backoff_ms" => {
                base_backoff_ms = value.parse().expect("base_backoff_ms should parse")
            }
            "max_backoff_ms" => {
                max_backoff_ms = value.parse().expect("max_backoff_ms should parse")
            }
            "outcomes" => outcomes_raw = value.to_string(),
            _ => panic!("unknown chaos fixture key: {key}"),
        }
    }

    match kind {
        ChaosScenarioKind::Trigger => {
            let steps = parse_trigger_steps(&steps_raw);
            ChaosScenario::Trigger(TriggerScenario {
                name,
                config: FailoverTriggerConfig {
                    degraded_grace_polls: grace_polls,
                    poll_failure_streak_threshold: poll_failure_threshold,
                    hysteresis_window_ms: hysteresis_ms,
                    dead_relay_triggers_immediately: dead_immediate,
                },
                steps,
            })
        }
        ChaosScenarioKind::Executor => {
            let candidates = parse_candidates(&candidates_raw);
            let protocol_order = parse_protocols(&protocols_raw);
            let outcomes = parse_outcomes(&outcomes_raw);
            ChaosScenario::Executor(ExecutorScenario {
                name,
                context: FailoverExecutionContext {
                    active_relay_id,
                    candidates,
                    protocol_order,
                    retry_policy: RetryPolicy {
                        max_attempts_per_protocol,
                        base_backoff_ms,
                        max_backoff_ms,
                    },
                },
                outcomes,
            })
        }
    }
}

fn parse_trigger_steps(raw: &str) -> Vec<TriggerStep> {
    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 3 {
                panic!("invalid trigger step: {token}");
            }
            TriggerStep {
                now_unix_ms: fields[0].parse().expect("step timestamp should parse"),
                status: parse_health_status(fields[1]),
                poll_failure_streak: fields[2].parse().expect("poll failure streak should parse"),
            }
        })
        .collect()
}

fn parse_candidates(raw: &str) -> Vec<RouteCandidate> {
    raw.split(',')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 4 {
                panic!("invalid candidate token: {token}");
            }
            RouteCandidate {
                relay_id: fields[0].to_string(),
                region: fields[1].to_string(),
                hostname: fields[2].to_string(),
                priority: fields[3].parse().expect("candidate priority should parse"),
            }
        })
        .collect()
}

fn parse_protocols(raw: &str) -> Vec<RouteProtocol> {
    raw.split(',')
        .filter(|token| !token.trim().is_empty())
        .map(|token| parse_protocol(token.trim()))
        .collect()
}

fn parse_outcomes(raw: &str) -> Vec<AttemptStepOutcome> {
    raw.split(',')
        .filter(|token| !token.trim().is_empty())
        .map(|token| parse_outcome(token.trim()))
        .collect()
}

fn parse_health_status(value: &str) -> RelayHealthStatus {
    match value {
        "ok" => RelayHealthStatus::Ok,
        "warn" => RelayHealthStatus::Warn,
        "dead" => RelayHealthStatus::Dead,
        _ => panic!("unknown health status: {value}"),
    }
}

fn parse_protocol(value: &str) -> RouteProtocol {
    match value {
        "wireguard" => RouteProtocol::WireGuard,
        "tcp_tls" => RouteProtocol::TcpTls,
        "quic" => RouteProtocol::Quic,
        _ => panic!("unknown protocol: {value}"),
    }
}

fn parse_outcome(value: &str) -> AttemptStepOutcome {
    match value {
        "success" => AttemptStepOutcome::Success,
        "timeout" => AttemptStepOutcome::Failed(AttemptFailureReason::Timeout),
        "handshake_failed" => AttemptStepOutcome::Failed(AttemptFailureReason::HandshakeFailed),
        "auth_rejected" => AttemptStepOutcome::Failed(AttemptFailureReason::AuthRejected),
        "network_unreachable" => {
            AttemptStepOutcome::Failed(AttemptFailureReason::NetworkUnreachable)
        }
        "unknown" => AttemptStepOutcome::Failed(AttemptFailureReason::Unknown),
        _ => panic!("unknown outcome: {value}"),
    }
}

fn health_as_str(status: RelayHealthStatus) -> &'static str {
    match status {
        RelayHealthStatus::Ok => "ok",
        RelayHealthStatus::Warn => "warn",
        RelayHealthStatus::Dead => "dead",
    }
}

fn protocol_as_str(protocol: RouteProtocol) -> &'static str {
    match protocol {
        RouteProtocol::WireGuard => "wireguard",
        RouteProtocol::TcpTls => "tcp_tls",
        RouteProtocol::Quic => "quic",
    }
}

fn scenario_name(scenario: &ChaosScenario) -> &str {
    match scenario {
        ChaosScenario::Trigger(definition) => &definition.name,
        ChaosScenario::Executor(definition) => &definition.name,
    }
}

#[test]
fn harness_covers_degraded_spikes_hard_failures_and_recovery_sequences() {
    let results = run_failover_chaos_suite();
    let degraded = results
        .iter()
        .find(|result| result.name == "degraded_spike_recovery")
        .expect("degraded scenario should be present");
    let hard = results
        .iter()
        .find(|result| result.name == "hard_failure_recovery")
        .expect("hard failure scenario should be present");

    assert!(degraded.terminal.contains("failovers=1"));
    assert!(degraded.terminal.contains("recovered_after_failover=true"));
    assert!(hard.terminal.contains("failovers=1"));
    assert!(hard
        .trace
        .iter()
        .any(|step| step.contains("dead:0:dead_relay_detected:true")));
}

#[test]
fn anti_flapping_logic_is_validated_under_rapid_oscillation() {
    let results = run_failover_chaos_suite();
    let oscillation = results
        .iter()
        .find(|result| result.name == "rapid_oscillation")
        .expect("rapid oscillation scenario should be present");

    assert!(oscillation.terminal.contains("failovers=2"));
    assert!(oscillation.terminal.contains("hysteresis_blocks=3"));
    assert!(oscillation
        .trace
        .iter()
        .any(|step| step.contains("hysteresis_window_active:false")));
}

#[test]
fn retry_budget_exhaustion_path_is_verified() {
    let results = run_failover_chaos_suite();
    let retry_exhaustion = results
        .iter()
        .find(|result| result.name == "retry_budget_exhaustion")
        .expect("retry exhaustion scenario should be present");

    assert!(retry_exhaustion
        .terminal
        .contains("failure_code=FAILOVER_RETRY_BUDGET_EXHAUSTED"));
    assert!(retry_exhaustion.terminal.contains("attempts=9"));
    assert!(retry_exhaustion.trace.contains(&"backoff:250".to_string()));
    assert!(retry_exhaustion.trace.contains(&"backoff:500".to_string()));
}

#[test]
fn output_provides_deterministic_trace_for_debugging() {
    let first = run_failover_chaos_suite();
    let second = run_failover_chaos_suite();
    assert_eq!(first, second);
}
