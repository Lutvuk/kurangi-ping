use client_engine::db::{MetricSampleRecord, SqliteMetricsRepo};
use client_engine::metrics::PingMetrics;
use client_engine::routing::{build_failover_state_payload, FailoverUiState};
use client_engine::telemetry::events::ping_metrics::{
    emit_ping_measured_event, PingMeasuredEmissionPolicy, PingMeasuredEmitState,
};
use client_engine::telemetry::events::relay_failover::{
    emit_relay_failed, emit_relay_recovered, RelayFailoverEmissionPolicy, RELAY_FAILED_EVENT_NAME,
    RELAY_RECOVERED_EVENT_NAME,
};
use client_engine::telemetry::{TelemetryEvent, TelemetryService, TelemetryValue};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
struct MetricsFixtureSample {
    at_ms: u64,
    baseline_ping_ms: f64,
    routed_ping_ms: f64,
    jitter_ms: f64,
    packet_loss_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MetricsFixtureEvent {
    at_ms: u64,
    event_name: String,
    attempt_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct MetricsFixtureSequence {
    name: String,
    sample_interval_ms: u64,
    samples: Vec<MetricsFixtureSample>,
    events: Vec<MetricsFixtureEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimedTelemetryEvent {
    at_ms: u64,
    event: TelemetryEvent,
}

#[derive(Debug, Clone, PartialEq)]
struct MetricsKpiReport {
    scenario: String,
    avg_reduction_ms: f64,
    degraded_ratio: f64,
    measurement_continuity_ratio: f64,
    persisted_sample_count: usize,
    ping_event_count: usize,
    failover_pairs: usize,
}

impl MetricsKpiReport {
    fn analytics_summary(&self) -> String {
        let release_ready = self.avg_reduction_ms >= 40.0
            && self.measurement_continuity_ratio >= 0.95
            && self.degraded_ratio <= 0.40;
        format!(
            "scenario={}; release_ready={}; avg_reduction_ms={:.3}; degraded_ratio={:.3}; measurement_continuity={:.3}; persisted_sample_count={}; ping_event_count={}; failover_pairs={}",
            self.scenario,
            release_ready,
            self.avg_reduction_ms,
            self.degraded_ratio,
            self.measurement_continuity_ratio,
            self.persisted_sample_count,
            self.ping_event_count,
            self.failover_pairs
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MetricsKpiValidationError {
    NonMonotonicSampleTimestamp {
        previous_ms: u64,
        current_ms: u64,
    },
    NonMonotonicEventTimestamp {
        previous_ms: u64,
        current_ms: u64,
    },
    UnsupportedFixtureEvent {
        event_name: String,
    },
    MissingAttemptCount {
        at_ms: u64,
        event_name: String,
    },
    RecoveryWithoutFailure {
        at_ms: u64,
    },
    MissingRecoveryAfterFailure {
        at_ms: u64,
    },
    MissingPingMeasuredForPersistedSamples {
        persisted_samples: usize,
        ping_events: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum ScenarioOutcome {
    Passed(MetricsKpiReport),
    Failed {
        scenario: String,
        error: MetricsKpiValidationError,
    },
}

fn run_metrics_kpi_validation_suite() -> Vec<ScenarioOutcome> {
    let mut scenarios = load_metrics_sequences();
    scenarios.sort_by(|left, right| left.name.cmp(&right.name));
    scenarios
        .into_iter()
        .map(|scenario| {
            let scenario_name = scenario.name.clone();
            match run_metrics_kpi_validation(&scenario) {
                Ok(report) => ScenarioOutcome::Passed(report),
                Err(error) => ScenarioOutcome::Failed {
                    scenario: scenario_name,
                    error,
                },
            }
        })
        .collect()
}

fn run_metrics_kpi_validation(
    sequence: &MetricsFixtureSequence,
) -> Result<MetricsKpiReport, MetricsKpiValidationError> {
    validate_sample_timestamps(&sequence.samples)?;
    validate_event_timestamps(&sequence.events)?;

    let persisted_samples = persist_samples(&sequence.samples);
    let telemetry_stream = simulate_telemetry_stream(sequence);
    derive_kpi_report(sequence, &persisted_samples, &telemetry_stream)
}

fn persist_samples(samples: &[MetricsFixtureSample]) -> Vec<MetricSampleRecord> {
    let conn = setup_metrics_repo_db();
    let repo = SqliteMetricsRepo::new(&conn);

    for (index, sample) in samples.iter().enumerate() {
        let record = MetricSampleRecord {
            sample_id: format!("sample-{index}"),
            session_id: "sess-kpi".to_string(),
            baseline_ping_ms: sample.baseline_ping_ms,
            routed_ping_ms: Some(sample.routed_ping_ms),
            jitter_ms: Some(sample.jitter_ms),
            packet_loss_pct: Some(sample.packet_loss_pct),
            sampled_at: sampled_at_iso(sample.at_ms),
        };
        repo.append_metric_sample(&record)
            .expect("sample should persist for KPI validation");
    }

    repo.recent_metric_samples("sess-kpi", samples.len().saturating_add(8))
        .expect("persisted samples should be queryable")
}

fn simulate_telemetry_stream(sequence: &MetricsFixtureSequence) -> Vec<TimedTelemetryEvent> {
    let mut telemetry = TelemetryService::new();
    let mut ping_emit_state = PingMeasuredEmitState::default();
    let mut stream = Vec::new();

    for sample in &sequence.samples {
        let metrics = PingMetrics {
            baseline_ping_ms: sample.baseline_ping_ms,
            routed_ping_ms: Some(sample.routed_ping_ms),
            jitter_ms: Some(sample.jitter_ms),
            packet_loss_pct: Some(sample.packet_loss_pct),
            sample_count: 1,
        };
        let emit_status = emit_ping_measured_event(
            &mut telemetry,
            &metrics,
            sample.at_ms,
            &mut ping_emit_state,
            PingMeasuredEmissionPolicy {
                max_queue_depth: 100_000,
                min_emit_interval_ms: 0,
            },
        )
        .expect("ping_measured should emit from fixture sample");
        assert_eq!(
            emit_status,
            client_engine::telemetry::events::ping_metrics::PingMeasuredEmitStatus::Emitted
        );

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("ping_measured should be queued");
        stream.push(TimedTelemetryEvent {
            at_ms: sample.at_ms,
            event,
        });
    }

    for fixture_event in &sequence.events {
        let payload = match fixture_event.event_name.as_str() {
            RELAY_FAILED_EVENT_NAME => build_failover_state_payload(
                FailoverUiState::Switching,
                Some("sin-01"),
                Some("nrt-01"),
                Some("dead_relay_detected"),
            ),
            RELAY_RECOVERED_EVENT_NAME => build_failover_state_payload(
                FailoverUiState::Recovered,
                Some("sin-01"),
                Some("nrt-01"),
                Some("switch_successful"),
            ),
            _ => panic!("unsupported fixture event: {}", fixture_event.event_name),
        };

        let status = if fixture_event.event_name == RELAY_FAILED_EVENT_NAME {
            emit_relay_failed(
                &mut telemetry,
                &payload,
                fixture_event.attempt_count,
                RelayFailoverEmissionPolicy {
                    max_queue_depth: 100_000,
                },
            )
            .expect("relay_failed should emit")
        } else {
            emit_relay_recovered(
                &mut telemetry,
                &payload,
                fixture_event.attempt_count,
                RelayFailoverEmissionPolicy {
                    max_queue_depth: 100_000,
                },
            )
            .expect("relay_recovered should emit")
        };
        assert_eq!(
            status,
            client_engine::telemetry::events::relay_failover::RelayFailoverEmitStatus::Emitted
        );

        let event = telemetry
            .drain_batch(1)
            .pop()
            .expect("relay event should be queued");
        stream.push(TimedTelemetryEvent {
            at_ms: fixture_event.at_ms,
            event,
        });
    }

    stream
}

fn derive_kpi_report(
    sequence: &MetricsFixtureSequence,
    persisted_samples: &[MetricSampleRecord],
    telemetry_stream: &[TimedTelemetryEvent],
) -> Result<MetricsKpiReport, MetricsKpiValidationError> {
    let avg_reduction_ms = average_reduction_from_samples(persisted_samples);
    let ping_events = telemetry_stream
        .iter()
        .filter(|entry| entry.event.name == "ping_measured")
        .collect::<Vec<_>>();
    let continuity_ratio = if persisted_samples.is_empty() {
        1.0
    } else {
        ping_events.len() as f64 / persisted_samples.len() as f64
    };

    if ping_events.len() < persisted_samples.len() {
        return Err(
            MetricsKpiValidationError::MissingPingMeasuredForPersistedSamples {
                persisted_samples: persisted_samples.len(),
                ping_events: ping_events.len(),
            },
        );
    }

    let relay_events = telemetry_stream
        .iter()
        .filter(|entry| {
            entry.event.name == RELAY_FAILED_EVENT_NAME
                || entry.event.name == RELAY_RECOVERED_EVENT_NAME
        })
        .cloned()
        .collect::<Vec<_>>();
    let (degraded_windows, failover_pairs) = derive_degraded_windows(&relay_events)?;

    for relay in &relay_events {
        match relay.event.payload.get("attempt_count") {
            Some(TelemetryValue::Integer(_)) => {}
            _ => {
                return Err(MetricsKpiValidationError::MissingAttemptCount {
                    at_ms: relay.at_ms,
                    event_name: relay.event.name.clone(),
                })
            }
        }
    }

    let degraded_pings = ping_events
        .iter()
        .filter(|entry| is_within_degraded_windows(entry.at_ms, &degraded_windows))
        .count();
    let degraded_ratio = if ping_events.is_empty() {
        0.0
    } else {
        degraded_pings as f64 / ping_events.len() as f64
    };

    Ok(MetricsKpiReport {
        scenario: sequence.name.clone(),
        avg_reduction_ms,
        degraded_ratio,
        measurement_continuity_ratio: continuity_ratio,
        persisted_sample_count: persisted_samples.len(),
        ping_event_count: ping_events.len(),
        failover_pairs,
    })
}

fn derive_degraded_windows(
    relay_events: &[TimedTelemetryEvent],
) -> Result<(Vec<(u64, u64)>, usize), MetricsKpiValidationError> {
    let mut sorted = relay_events.to_vec();
    sorted.sort_by_key(|entry| entry.at_ms);

    let mut windows = Vec::new();
    let mut open_failure_at = None;
    let mut recovered_pairs = 0usize;

    for relay in sorted {
        match relay.event.name.as_str() {
            RELAY_FAILED_EVENT_NAME => {
                if open_failure_at.is_none() {
                    open_failure_at = Some(relay.at_ms);
                }
            }
            RELAY_RECOVERED_EVENT_NAME => {
                let started_at =
                    open_failure_at.ok_or(MetricsKpiValidationError::RecoveryWithoutFailure {
                        at_ms: relay.at_ms,
                    })?;
                windows.push((started_at, relay.at_ms));
                open_failure_at = None;
                recovered_pairs = recovered_pairs.saturating_add(1);
            }
            other => {
                return Err(MetricsKpiValidationError::UnsupportedFixtureEvent {
                    event_name: other.to_string(),
                })
            }
        }
    }

    if let Some(started_at) = open_failure_at {
        return Err(MetricsKpiValidationError::MissingRecoveryAfterFailure { at_ms: started_at });
    }

    Ok((windows, recovered_pairs))
}

fn is_within_degraded_windows(at_ms: u64, windows: &[(u64, u64)]) -> bool {
    windows
        .iter()
        .any(|(start_ms, end_ms)| at_ms >= *start_ms && at_ms < *end_ms)
}

fn average_reduction_from_samples(samples: &[MetricSampleRecord]) -> f64 {
    let mut reductions = Vec::new();
    for sample in samples {
        if let Some(routed) = sample.routed_ping_ms {
            reductions.push(sample.baseline_ping_ms - routed);
        }
    }
    if reductions.is_empty() {
        return 0.0;
    }
    reductions.iter().sum::<f64>() / reductions.len() as f64
}

fn sampled_at_iso(at_ms: u64) -> String {
    let total_seconds = at_ms / 1_000;
    let hours = (total_seconds / 3_600) % 24;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;
    format!("2026-08-01T{:02}:{:02}:{:02}Z", hours, minutes, seconds)
}

fn validate_sample_timestamps(
    samples: &[MetricsFixtureSample],
) -> Result<(), MetricsKpiValidationError> {
    for pair in samples.windows(2) {
        if pair[1].at_ms < pair[0].at_ms {
            return Err(MetricsKpiValidationError::NonMonotonicSampleTimestamp {
                previous_ms: pair[0].at_ms,
                current_ms: pair[1].at_ms,
            });
        }
    }
    Ok(())
}

fn validate_event_timestamps(
    events: &[MetricsFixtureEvent],
) -> Result<(), MetricsKpiValidationError> {
    for pair in events.windows(2) {
        if pair[1].at_ms < pair[0].at_ms {
            return Err(MetricsKpiValidationError::NonMonotonicEventTimestamp {
                previous_ms: pair[0].at_ms,
                current_ms: pair[1].at_ms,
            });
        }
    }
    Ok(())
}

fn setup_metrics_repo_db() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE ping_sample (
          sample_id TEXT PRIMARY KEY,
          session_id TEXT NOT NULL,
          baseline_ping_ms REAL NOT NULL CHECK (baseline_ping_ms >= 0),
          routed_ping_ms REAL CHECK (routed_ping_ms IS NULL OR routed_ping_ms >= 0),
          jitter_ms REAL CHECK (jitter_ms IS NULL OR jitter_ms >= 0),
          packet_loss_pct REAL CHECK (packet_loss_pct IS NULL OR (packet_loss_pct >= 0 AND packet_loss_pct <= 100)),
          sampled_at TEXT NOT NULL CHECK (datetime(sampled_at) IS NOT NULL AND sampled_at GLOB '????-??-??T??:??:??*Z')
        );
        "#,
    )
    .expect("fixture ping_sample table should be created");
    conn
}

fn load_metrics_sequences() -> Vec<MetricsFixtureSequence> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("metrics_event_sequences");
    let entries =
        fs::read_dir(base).expect("metrics_event_sequences fixture directory should be readable");

    let mut sequences = Vec::new();
    for entry in entries {
        let path = entry.expect("fixture entry should exist").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("seq") {
            continue;
        }
        let raw = fs::read_to_string(path).expect("kpi sequence fixture should be readable");
        sequences.push(parse_metrics_sequence_fixture(&raw));
    }
    sequences
}

fn parse_metrics_sequence_fixture(raw: &str) -> MetricsFixtureSequence {
    let mut name = String::new();
    let mut sample_interval_ms = 1_000_u64;
    let mut samples_raw = String::new();
    let mut events_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("fixture key should exist").trim();
        let value = parts.next().expect("fixture value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "sample_interval_ms" => {
                sample_interval_ms = value.parse().expect("sample_interval_ms should parse")
            }
            "samples" => samples_raw = value.to_string(),
            "events" => events_raw = value.to_string(),
            _ => panic!("unknown KPI fixture key: {key}"),
        }
    }

    MetricsFixtureSequence {
        name,
        sample_interval_ms,
        samples: parse_metrics_samples(&samples_raw),
        events: parse_metrics_events(&events_raw),
    }
}

fn parse_metrics_samples(raw: &str) -> Vec<MetricsFixtureSample> {
    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 5 {
                panic!("invalid sample token: {token}");
            }
            MetricsFixtureSample {
                at_ms: fields[0].parse().expect("sample at_ms should parse"),
                baseline_ping_ms: fields[1].parse().expect("baseline should parse"),
                routed_ping_ms: fields[2].parse().expect("routed should parse"),
                jitter_ms: fields[3].parse().expect("jitter should parse"),
                packet_loss_pct: fields[4].parse().expect("packet loss should parse"),
            }
        })
        .collect()
}

fn parse_metrics_events(raw: &str) -> Vec<MetricsFixtureEvent> {
    if raw == "_" {
        return Vec::new();
    }

    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 3 {
                panic!("invalid event token: {token}");
            }
            MetricsFixtureEvent {
                at_ms: fields[0].parse().expect("event at_ms should parse"),
                event_name: fields[1].to_string(),
                attempt_count: fields[2].parse().expect("attempt_count should parse"),
            }
        })
        .collect()
}

