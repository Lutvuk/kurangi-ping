use client_engine::detection::scanner_windows::{
    scan_processes_with, ProcessEntry, ProcessEnumerator, ScanError, ScanErrorCode, SupportedGame,
};
use client_engine::detection::state_resolver::{
    resolve_detection_state, DetectionResolverInput, DetectionState, FreshnessWindowConfig,
};
use client_engine::routing::{can_activate_routing, DetectionGateReasonCode};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScenarioSource {
    Fixture(&'static str),
    ScanError(ScanErrorCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectionScenario {
    name: &'static str,
    source: ScenarioSource,
    scanned_at_unix_ms: u64,
    last_detected_at_unix_ms: Option<u64>,
    expected_state: DetectionState,
    expected_matches: BTreeSet<String>,
    expected_gate_reason: Option<DetectionGateReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectionScenarioResult {
    name: String,
    resolved_state: DetectionState,
    matched_game_ids: BTreeSet<String>,
    false_positive_matches: usize,
    gate_allowed: bool,
    gate_reason: Option<DetectionGateReasonCode>,
}

struct ScenarioEnumerator {
    source: ScenarioSource,
}

impl ProcessEnumerator for ScenarioEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
        match &self.source {
            ScenarioSource::Fixture(path) => load_process_fixture(path),
            ScenarioSource::ScanError(code) => Err(ScanError {
                code: *code,
                message: format!("simulated scan error: {code:?}"),
            }),
        }
    }
}

fn allowlist() -> Vec<SupportedGame> {
    vec![
        SupportedGame {
            game_id: "ffxiv".to_string(),
            executable_name: "ffxiv_dx11.exe".to_string(),
            enabled: true,
        },
        SupportedGame {
            game_id: "wow".to_string(),
            executable_name: "wow.exe".to_string(),
            enabled: true,
        },
        SupportedGame {
            game_id: "swtor".to_string(),
            executable_name: "swtor.exe".to_string(),
            enabled: false,
        },
    ]
}

fn scenarios() -> Vec<DetectionScenario> {
    vec![
        DetectionScenario {
            name: "supported_mixed_exact_match",
            source: ScenarioSource::Fixture("supported_mixed.json"),
            scanned_at_unix_ms: 40_000,
            last_detected_at_unix_ms: None,
            expected_state: DetectionState::Detected,
            expected_matches: BTreeSet::from(["ffxiv".to_string(), "wow".to_string()]),
            expected_gate_reason: None,
        },
        DetectionScenario {
            name: "near_match_rejection_no_false_positive",
            source: ScenarioSource::Fixture("near_match_only.json"),
            scanned_at_unix_ms: 80_000,
            last_detected_at_unix_ms: None,
            expected_state: DetectionState::NotFound,
            expected_matches: BTreeSet::new(),
            expected_gate_reason: Some(DetectionGateReasonCode::DetectionNotFound),
        },
        DetectionScenario {
            name: "stale_state_with_expired_history",
            source: ScenarioSource::Fixture("near_match_only.json"),
            scanned_at_unix_ms: 100_000,
            last_detected_at_unix_ms: Some(70_000),
            expected_state: DetectionState::Stale,
            expected_matches: BTreeSet::new(),
            expected_gate_reason: Some(DetectionGateReasonCode::DetectionStale),
        },
        DetectionScenario {
            name: "error_state_permission_denied",
            source: ScenarioSource::ScanError(ScanErrorCode::PermissionDenied),
            scanned_at_unix_ms: 120_000,
            last_detected_at_unix_ms: Some(118_000),
            expected_state: DetectionState::Error,
            expected_matches: BTreeSet::new(),
            expected_gate_reason: Some(DetectionGateReasonCode::DetectionErrorPermissionDenied),
        },
    ]
}

fn run_detection_scenarios() -> Vec<DetectionScenarioResult> {
    let supported_games = allowlist();

    scenarios()
        .iter()
        .map(|scenario| {
            let enumerator = ScenarioEnumerator {
                source: scenario.source.clone(),
            };

            let scan = scan_processes_with(&enumerator, &supported_games);
            let (matches, scan_error) = match scan {
                Ok(matches) => (matches, None),
                Err(error) => (Vec::new(), Some(error.code)),
            };

            let resolution = resolve_detection_state(&DetectionResolverInput {
                matches: matches.clone(),
                scan_error,
                scanned_at_unix_ms: scenario.scanned_at_unix_ms,
                last_detected_at_unix_ms: scenario.last_detected_at_unix_ms,
                freshness: FreshnessWindowConfig {
                    stale_after_ms: 15_000,
                },
            });
            let gate = can_activate_routing(&resolution);

            let matched_game_ids = matches
                .iter()
                .map(|entry| entry.game_id.clone())
                .collect::<BTreeSet<_>>();
            let false_positive_matches = matches
                .iter()
                .filter(|entry| !scenario.expected_matches.contains(&entry.game_id))
                .count();

            assert_eq!(
                resolution.state, scenario.expected_state,
                "{}: detection state mismatch",
                scenario.name
            );
            assert_eq!(
                matched_game_ids, scenario.expected_matches,
                "{}: matched set mismatch",
                scenario.name
            );
            assert_eq!(
                false_positive_matches, 0,
                "{}: false positives must remain zero",
                scenario.name
            );
            assert_eq!(
                gate.reason_code, scenario.expected_gate_reason,
                "{}: gate reason mismatch",
                scenario.name
            );
            assert_eq!(
                gate.can_activate,
                scenario.expected_state == DetectionState::Detected,
                "{}: gate allow mismatch",
                scenario.name
            );

            DetectionScenarioResult {
                name: scenario.name.to_string(),
                resolved_state: resolution.state,
                matched_game_ids,
                false_positive_matches,
                gate_allowed: gate.can_activate,
                gate_reason: gate.reason_code,
            }
        })
        .collect()
}

fn load_process_fixture(name: &str) -> Result<Vec<ProcessEntry>, ScanError> {
    let path = fixture_path(name);
    let raw = fs::read_to_string(&path).map_err(|error| ScanError {
        code: ScanErrorCode::EnumerationFailed,
        message: format!("failed to read fixture {name}: {error}"),
    })?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| ScanError {
        code: ScanErrorCode::EnumerationFailed,
        message: format!("failed to parse fixture {name}: {error}"),
    })?;

    let rows = value.as_array().ok_or_else(|| ScanError {
        code: ScanErrorCode::EnumerationFailed,
        message: format!("fixture {name} must be a JSON array"),
    })?;

    rows.iter()
        .map(|row| {
            let process_id = row
                .get("process_id")
                .and_then(Value::as_u64)
                .ok_or_else(|| ScanError {
                    code: ScanErrorCode::EnumerationFailed,
                    message: format!("fixture {name} row missing process_id"),
                })? as u32;
            let executable_name = row
                .get("executable_name")
                .and_then(Value::as_str)
                .ok_or_else(|| ScanError {
                    code: ScanErrorCode::EnumerationFailed,
                    message: format!("fixture {name} row missing executable_name"),
                })?
                .to_string();
            Ok(ProcessEntry {
                process_id,
                executable_name,
            })
        })
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("process_lists")
        .join(name)
}

#[test]
fn harness_validates_detection_for_supported_process_sets() {
    let result = run_detection_scenarios();
    let supported = result
        .iter()
        .find(|entry| entry.name == "supported_mixed_exact_match")
        .expect("supported scenario must exist");

    assert_eq!(supported.resolved_state, DetectionState::Detected);
    assert_eq!(
        supported.matched_game_ids,
        BTreeSet::from(["ffxiv".to_string(), "wow".to_string()])
    );
    assert!(supported.gate_allowed);
}

#[test]
fn harness_validates_rejection_of_unsupported_and_near_match_processes() {
    let result = run_detection_scenarios();
    let near_match = result
        .iter()
        .find(|entry| entry.name == "near_match_rejection_no_false_positive")
        .expect("near-match scenario must exist");

    assert_eq!(near_match.resolved_state, DetectionState::NotFound);
    assert!(near_match.matched_game_ids.is_empty());
    assert_eq!(near_match.false_positive_matches, 0);
    assert!(!near_match.gate_allowed);
}

#[test]
fn harness_covers_stale_and_error_state_scenarios() {
    let result = run_detection_scenarios();
    let stale = result
        .iter()
        .find(|entry| entry.name == "stale_state_with_expired_history")
        .expect("stale scenario must exist");
    let error = result
        .iter()
        .find(|entry| entry.name == "error_state_permission_denied")
        .expect("error scenario must exist");

    assert_eq!(stale.resolved_state, DetectionState::Stale);
    assert_eq!(
        stale.gate_reason,
        Some(DetectionGateReasonCode::DetectionStale)
    );
    assert_eq!(error.resolved_state, DetectionState::Error);
    assert_eq!(
        error.gate_reason,
        Some(DetectionGateReasonCode::DetectionErrorPermissionDenied)
    );
}

#[test]
fn harness_results_are_deterministic_across_runs() {
    let first = run_detection_scenarios();
    let second = run_detection_scenarios();
    assert_eq!(first, second);
}
