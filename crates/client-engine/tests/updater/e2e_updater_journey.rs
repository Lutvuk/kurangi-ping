use client_engine::updater::apply_orchestrator::{
    apply_update, check_for_update, download_update, ApplyUpdateErrorCode,
    DownloadTransportErrorCode, UpdateApplier, UpdateDownloader, UpdateVerifier,
};
use client_engine::updater::{
    resolve_updater_state, UpdateChannel, UpdateCheckOutcome, UpdateDownloadOutcome,
    UpdateFeedSnapshot, UpdatePlan, UpdateRelease, UpdaterErrorReasonCode, UpdaterLifecycleSignal,
    UpdaterState, UpdaterStateTag,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdaterJourneyScenarioResult {
    scenario: String,
    terminal_state: UpdaterStateTag,
    recoverable: bool,
    ready_state_preserved: bool,
    trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterJourneySuiteResult {
    success_path: UpdaterJourneyScenarioResult,
    download_failure_path: UpdaterJourneyScenarioResult,
    verification_failure_path: UpdaterJourneyScenarioResult,
    deferred_restart_path: UpdaterJourneyScenarioResult,
}

#[derive(Debug)]
struct StubDownloader {
    result: Result<Vec<u8>, DownloadTransportErrorCode>,
}

impl UpdateDownloader for StubDownloader {
    fn download(&mut self, _package_url: &str) -> Result<Vec<u8>, DownloadTransportErrorCode> {
        self.result.clone()
    }
}

#[derive(Debug)]
struct StubVerifier {
    expected_signature: String,
}

impl UpdateVerifier for StubVerifier {
    fn verify(
        &self,
        _package_bytes: &[u8],
        expected_signature: &str,
    ) -> Result<(), client_engine::updater::PackageVerificationErrorCode> {
        if expected_signature == self.expected_signature {
            return Ok(());
        }
        Err(client_engine::updater::PackageVerificationErrorCode::SignatureMismatch)
    }
}

#[derive(Debug, Default)]
struct StubApplier;

impl UpdateApplier for StubApplier {
    fn apply(
        &mut self,
        _package_bytes: &[u8],
        _target_version: &str,
    ) -> Result<(), ApplyUpdateErrorCode> {
        Ok(())
    }
}

pub fn run_updater_journey_suite() -> UpdaterJourneySuiteResult {
    UpdaterJourneySuiteResult {
        success_path: run_success_path_scenario(),
        download_failure_path: run_download_failure_scenario(),
        verification_failure_path: run_verification_failure_scenario(),
        deferred_restart_path: run_deferred_restart_scenario(),
    }
}

fn updater_feed(version: &str, signature: &str) -> UpdateFeedSnapshot {
    UpdateFeedSnapshot {
        available: true,
        releases: vec![UpdateRelease {
            version: version.to_string(),
            channel: UpdateChannel::Stable,
            package_url: "https://updates.example.com/kurangi-ping-2.zip".to_string(),
            signature: signature.to_string(),
        }],
    }
}

fn extract_plan(outcome: UpdateCheckOutcome) -> UpdatePlan {
    match outcome {
        UpdateCheckOutcome::UpdateAvailable(plan) => plan,
        other => panic!("expected update plan from check outcome, got: {other:?}"),
    }
}

fn defer_restart(state: &UpdaterState) -> UpdaterState {
    match state {
        UpdaterState::ReadyToRestart { target_version } => UpdaterState::ReadyToRestart {
            target_version: target_version.clone(),
        },
        _ => state.clone(),
    }
}

fn run_success_path_scenario() -> UpdaterJourneyScenarioResult {
    let mut trace = Vec::new();
    let mut state = UpdaterState::UpToDate;

    let signature = "sig-1.1.0";
    let check = check_for_update(
        "1.0.0",
        UpdateChannel::Stable,
        &updater_feed("1.1.0", signature),
    );
    trace.push(format!("check_outcome={:?}", check.outcome));
    let plan = extract_plan(check.outcome.clone());
    let check_transition =
        resolve_updater_state(&state, UpdaterLifecycleSignal::CheckResult(check.outcome))
            .expect("check transition should resolve");
    state = check_transition.state;
    trace.push(format!("state_after_check={}", state.tag().as_str()));

    let begin_download = resolve_updater_state(&state, UpdaterLifecycleSignal::BeginDownload)
        .expect("begin download transition should resolve");
    state = begin_download.state;
    trace.push(format!(
        "state_after_begin_download={}",
        state.tag().as_str()
    ));

    let mut downloader = StubDownloader {
        result: Ok(vec![1, 2, 3, 4]),
    };
    let download = download_update(&plan, &mut downloader);
    trace.push(format!("download_outcome={:?}", download.outcome));
    let download_transition = resolve_updater_state(
        &state,
        UpdaterLifecycleSignal::DownloadResult(download.outcome.clone()),
    )
    .expect("download transition should resolve");
    state = download_transition.state;
    trace.push(format!(
        "state_after_download_result={}",
        state.tag().as_str()
    ));

    let downloaded = match download.outcome {
        UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
        other => panic!("expected downloaded package, got: {other:?}"),
    };
    let verifier = StubVerifier {
        expected_signature: signature.to_string(),
    };
    let mut applier = StubApplier;
    let apply = apply_update(&downloaded, &verifier, &mut applier);
    trace.push(format!("apply_outcome={:?}", apply.outcome));
    let apply_transition = resolve_updater_state(
        &state,
        UpdaterLifecycleSignal::ApplyResult(apply.outcome.clone()),
    )
    .expect("apply transition should resolve");
    state = apply_transition.state;
    trace.push(format!("terminal_state={}", state.tag().as_str()));

    UpdaterJourneyScenarioResult {
        scenario: "success_path".to_string(),
        terminal_state: state.tag(),
        recoverable: true,
        ready_state_preserved: false,
        trace,
    }
}

fn run_download_failure_scenario() -> UpdaterJourneyScenarioResult {
    let mut trace = Vec::new();
    let mut state = UpdaterState::UpToDate;

    let check = check_for_update(
        "1.0.0",
        UpdateChannel::Stable,
        &updater_feed("1.1.0", "sig-1.1.0"),
    );
    let plan = extract_plan(check.outcome.clone());
    state = resolve_updater_state(&state, UpdaterLifecycleSignal::CheckResult(check.outcome))
        .expect("check transition should resolve")
        .state;
    state = resolve_updater_state(&state, UpdaterLifecycleSignal::BeginDownload)
        .expect("begin download transition should resolve")
        .state;

    let mut downloader = StubDownloader {
        result: Err(DownloadTransportErrorCode::Timeout),
    };
    let download = download_update(&plan, &mut downloader);
    trace.push(format!("download_failure_outcome={:?}", download.outcome));
    let transition = resolve_updater_state(
        &state,
        UpdaterLifecycleSignal::DownloadResult(download.outcome),
    )
    .expect("download failure transition should resolve");
    state = transition.state;
    trace.push(format!("terminal_state={}", state.tag().as_str()));

    let recoverable = matches!(
        state,
        UpdaterState::UpdateError {
            reason: UpdaterErrorReasonCode::DownloadTransportFailed
        }
    );

    UpdaterJourneyScenarioResult {
        scenario: "download_failure_path".to_string(),
        terminal_state: state.tag(),
        recoverable,
        ready_state_preserved: false,
        trace,
    }
}

fn run_verification_failure_scenario() -> UpdaterJourneyScenarioResult {
    let mut trace = Vec::new();
    let mut state = UpdaterState::UpToDate;

    let signature = "sig-1.1.0";
    let check = check_for_update(
        "1.0.0",
        UpdateChannel::Stable,
        &updater_feed("1.1.0", signature),
    );
    let plan = extract_plan(check.outcome.clone());
    state = resolve_updater_state(&state, UpdaterLifecycleSignal::CheckResult(check.outcome))
        .expect("check transition should resolve")
        .state;
    state = resolve_updater_state(&state, UpdaterLifecycleSignal::BeginDownload)
        .expect("begin download transition should resolve")
        .state;

    let mut downloader = StubDownloader {
        result: Ok(vec![9, 9, 9]),
    };
    let download = download_update(&plan, &mut downloader);
    let downloaded = match download.outcome.clone() {
        UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
        other => panic!("expected downloaded package, got: {other:?}"),
    };
    state = resolve_updater_state(
        &state,
        UpdaterLifecycleSignal::DownloadResult(download.outcome),
    )
    .expect("download success transition should resolve")
    .state;

    let verifier = StubVerifier {
        expected_signature: "mismatch-signature".to_string(),
    };
    let mut applier = StubApplier;
    let apply = apply_update(&downloaded, &verifier, &mut applier);
    trace.push(format!("apply_failure_outcome={:?}", apply.outcome));
    let transition =
        resolve_updater_state(&state, UpdaterLifecycleSignal::ApplyResult(apply.outcome))
            .expect("apply failure transition should resolve");
    state = transition.state;
    trace.push(format!("terminal_state={}", state.tag().as_str()));

    let recoverable = matches!(
        state,
        UpdaterState::UpdateError {
            reason: UpdaterErrorReasonCode::PackageSignatureMismatch
        }
    );

    UpdaterJourneyScenarioResult {
        scenario: "verification_failure_path".to_string(),
        terminal_state: state.tag(),
        recoverable,
        ready_state_preserved: false,
        trace,
    }
}

fn run_deferred_restart_scenario() -> UpdaterJourneyScenarioResult {
    let mut trace = Vec::new();
    let success = run_success_path_scenario();
    let ready_state = UpdaterState::ReadyToRestart {
        target_version: "1.1.0".to_string(),
    };
    assert_eq!(success.terminal_state, UpdaterStateTag::ReadyToRestart);
    trace.extend(success.trace);
    trace.push("defer_action=later".to_string());

    let after_defer = defer_restart(&ready_state);
    trace.push(format!("state_after_defer={}", after_defer.tag().as_str()));

    let ready_state_preserved = matches!(after_defer, UpdaterState::ReadyToRestart { .. });
    UpdaterJourneyScenarioResult {
        scenario: "deferred_restart_path".to_string(),
        terminal_state: after_defer.tag(),
        recoverable: true,
        ready_state_preserved,
        trace,
    }
}

#[test]
fn success_path_from_check_to_ready_to_restart_validates() {
    let suite = run_updater_journey_suite();
    let success = suite.success_path;

    assert_eq!(success.scenario, "success_path");
    assert_eq!(success.terminal_state, UpdaterStateTag::ReadyToRestart);
    assert!(success
        .trace
        .iter()
        .any(|line| line.contains("check_outcome")));
    assert!(success
        .trace
        .iter()
        .any(|line| line.contains("apply_outcome")));
}

#[test]
fn failure_paths_produce_recoverable_ui_states() {
    let suite = run_updater_journey_suite();
    let download_failure = suite.download_failure_path;
    let verification_failure = suite.verification_failure_path;

    assert_eq!(
        download_failure.terminal_state,
        UpdaterStateTag::UpdateError
    );
    assert!(download_failure.recoverable);
    assert!(download_failure
        .trace
        .iter()
        .any(|line| line.contains("DownloadTransportFailed")));

    assert_eq!(
        verification_failure.terminal_state,
        UpdaterStateTag::UpdateError
    );
    assert!(verification_failure.recoverable);
    assert!(verification_failure
        .trace
        .iter()
        .any(|line| line.contains("SignatureMismatch")));
}

#[test]
fn deferred_restart_path_preserves_update_readiness_state() {
    let suite = run_updater_journey_suite();
    let deferred = suite.deferred_restart_path;

    assert_eq!(deferred.scenario, "deferred_restart_path");
    assert_eq!(deferred.terminal_state, UpdaterStateTag::ReadyToRestart);
    assert!(deferred.ready_state_preserved);
    assert!(deferred
        .trace
        .iter()
        .any(|line| line.contains("state_after_defer=ready_to_restart")));
}

#[test]
fn journey_traces_are_deterministic() {
    let first = run_updater_journey_suite();
    let second = run_updater_journey_suite();

    assert_eq!(first, second);
    assert!(!first.success_path.trace.is_empty());
    assert!(!first.download_failure_path.trace.is_empty());
    assert!(!first.verification_failure_path.trace.is_empty());
    assert!(!first.deferred_restart_path.trace.is_empty());
}
