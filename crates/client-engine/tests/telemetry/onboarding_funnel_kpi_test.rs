use client_engine::telemetry::events::onboarding::validate_onboarding_completed_payload;
use client_engine::telemetry::{TelemetryPayload, TelemetryValue};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnboardingFunnelFixture {
    name: String,
    journeys: Vec<JourneyFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JourneyFixture {
    session_id: String,
    started_at_ms: u64,
    steps: Vec<StepFixture>,
    completion_event: Option<CompletionEventFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StepFixture {
    step: OnboardingStepCode,
    at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletionEventFixture {
    at_ms: u64,
    onboarding_duration_s: u64,
    game_id: String,
    success_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum OnboardingStepCode {
    Welcome,
    PermissionCheck,
    RelayTest,
    GameDetectionTest,
    FirstConnect,
}

impl OnboardingStepCode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Welcome => "welcome",
            Self::PermissionCheck => "permission_check",
            Self::RelayTest => "relay_test",
            Self::GameDetectionTest => "game_detection_test",
            Self::FirstConnect => "first_connect",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OnboardingFunnelKpiReport {
    scenario: String,
    total_journeys: usize,
    completed_journeys: usize,
    completion_rate: f64,
    average_time_to_connect_s: f64,
    step_drop_off: BTreeMap<String, usize>,
}

impl OnboardingFunnelKpiReport {
    fn tuning_summary(&self) -> String {
        let dominant_drop_off = self
            .step_drop_off
            .iter()
            .max_by_key(|entry| entry.1)
            .map(|(step, count)| format!("{step}:{count}"))
            .unwrap_or_else(|| "none:0".to_string());
        let recommendation = if dominant_drop_off.starts_with("permission_check:") {
            "improve_permission_guidance"
        } else if dominant_drop_off.starts_with("relay_test:") {
            "improve_relay_probe_stability"
        } else if dominant_drop_off.starts_with("game_detection_test:") {
            "improve_game_detection_copy"
        } else {
            "maintain_current_onboarding_flow"
        };

        format!(
            "scenario={}; completion_rate={:.3}; avg_time_to_connect_s={:.3}; dominant_drop_off={}; recommendation={}",
            self.scenario,
            self.completion_rate,
            self.average_time_to_connect_s,
            dominant_drop_off,
            recommendation
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OnboardingFunnelValidationError {
    NonMonotonicStepTimestamp {
        session_id: String,
        previous_ms: u64,
        current_ms: u64,
    },
    CompletionEventWithoutFirstConnect {
        session_id: String,
    },
    MissingCompletionEvent {
        session_id: String,
    },
    CompletionEventBeforeFirstConnect {
        session_id: String,
        event_at_ms: u64,
        first_connect_at_ms: u64,
    },
    CompletionDurationMismatch {
        session_id: String,
        expected_s: u64,
        actual_s: u64,
    },
    SuccessPathMismatch {
        session_id: String,
        expected: String,
        actual: String,
    },
    InvalidCompletionPayload {
        session_id: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum ScenarioOutcome {
    Passed(OnboardingFunnelKpiReport),
    Failed {
        scenario: String,
        error: OnboardingFunnelValidationError,
    },
}

fn run_onboarding_funnel_kpi_validation_suite() -> Vec<ScenarioOutcome> {
    let mut fixtures = load_onboarding_funnel_sequences();
    fixtures.sort_by(|left, right| left.name.cmp(&right.name));

    fixtures
        .into_iter()
        .map(|fixture| {
            let scenario = fixture.name.clone();
            match run_onboarding_funnel_kpi_validation(&fixture) {
                Ok(report) => ScenarioOutcome::Passed(report),
                Err(error) => ScenarioOutcome::Failed { scenario, error },
            }
        })
        .collect()
}

fn run_onboarding_funnel_kpi_validation(
    fixture: &OnboardingFunnelFixture,
) -> Result<OnboardingFunnelKpiReport, OnboardingFunnelValidationError> {
    let mut completed_journeys = 0usize;
    let mut connect_durations_s = Vec::new();
    let mut step_drop_off = BTreeMap::new();

    for journey in &fixture.journeys {
        validate_step_timestamps(journey)?;
        let first_connect_at = journey
            .steps
            .iter()
            .find(|step| step.step == OnboardingStepCode::FirstConnect)
            .map(|step| step.at_ms);

        match (first_connect_at, &journey.completion_event) {
            (Some(connect_at), Some(completed_event)) => {
                if completed_event.at_ms < connect_at {
                    return Err(OnboardingFunnelValidationError::CompletionEventBeforeFirstConnect {
                        session_id: journey.session_id.clone(),
                        event_at_ms: completed_event.at_ms,
                        first_connect_at_ms: connect_at,
                    });
                }

                let actual_duration_s = (completed_event.at_ms - journey.started_at_ms) / 1_000;
                if completed_event.onboarding_duration_s != actual_duration_s {
                    return Err(OnboardingFunnelValidationError::CompletionDurationMismatch {
                        session_id: journey.session_id.clone(),
                        expected_s: actual_duration_s,
                        actual_s: completed_event.onboarding_duration_s,
                    });
                }

                let expected_success_path = journey
                    .steps
                    .iter()
                    .map(|step| step.step.as_str())
                    .collect::<Vec<_>>()
                    .join(">");
                if completed_event.success_path != expected_success_path {
                    return Err(OnboardingFunnelValidationError::SuccessPathMismatch {
                        session_id: journey.session_id.clone(),
                        expected: expected_success_path,
                        actual: completed_event.success_path.clone(),
                    });
                }

                let payload = TelemetryPayload::from([
                    (
                        "onboarding_duration_s".to_string(),
                        TelemetryValue::Integer(completed_event.onboarding_duration_s as i64),
                    ),
                    (
                        "game_id".to_string(),
                        TelemetryValue::Text(completed_event.game_id.clone()),
                    ),
                    (
                        "success_path".to_string(),
                        TelemetryValue::Text(completed_event.success_path.clone()),
                    ),
                ]);
                validate_onboarding_completed_payload(&payload).map_err(|error| {
                    OnboardingFunnelValidationError::InvalidCompletionPayload {
                        session_id: journey.session_id.clone(),
                        message: error.message,
                    }
                })?;

                completed_journeys = completed_journeys.saturating_add(1);
                connect_durations_s.push((connect_at - journey.started_at_ms) as f64 / 1_000.0);
            }
            (Some(_), None) => {
                return Err(OnboardingFunnelValidationError::MissingCompletionEvent {
                    session_id: journey.session_id.clone(),
                });
            }
            (None, Some(_)) => {
                return Err(OnboardingFunnelValidationError::CompletionEventWithoutFirstConnect {
                    session_id: journey.session_id.clone(),
                });
            }
            (None, None) => {
                if let Some(last_step) = journey.steps.last() {
                    let key = last_step.step.as_str().to_string();
                    let next = step_drop_off
                        .get(&key)
                        .copied()
                        .unwrap_or(0usize)
                        .saturating_add(1);
                    step_drop_off.insert(key, next);
                }
            }
        }
    }

    let total_journeys = fixture.journeys.len();
    let completion_rate = ratio(completed_journeys, total_journeys);
    let average_time_to_connect_s = if connect_durations_s.is_empty() {
        0.0
    } else {
        connect_durations_s.iter().sum::<f64>() / connect_durations_s.len() as f64
    };

    Ok(OnboardingFunnelKpiReport {
        scenario: fixture.name.clone(),
        total_journeys,
        completed_journeys,
        completion_rate,
        average_time_to_connect_s,
        step_drop_off,
    })
}

fn validate_step_timestamps(journey: &JourneyFixture) -> Result<(), OnboardingFunnelValidationError> {
    for pair in journey.steps.windows(2) {
        if pair[1].at_ms < pair[0].at_ms {
            return Err(OnboardingFunnelValidationError::NonMonotonicStepTimestamp {
                session_id: journey.session_id.clone(),
                previous_ms: pair[0].at_ms,
                current_ms: pair[1].at_ms,
            });
        }
    }
    Ok(())
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn load_onboarding_funnel_sequences() -> Vec<OnboardingFunnelFixture> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("onboarding_event_sequences");
    let entries =
        fs::read_dir(base).expect("onboarding_event_sequences fixture directory should exist");

    let mut fixtures = Vec::new();
    for entry in entries {
        let path = entry.expect("fixture entry should be readable").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("seq") {
            continue;
        }
        let raw = fs::read_to_string(path).expect("onboarding fixture should be readable");
        fixtures.push(parse_onboarding_fixture(&raw));
    }
    fixtures
}

fn parse_onboarding_fixture(raw: &str) -> OnboardingFunnelFixture {
    let mut name = String::new();
    let mut journeys_raw = String::new();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.splitn(2, '=');
        let key = parts.next().expect("fixture key should exist").trim();
        let value = parts.next().expect("fixture value should exist").trim();
        match key {
            "name" => name = value.to_string(),
            "journeys" => journeys_raw = value.to_string(),
            _ => panic!("unknown onboarding fixture key: {key}"),
        }
    }

    OnboardingFunnelFixture {
        name,
        journeys: parse_journeys(&journeys_raw),
    }
}

fn parse_journeys(raw: &str) -> Vec<JourneyFixture> {
    raw.split(';')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('|').collect::<Vec<_>>();
            if fields.len() != 4 {
                panic!("invalid journey token: {token}");
            }
            JourneyFixture {
                session_id: fields[0].to_string(),
                started_at_ms: fields[1].parse().expect("started_at_ms should parse"),
                steps: parse_steps(fields[2]),
                completion_event: parse_completion_event(fields[3]),
            }
        })
        .collect()
}

fn parse_steps(raw: &str) -> Vec<StepFixture> {
    raw.split(',')
        .filter(|token| !token.trim().is_empty())
        .map(|token| {
            let fields = token.split('@').collect::<Vec<_>>();
            if fields.len() != 2 {
                panic!("invalid step token: {token}");
            }

            let step = match fields[0] {
                "welcome" => OnboardingStepCode::Welcome,
                "permission_check" => OnboardingStepCode::PermissionCheck,
                "relay_test" => OnboardingStepCode::RelayTest,
                "game_detection_test" => OnboardingStepCode::GameDetectionTest,
                "first_connect" => OnboardingStepCode::FirstConnect,
                _ => panic!("unsupported onboarding step code: {}", fields[0]),
            };

            StepFixture {
                step,
                at_ms: fields[1].parse().expect("step timestamp should parse"),
            }
        })
        .collect()
}

fn parse_completion_event(raw: &str) -> Option<CompletionEventFixture> {
    if raw == "_" {
        return None;
    }

    let fields = raw.split('@').collect::<Vec<_>>();
    if fields.len() != 5 || fields[0] != "onboarding_completed" {
        panic!("invalid completion event token: {raw}");
    }

    Some(CompletionEventFixture {
        at_ms: fields[1]
            .parse()
            .expect("completion event timestamp should parse"),
        onboarding_duration_s: fields[2]
            .parse()
            .expect("completion event duration should parse"),
        game_id: fields[3].to_string(),
        success_path: fields[4].to_string(),
    })
}

fn load_sequence(path: &str) -> OnboardingFunnelFixture {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("onboarding_event_sequences")
        .join(path);
    let raw = fs::read_to_string(base).expect("onboarding fixture should be readable");
    parse_onboarding_fixture(&raw)
}

fn approx_eq(left: f64, right: f64) {
    let delta = (left - right).abs();
    assert!(
        delta < 0.000_001,
        "expected {left} ~= {right}, delta={delta}"
    );
}

#[test]
fn completion_and_drop_off_kpi_derivation_is_reproducible() {
    let fixture = load_sequence("nominal_onboarding_funnel_kpi.seq");
    let report =
        run_onboarding_funnel_kpi_validation(&fixture).expect("nominal fixture should aggregate");

    assert_eq!(report.scenario, "nominal_onboarding_funnel_kpi");
    assert_eq!(report.total_journeys, 3);
    assert_eq!(report.completed_journeys, 2);
    approx_eq(report.completion_rate, 0.666_666_666_7);
    assert_eq!(
        report.step_drop_off.get("permission_check").copied(),
        Some(1)
    );

    let first = run_onboarding_funnel_kpi_validation_suite();
    let second = run_onboarding_funnel_kpi_validation_suite();
    assert_eq!(first, second);
}

#[test]
fn missing_or_invalid_event_sequences_fail_assertions() {
    let missing_completion = load_sequence("missing_completion_event_onboarding_funnel_kpi.seq");
    let missing_err = run_onboarding_funnel_kpi_validation(&missing_completion)
        .expect_err("first_connect without completion event should fail");
    assert_eq!(
        missing_err,
        OnboardingFunnelValidationError::MissingCompletionEvent {
            session_id: "sess-missing".to_string()
        }
    );

    let invalid_timing = load_sequence("invalid_event_timing_onboarding_funnel_kpi.seq");
    let invalid_timing_err = run_onboarding_funnel_kpi_validation(&invalid_timing)
        .expect_err("completion event before first_connect should fail");
    assert_eq!(
        invalid_timing_err,
        OnboardingFunnelValidationError::CompletionEventBeforeFirstConnect {
            session_id: "sess-invalid-time".to_string(),
            event_at_ms: 4_800,
            first_connect_at_ms: 5_200,
        }
    );
}

#[test]
fn time_to_connect_metric_derivation_is_validated() {
    let fixture = load_sequence("nominal_onboarding_funnel_kpi.seq");
    let report =
        run_onboarding_funnel_kpi_validation(&fixture).expect("nominal fixture should aggregate");

    approx_eq(report.average_time_to_connect_s, 5.7);
}

#[test]
fn output_supports_product_tuning_decisions() {
    let fixture = load_sequence("nominal_onboarding_funnel_kpi.seq");
    let report =
        run_onboarding_funnel_kpi_validation(&fixture).expect("nominal fixture should aggregate");
    let summary = report.tuning_summary();

    assert!(summary.contains("completion_rate=0.667"));
    assert!(summary.contains("avg_time_to_connect_s=5.700"));
    assert!(summary.contains("dominant_drop_off=permission_check:1"));
    assert!(summary.contains("recommendation=improve_permission_guidance"));
}
