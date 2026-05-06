use client_engine::telemetry::events::updater::{
    validate_updater_payload, UPDATER_APPLY_EVENT_NAME, UPDATER_AVAILABLE_EVENT_NAME,
    UPDATER_CHECK_EVENT_NAME, UPDATER_DOWNLOAD_EVENT_NAME, UPDATER_FAILURE_EVENT_NAME,
};
use client_engine::telemetry::{TelemetryEvent, TelemetryPayload, TelemetryValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdaterFixtureSequence {
    name: String,
    steps: Vec<UpdaterFixtureStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdaterFixtureStep {
    at_ms: u64,
    session_id: String,
    event_name: String,
    state: String,
    channel: String,
    target_version: Option<String>,
    reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimedUpdaterEvent {
    at_ms: u64,
    session_id: String,
    event: TelemetryEvent,
}

#[derive(Debug, Clone, PartialEq)]
struct UpdaterKpiReport {
    scenario: String,
    total_sessions: usize,
    sessions_with_update_available: usize,
    sessions_reached_ready_to_restart: usize,
    sessions_with_failure: usize,
    adoption_rate: f64,
    success_rate: f64,
    failure_reason_breakdown: BTreeMap<String, usize>,
}

impl UpdaterKpiReport {
    fn release_readiness_summary(&self) -> String {
        let release_ready = self.adoption_rate >= 0.60
            && self.success_rate >= 0.60
            && self
                .failure_reason_breakdown
                .keys()
                .all(|reason| reason.starts_with("updater_"));
        format!(
            "scenario={}; release_ready={}; total_sessions={}; update_available_sessions={}; ready_to_restart_sessions={}; sessions_with_failure={}; adoption_rate={:.3}; success_rate={:.3}; failure_breakdown={}",
            self.scenario,
            release_ready,
            self.total_sessions,
            self.sessions_with_update_available,
            self.sessions_reached_ready_to_restart,
            self.sessions_with_failure,
            self.adoption_rate,
            self.success_rate,
            render_breakdown(&self.failure_reason_breakdown),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UpdaterKpiValidationError {
    NonMonotonicTimestamp {
        previous_ms: u64,
        current_ms: u64,
    },
    MissingAvailableBeforeDownload {
        session_id: String,
        at_ms: u64,
    },
    MissingDownloadBeforeApply {
        session_id: String,
        at_ms: u64,
    },
    MissingFailureReasonCode {
        session_id: String,
        at_ms: u64,
    },
    InvalidUpdaterPayload {
        session_id: String,
        at_ms: u64,
        event_name: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum ScenarioOutcome {
    Passed(UpdaterKpiReport),
    Failed {
        scenario: String,
        error: UpdaterKpiValidationError,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SessionState {
    saw_available: bool,
    saw_download: bool,
    reached_ready_to_restart: bool,
    saw_failure: bool,
}

fn run_updater_kpi_validation_suite() -> Vec<ScenarioOutcome> {
    let mut fixtures = load_updater_sequences();
    fixtures.sort_by(|left, right| left.name.cmp(&right.name));

    fixtures
        .into_iter()
        .map(|fixture| {
            let scenario = fixture.name.clone();
            match run_updater_kpi_validation(&fixture) {
                Ok(report) => ScenarioOutcome::Passed(report),
                Err(error) => ScenarioOutcome::Failed { scenario, error },
            }
        })
        .collect()
}

fn run_updater_kpi_validation(
    fixture: &UpdaterFixtureSequence,
) -> Result<UpdaterKpiReport, UpdaterKpiValidationError> {
    let stream = simulate_updater_stream(fixture)?;
    derive_updater_kpis(fixture, &stream)
}

fn simulate_updater_stream(
    fixture: &UpdaterFixtureSequence,
) -> Result<Vec<TimedUpdaterEvent>, UpdaterKpiValidationError> {
    let mut previous_at_ms = 0_u64;
    let mut stream = Vec::new();

    for step in &fixture.steps {
        if step.at_ms < previous_at_ms {
            return Err(UpdaterKpiValidationError::NonMonotonicTimestamp {
                previous_ms: previous_at_ms,
                current_ms: step.at_ms,
            });
        }
        previous_at_ms = step.at_ms;

        let payload = build_payload(step);
        validate_updater_payload(&step.event_name, &payload).map_err(|error| {
            UpdaterKpiValidationError::InvalidUpdaterPayload {
                session_id: step.session_id.clone(),
                at_ms: step.at_ms,
                event_name: step.event_name.clone(),
                message: error.message,
            }
        })?;

        stream.push(TimedUpdaterEvent {
            at_ms: step.at_ms,
            session_id: step.session_id.clone(),
            event: TelemetryEvent::new(step.event_name.clone(), payload),
        });
    }

    Ok(stream)
}

fn build_payload(step: &UpdaterFixtureStep) -> TelemetryPayload {
    let mut payload = TelemetryPayload::from([
        (
            "state".to_string(),
            TelemetryValue::Text(step.state.clone()),
        ),
        (
            "channel".to_string(),
            TelemetryValue::Text(step.channel.clone()),
        ),
    ]);

    match step.event_name.as_str() {
        UPDATER_AVAILABLE_EVENT_NAME | UPDATER_DOWNLOAD_EVENT_NAME | UPDATER_APPLY_EVENT_NAME => {
            if let Some(target_version) = &step.target_version {
                payload.insert(
                    "target_version".to_string(),
                    TelemetryValue::Text(target_version.clone()),
                );
            }
        }
        UPDATER_FAILURE_EVENT_NAME => {
            if let Some(reason_code) = &step.reason_code {
                payload.insert(
                    "reason_code".to_string(),
                    TelemetryValue::Text(reason_code.clone()),
                );
            }
        }
        UPDATER_CHECK_EVENT_NAME => {}
        other => panic!("unsupported updater event in fixture: {other}"),
    }

    payload
}

fn derive_updater_kpis(
    fixture: &UpdaterFixtureSequence,
    stream: &[TimedUpdaterEvent],
) -> Result<UpdaterKpiReport, UpdaterKpiValidationError> {
    let mut session_states = BTreeMap::<String, SessionState>::new();
    let mut failure_reason_breakdown = BTreeMap::<String, usize>::new();
    let mut all_sessions = BTreeSet::<String>::new();

    for entry in stream {
        all_sessions.insert(entry.session_id.clone());
        let state = session_states.entry(entry.session_id.clone()).or_default();

        match entry.event.name.as_str() {
            UPDATER_CHECK_EVENT_NAME => {}
            UPDATER_AVAILABLE_EVENT_NAME => {
                state.saw_available = true;
            }
            UPDATER_DOWNLOAD_EVENT_NAME => {
                if !state.saw_available {
                    return Err(UpdaterKpiValidationError::MissingAvailableBeforeDownload {
                        session_id: entry.session_id.clone(),
                        at_ms: entry.at_ms,
                    });
                }
                state.saw_download = true;
            }
            UPDATER_APPLY_EVENT_NAME => {
                if !state.saw_download {
                    return Err(UpdaterKpiValidationError::MissingDownloadBeforeApply {
                        session_id: entry.session_id.clone(),
                        at_ms: entry.at_ms,
                    });
                }
                state.reached_ready_to_restart = true;
            }
            UPDATER_FAILURE_EVENT_NAME => {
                let reason = match entry.event.payload.get("reason_code") {
                    Some(TelemetryValue::Text(reason)) if !reason.trim().is_empty() => {
                        reason.clone()
                    }
                    _ => {
                        return Err(UpdaterKpiValidationError::MissingFailureReasonCode {
                            session_id: entry.session_id.clone(),
                            at_ms: entry.at_ms,
                        })
                    }
                };
                state.saw_failure = true;
                let next = failure_reason_breakdown
                    .get(&reason)
                    .copied()
                    .unwrap_or(0usize)
                    .saturating_add(1);
                failure_reason_breakdown.insert(reason, next);
            }
            other => panic!("unsupported telemetry event encountered: {other}"),
        }
    }

    let total_sessions = all_sessions.len();
    let sessions_with_update_available =
        session_states.values().filter(|s| s.saw_available).count();
    let sessions_reached_ready_to_restart = session_states
        .values()
        .filter(|s| s.reached_ready_to_restart)
        .count();
    let sessions_with_failure = session_states.values().filter(|s| s.saw_failure).count();

    let adoption_rate = ratio(
        sessions_reached_ready_to_restart,
        sessions_with_update_available,
    );
    let success_rate = ratio(sessions_reached_ready_to_restart, total_sessions);

    Ok(UpdaterKpiReport {
        scenario: fixture.name.clone(),
        total_sessions,
        sessions_with_update_available,
        sessions_reached_ready_to_restart,
        sessions_with_failure,
        adoption_rate,
        success_rate,
        failure_reason_breakdown,
    })
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn render_breakdown(breakdown: &BTreeMap<String, usize>) -> String {
    if breakdown.is_empty() {
        return "none".to_string();
    }

    breakdown
        .iter()
        .map(|(reason, count)| format!("{reason}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn load_updater_sequences() -> Vec<UpdaterFixtureSequence> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("updater_event_sequences");
    let entries =
        fs::read_dir(base).expect("updater_event_sequences fixture directory should exist");

    let mut fixtures = Vec::new();
    for entry in entries {
        let path = entry.expect("fixture entry should be readable").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("seq") {
            continue;
        }
        let raw = fs::read_to_string(path).expect("updater fixture should be readable");
        fixtures.push(parse_updater_fixture(&raw));
    }
    fixtures
}

fn parse_updater_fixture(raw: &str) -> UpdaterFixtureSequence {
    let mut name = String::new();
    let mut steps_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("fixture key should exist").trim();
        let value = parts.next().expect("fixture value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "steps" => steps_raw = value.to_string(),
            _ => panic!("unknown updater fixture key: {key}"),
        }
    }

    UpdaterFixtureSequence {
        name,
        steps: parse_updater_steps(&steps_raw),
    }
}

fn parse_updater_steps(raw: &str) -> Vec<UpdaterFixtureStep> {
    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 7 {
                panic!("invalid updater step token: {token}");
            }

            UpdaterFixtureStep {
                at_ms: fields[0].parse().expect("at_ms should parse"),
                session_id: fields[1].to_string(),
                event_name: fields[2].to_string(),
                state: fields[3].to_string(),
                channel: fields[4].to_string(),
                target_version: normalize_optional_field(fields[5]),
                reason_code: normalize_optional_field(fields[6]),
            }
        })
        .collect()
}

fn normalize_optional_field(raw: &str) -> Option<String> {
    if raw == "_" {
        None
    } else {
        Some(raw.to_string())
    }
}

fn find_scenario_report<'a>(
    outcomes: &'a [ScenarioOutcome],
    scenario: &str,
) -> &'a UpdaterKpiReport {
    match outcomes.iter().find(
        |outcome| matches!(outcome, ScenarioOutcome::Passed(report) if report.scenario == scenario),
    ) {
        Some(ScenarioOutcome::Passed(report)) => report,
        _ => panic!("expected passed scenario '{scenario}'"),
    }
}

#[test]
fn kpi_derivation_for_update_adoption_rate_is_reproducible() {
    let first = run_updater_kpi_validation_suite();
    let second = run_updater_kpi_validation_suite();
    assert_eq!(first, second);

    let report = find_scenario_report(&first, "nominal_updater_adoption");
    assert_eq!(report.total_sessions, 3);
    assert_eq!(report.sessions_with_update_available, 3);
    assert_eq!(report.sessions_reached_ready_to_restart, 2);
    assert!((report.adoption_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
}

#[test]
fn failure_category_breakdown_is_derivable_from_reason_codes() {
    let outcomes = run_updater_kpi_validation_suite();
    let report = find_scenario_report(&outcomes, "failure_breakdown_updater");

    assert_eq!(report.sessions_with_failure, 2);
    assert_eq!(
        report
            .failure_reason_breakdown
            .get("updater_package_signature_mismatch"),
        Some(&1)
    );
    assert_eq!(
        report.failure_reason_breakdown.get("updater_apply_failed"),
        Some(&1)
    );
}

#[test]
fn missing_sequence_segments_fail_assertions() {
    let outcomes = run_updater_kpi_validation_suite();
    let failed = outcomes
        .iter()
        .find(|outcome| {
            matches!(
                outcome,
                ScenarioOutcome::Failed {
                    scenario,
                    ..
                } if scenario == "missing_available_segment"
            )
        })
        .expect("missing sequence scenario should fail");

    match failed {
        ScenarioOutcome::Failed { error, .. } => {
            assert_eq!(
                *error,
                UpdaterKpiValidationError::MissingAvailableBeforeDownload {
                    session_id: "sess-missing".to_string(),
                    at_ms: 1000,
                }
            );
        }
        ScenarioOutcome::Passed(_) => panic!("scenario should fail"),
    }
}

#[test]
fn output_supports_release_readiness_decisioning() {
    let outcomes = run_updater_kpi_validation_suite();
    let report = find_scenario_report(&outcomes, "nominal_updater_adoption");
    let summary = report.release_readiness_summary();

    assert!(summary.contains("release_ready=true"));
    assert!(summary.contains("adoption_rate=0.667"));
    assert!(summary.contains("success_rate=0.667"));
    assert!(summary.contains("failure_breakdown=updater_download_transport_failed:1"));
}
