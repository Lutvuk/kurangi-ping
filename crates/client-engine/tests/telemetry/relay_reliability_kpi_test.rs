use client_engine::routing::{build_failover_state_payload, FailoverUiState};
use client_engine::telemetry::events::relay_failover::{
    emit_relay_failed, emit_relay_recovered, RelayFailoverEmissionPolicy, RelayFailoverEmitStatus,
};
use client_engine::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureSequence {
    name: String,
    steps: Vec<FixtureStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureStep {
    at_ms: u64,
    event_name: String,
    state: String,
    previous_relay_id: Option<String>,
    next_relay_id: Option<String>,
    reason_code: String,
    attempt_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimedRelayEvent {
    at_ms: u64,
    event: TelemetryEvent,
}

#[derive(Debug, Clone, PartialEq)]
struct ReliabilityKpiReport {
    failure_count: usize,
    recovered_count: usize,
    failover_frequency_per_min: f64,
    average_recovery_ms: Option<u64>,
    max_recovery_ms: Option<u64>,
    stability_ratio: f64,
    max_attempt_count: i64,
}

impl ReliabilityKpiReport {
    fn release_readiness_summary(&self) -> String {
        let release_ready = self.stability_ratio >= 0.95
            && self.average_recovery_ms.unwrap_or(0) <= 3_000
            && self.failure_count == self.recovered_count;
        format!(
            "release_ready={}; failure_count={}; recovered_count={}; failover_frequency_per_min={:.3}; average_recovery_ms={}; max_recovery_ms={}; stability_ratio={:.3}; max_attempt_count={}",
            release_ready,
            self.failure_count,
            self.recovered_count,
            self.failover_frequency_per_min,
            self.average_recovery_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.max_recovery_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.stability_ratio,
            self.max_attempt_count,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum KpiAggregationError {
    NonMonotonicTimestamp { previous_ms: u64, current_ms: u64 },
    RecoveryWithoutFailure { at_ms: u64 },
    MissingRecoveryAfterFailure { at_ms: u64 },
    MissingAttemptCount { at_ms: u64, event_name: String },
}

fn run_relay_reliability_kpi(sequence: &FixtureSequence) -> Result<ReliabilityKpiReport, KpiAggregationError> {
    let emitted = simulate_sequence_emission(sequence);
    derive_kpis(&emitted)
}

fn simulate_sequence_emission(sequence: &FixtureSequence) -> Vec<TimedRelayEvent> {
    let mut telemetry = TelemetryService::new();
    let mut stream = Vec::new();
    let policy = RelayFailoverEmissionPolicy { max_queue_depth: 10_000 };

    for step in &sequence.steps {
        let state = parse_failover_state(&step.state);
        let payload = build_failover_state_payload(
            state,
            step.previous_relay_id.as_deref(),
            step.next_relay_id.as_deref(),
            Some(&step.reason_code),
        );

        let status = match step.event_name.as_str() {
            "relay_failed" => emit_relay_failed(&mut telemetry, &payload, step.attempt_count, policy)
                .expect("relay_failed emission should not fail"),
            "relay_recovered" => {
                emit_relay_recovered(&mut telemetry, &payload, step.attempt_count, policy)
                    .expect("relay_recovered emission should not fail")
            }
            other => panic!("unknown event in fixture: {other}"),
        };

        assert_eq!(
            status,
            RelayFailoverEmitStatus::Emitted,
            "fixture step should emit telemetry event: {:?}",
            step
        );

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("emitted event should exist in telemetry queue");
        stream.push(TimedRelayEvent {
            at_ms: step.at_ms,
            event,
        });
    }

    stream
}

fn derive_kpis(events: &[TimedRelayEvent]) -> Result<ReliabilityKpiReport, KpiAggregationError> {
    if events.is_empty() {
        return Ok(ReliabilityKpiReport {
            failure_count: 0,
            recovered_count: 0,
            failover_frequency_per_min: 0.0,
            average_recovery_ms: None,
            max_recovery_ms: None,
            stability_ratio: 1.0,
            max_attempt_count: 0,
        });
    }

    let mut previous_ts = events[0].at_ms;
    let mut last_failure_at: Option<u64> = None;
    let mut recovery_durations = Vec::new();
    let mut failure_count = 0_usize;
    let mut recovered_count = 0_usize;
    let mut max_attempt_count = 0_i64;

    for entry in events {
        if entry.at_ms < previous_ts {
            return Err(KpiAggregationError::NonMonotonicTimestamp {
                previous_ms: previous_ts,
                current_ms: entry.at_ms,
            });
        }
        previous_ts = entry.at_ms;

        let attempt_count = match entry.event.payload.get("attempt_count") {
            Some(TelemetryValue::Integer(value)) => *value,
            _ => {
                return Err(KpiAggregationError::MissingAttemptCount {
                    at_ms: entry.at_ms,
                    event_name: entry.event.name.clone(),
                })
            }
        };
        if attempt_count > max_attempt_count {
            max_attempt_count = attempt_count;
        }

        match entry.event.name.as_str() {
            "relay_failed" => {
                failure_count = failure_count.saturating_add(1);
                last_failure_at = Some(entry.at_ms);
            }
            "relay_recovered" => {
                recovered_count = recovered_count.saturating_add(1);
                let failed_at =
                    last_failure_at.ok_or(KpiAggregationError::RecoveryWithoutFailure {
                        at_ms: entry.at_ms,
                    })?;
                recovery_durations.push(entry.at_ms.saturating_sub(failed_at));
                last_failure_at = None;
            }
            _ => {}
        }
    }

    if let Some(missing_at) = last_failure_at {
        return Err(KpiAggregationError::MissingRecoveryAfterFailure { at_ms: missing_at });
    }

    let window_ms = events
        .last()
        .expect("non-empty events must have last")
        .at_ms
        .saturating_sub(events[0].at_ms)
        .max(1);
    let failures_per_min = (failure_count as f64) * 60_000_f64 / (window_ms as f64);

    let average_recovery_ms = if recovery_durations.is_empty() {
        None
    } else {
        Some(
            recovery_durations.iter().sum::<u64>()
                / u64::try_from(recovery_durations.len()).expect("length should fit"),
        )
    };
    let max_recovery_ms = recovery_durations.iter().copied().max();
    let stability_ratio = if failure_count == 0 {
        1.0
    } else {
        recovered_count as f64 / failure_count as f64
    };

    Ok(ReliabilityKpiReport {
        failure_count,
        recovered_count,
        failover_frequency_per_min: failures_per_min,
        average_recovery_ms,
        max_recovery_ms,
        stability_ratio,
        max_attempt_count,
    })
}

fn parse_failover_state(value: &str) -> FailoverUiState {
    match value {
        "stable" => FailoverUiState::Stable,
        "switching" => FailoverUiState::Switching,
        "recovered" => FailoverUiState::Recovered,
        "failed" => FailoverUiState::Failed,
        _ => panic!("unknown failover state in fixture: {value}"),
    }
}

fn load_sequence(path: &str) -> FixtureSequence {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("relay_event_sequences")
        .join(path);
    let raw = fs::read_to_string(base).expect("relay event fixture should be readable");

    let mut name = String::new();
    let mut steps_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("key should exist").trim();
        let value = parts.next().expect("value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "steps" => steps_raw = value.to_string(),
            _ => panic!("unknown fixture key: {key}"),
        }
    }

    let steps = steps_raw
        .split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 7 {
                panic!("invalid step fixture token: {token}");
            }
            FixtureStep {
                at_ms: fields[0].parse().expect("at_ms should parse"),
                event_name: fields[1].to_string(),
                state: fields[2].to_string(),
                previous_relay_id: Some(fields[3].to_string()),
                next_relay_id: Some(fields[4].to_string()),
                reason_code: fields[5].to_string(),
                attempt_count: fields[6].parse().expect("attempt_count should parse"),
            }
        })
        .collect();

    FixtureSequence { name, steps }
}

#[test]
fn event_sequences_can_derive_failover_frequency_metrics() {
    let sequence = load_sequence("nominal_recovery.seq");
    let report = run_relay_reliability_kpi(&sequence).expect("nominal stream should aggregate");

    assert_eq!(sequence.name, "nominal_recovery");
    assert_eq!(report.failure_count, 2);
    assert!(report.failover_frequency_per_min > 20.0);
    assert!(report.failover_frequency_per_min < 22.0);
}

#[test]
fn event_sequences_can_derive_recovery_duration_metrics() {
    let sequence = load_sequence("nominal_recovery.seq");
    let report = run_relay_reliability_kpi(&sequence).expect("nominal stream should aggregate");

    assert_eq!(report.average_recovery_ms, Some(1450));
    assert_eq!(report.max_recovery_ms, Some(1700));
    assert_eq!(report.stability_ratio, 1.0);
}

#[test]
fn missing_or_invalid_events_are_detected_by_assertions() {
    let missing = load_sequence("missing_recovery.seq");
    let err = run_relay_reliability_kpi(&missing).expect_err("missing recovery must fail");
    assert_eq!(
        err,
        KpiAggregationError::MissingRecoveryAfterFailure { at_ms: 5_000 }
    );

    let invalid = load_sequence("invalid_recovered_first.seq");
    let err =
        run_relay_reliability_kpi(&invalid).expect_err("recovered before failed must fail");
    assert_eq!(
        err,
        KpiAggregationError::RecoveryWithoutFailure { at_ms: 1_000 }
    );
}

#[test]
fn test_output_is_actionable_for_release_readiness_review() {
    let sequence = load_sequence("nominal_recovery.seq");
    let report = run_relay_reliability_kpi(&sequence).expect("nominal stream should aggregate");
    let summary = report.release_readiness_summary();

    assert!(summary.contains("release_ready=true"));
    assert!(summary.contains("failure_count=2"));
    assert!(summary.contains("average_recovery_ms=1450"));
    assert!(summary.contains("stability_ratio=1.000"));
    assert!(summary.contains("max_attempt_count=3"));
}