fn load_sequence(path: &str) -> MetricsFixtureSequence {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("metrics_event_sequences")
        .join(path);
    let raw = fs::read_to_string(base).expect("metrics_event fixture should be readable");
    parse_metrics_sequence_fixture(&raw)
}

fn approx_eq(left: f64, right: f64) {
    let delta = (left - right).abs();
    assert!(
        delta < 0.000_001,
        "expected {left} ~= {right}, delta={delta}"
    );
}

#[test]
fn derived_metrics_include_avg_reduction_degraded_ratio_and_continuity() {
    let sequence = load_sequence("nominal_metrics_kpi.seq");
    let report = run_metrics_kpi_validation(&sequence).expect("nominal fixture should aggregate");

    assert_eq!(report.scenario, "nominal_metrics_kpi");
    approx_eq(report.avg_reduction_ms, 52.666_666_666_7);
    approx_eq(report.degraded_ratio, 0.333_333_333_3);
    approx_eq(report.measurement_continuity_ratio, 1.0);
    assert_eq!(report.persisted_sample_count, 6);
    assert_eq!(report.ping_event_count, 6);
    assert_eq!(report.failover_pairs, 1);
}

#[test]
fn missing_or_inconsistent_event_sequences_fail_validation() {
    let missing = load_sequence("missing_recovery_metrics_kpi.seq");
    let missing_err =
        run_metrics_kpi_validation(&missing).expect_err("missing recovery should fail");
    assert_eq!(
        missing_err,
        MetricsKpiValidationError::MissingRecoveryAfterFailure { at_ms: 2_500 }
    );

    let inconsistent = load_sequence("recovered_without_failure_metrics_kpi.seq");
    let inconsistent_err =
        run_metrics_kpi_validation(&inconsistent).expect_err("recovered first should fail");
    assert_eq!(
        inconsistent_err,
        MetricsKpiValidationError::RecoveryWithoutFailure { at_ms: 1_500 }
    );
}

#[test]
fn kpi_extraction_logic_is_deterministic_across_runs() {
    let first = run_metrics_kpi_validation_suite();
    let second = run_metrics_kpi_validation_suite();
    assert_eq!(first, second);
}

#[test]
fn output_is_actionable_for_product_analytics_review() {
    let sequence = load_sequence("nominal_metrics_kpi.seq");
    let report = run_metrics_kpi_validation(&sequence).expect("nominal fixture should aggregate");
    let summary = report.analytics_summary();

    assert!(summary.contains("release_ready=true"));
    assert!(summary.contains("avg_reduction_ms=52.667"));
    assert!(summary.contains("degraded_ratio=0.333"));
    assert!(summary.contains("measurement_continuity=1.000"));
    assert!(summary.contains("failover_pairs=1"));
}
