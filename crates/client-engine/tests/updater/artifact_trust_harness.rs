use client_engine::updater::apply_orchestrator::{
    apply_update, ApplyUpdateErrorCode, DownloadedUpdate, PackageVerificationErrorCode,
    RecoverableUpdateErrorCode, UpdateApplier, UpdateVerifier,
};
use client_engine::updater::{
    map_updater_error_reason, UpdateChannel, UpdatePlan, UpdaterErrorReasonCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrustScenarioExpectation {
    Pass,
    Fail {
        recoverable_code: RecoverableUpdateErrorCode,
        taxonomy_reason: UpdaterErrorReasonCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustScenario {
    id: &'static str,
    package_bytes: Vec<u8>,
    expected_signature: String,
    expectation: TrustScenarioExpectation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustScenarioResult {
    id: &'static str,
    passed: bool,
    observed_recoverable_code: Option<RecoverableUpdateErrorCode>,
    observed_taxonomy_reason: Option<UpdaterErrorReasonCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactTrustSuiteReport {
    scenario_results: Vec<TrustScenarioResult>,
    reproducible_output: String,
}

#[derive(Debug, Default)]
struct NoopApplier;

impl UpdateApplier for NoopApplier {
    fn apply(
        &mut self,
        _package_bytes: &[u8],
        _target_version: &str,
    ) -> Result<(), ApplyUpdateErrorCode> {
        Ok(())
    }
}

#[derive(Debug, Default)]
struct DeterministicTrustVerifier;

impl DeterministicTrustVerifier {
    fn signature_for(package_bytes: &[u8]) -> String {
        let checksum = package_bytes.iter().fold(0_u64, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(u64::from(*byte))
        });
        format!("kp-sig:{checksum:016x}")
    }
}

impl UpdateVerifier for DeterministicTrustVerifier {
    fn verify(
        &self,
        package_bytes: &[u8],
        expected_signature: &str,
    ) -> Result<(), PackageVerificationErrorCode> {
        if expected_signature.trim().is_empty() {
            return Err(PackageVerificationErrorCode::SignatureMismatch);
        }

        if !package_bytes.starts_with(b"KPUP:") {
            return Err(PackageVerificationErrorCode::CorruptArchive);
        }

        let observed_signature = Self::signature_for(package_bytes);
        if observed_signature != expected_signature {
            return Err(PackageVerificationErrorCode::SignatureMismatch);
        }

        Ok(())
    }
}

fn build_downloaded_update(package_bytes: Vec<u8>, expected_signature: String) -> DownloadedUpdate {
    DownloadedUpdate {
        plan: UpdatePlan {
            target_version: "1.1.0".to_string(),
            channel: UpdateChannel::Stable,
            package_url: "https://updates.example.com/kurangi-ping-1.1.0.zip".to_string(),
            expected_signature,
        },
        package_bytes,
    }
}

fn trust_scenarios() -> Vec<TrustScenario> {
    let trusted_bytes = b"KPUP:kurangi-ping-release-1.1.0".to_vec();
    let trusted_signature = DeterministicTrustVerifier::signature_for(&trusted_bytes);
    let tampered_bytes = b"KPUP:kurangi-ping-release-1.1.0-tampered".to_vec();
    let corrupt_bytes = b"NOT-A-VALID-UPDATER-PACKAGE".to_vec();

    vec![
        TrustScenario {
            id: "trusted_signed_artifact",
            package_bytes: trusted_bytes.clone(),
            expected_signature: trusted_signature.clone(),
            expectation: TrustScenarioExpectation::Pass,
        },
        TrustScenario {
            id: "unsigned_artifact_rejected",
            package_bytes: trusted_bytes,
            expected_signature: String::new(),
            expectation: TrustScenarioExpectation::Fail {
                recoverable_code: RecoverableUpdateErrorCode::SignatureMismatch,
                taxonomy_reason: UpdaterErrorReasonCode::PackageSignatureMismatch,
            },
        },
        TrustScenario {
            id: "tampered_artifact_rejected",
            package_bytes: tampered_bytes,
            expected_signature: trusted_signature,
            expectation: TrustScenarioExpectation::Fail {
                recoverable_code: RecoverableUpdateErrorCode::SignatureMismatch,
                taxonomy_reason: UpdaterErrorReasonCode::PackageSignatureMismatch,
            },
        },
        TrustScenario {
            id: "corrupt_artifact_rejected",
            package_bytes: corrupt_bytes,
            expected_signature: "kp-sig:0000000000000001".to_string(),
            expectation: TrustScenarioExpectation::Fail {
                recoverable_code: RecoverableUpdateErrorCode::PackageCorrupt,
                taxonomy_reason: UpdaterErrorReasonCode::PackageCorrupt,
            },
        },
    ]
}

fn execute_scenario(
    verifier: &DeterministicTrustVerifier,
    scenario: &TrustScenario,
) -> TrustScenarioResult {
    let downloaded = build_downloaded_update(
        scenario.package_bytes.clone(),
        scenario.expected_signature.clone(),
    );
    let mut applier = NoopApplier;
    let apply = apply_update(&downloaded, verifier, &mut applier);

    match (&scenario.expectation, apply.outcome) {
        (
            TrustScenarioExpectation::Pass,
            client_engine::updater::UpdateApplyOutcome::ReadyToRestart { .. },
        ) => TrustScenarioResult {
            id: scenario.id,
            passed: true,
            observed_recoverable_code: None,
            observed_taxonomy_reason: None,
        },
        (
            TrustScenarioExpectation::Fail {
                recoverable_code,
                taxonomy_reason,
            },
            client_engine::updater::UpdateApplyOutcome::RecoverableError(error),
        ) => {
            let mapped_reason = map_updater_error_reason(Some(&error), None);
            let passed = error.code == *recoverable_code && mapped_reason == *taxonomy_reason;
            TrustScenarioResult {
                id: scenario.id,
                passed,
                observed_recoverable_code: Some(error.code),
                observed_taxonomy_reason: Some(mapped_reason),
            }
        }
        _ => TrustScenarioResult {
            id: scenario.id,
            passed: false,
            observed_recoverable_code: None,
            observed_taxonomy_reason: None,
        },
    }
}

fn build_reproducible_output(results: &[TrustScenarioResult]) -> String {
    let mut rows = results
        .iter()
        .map(|result| {
            let recoverable = result
                .observed_recoverable_code
                .map(|code| format!("{code:?}"))
                .unwrap_or_else(|| "-".to_string());
            let reason = result
                .observed_taxonomy_reason
                .map(|code| code.as_code().to_string())
                .unwrap_or_else(|| "-".to_string());
            format!(
                "{}|{}|{}|{}",
                result.id,
                if result.passed { "pass" } else { "fail" },
                recoverable,
                reason
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows.join("\n")
}

fn run_artifact_trust_suite() -> ArtifactTrustSuiteReport {
    let verifier = DeterministicTrustVerifier;
    let scenario_results = trust_scenarios()
        .iter()
        .map(|scenario| execute_scenario(&verifier, scenario))
        .collect::<Vec<_>>();
    let reproducible_output = build_reproducible_output(&scenario_results);
    ArtifactTrustSuiteReport {
        scenario_results,
        reproducible_output,
    }
}

#[test]
fn trusted_signed_artifact_path_passes() {
    let report = run_artifact_trust_suite();
    let trusted = report
        .scenario_results
        .iter()
        .find(|result| result.id == "trusted_signed_artifact")
        .expect("trusted scenario should exist");
    assert!(trusted.passed);
    assert_eq!(trusted.observed_recoverable_code, None);
    assert_eq!(trusted.observed_taxonomy_reason, None);
}

#[test]
fn unsigned_and_tampered_artifact_paths_fail_deterministically() {
    let report = run_artifact_trust_suite();
    let unsigned = report
        .scenario_results
        .iter()
        .find(|result| result.id == "unsigned_artifact_rejected")
        .expect("unsigned scenario should exist");
    let tampered = report
        .scenario_results
        .iter()
        .find(|result| result.id == "tampered_artifact_rejected")
        .expect("tampered scenario should exist");

    assert!(unsigned.passed);
    assert_eq!(
        unsigned.observed_recoverable_code,
        Some(RecoverableUpdateErrorCode::SignatureMismatch)
    );
    assert_eq!(
        unsigned.observed_taxonomy_reason,
        Some(UpdaterErrorReasonCode::PackageSignatureMismatch)
    );

    assert!(tampered.passed);
    assert_eq!(
        tampered.observed_recoverable_code,
        Some(RecoverableUpdateErrorCode::SignatureMismatch)
    );
    assert_eq!(
        tampered.observed_taxonomy_reason,
        Some(UpdaterErrorReasonCode::PackageSignatureMismatch)
    );
}

#[test]
fn failure_reasons_map_to_updater_error_taxonomy() {
    let report = run_artifact_trust_suite();
    let corrupt = report
        .scenario_results
        .iter()
        .find(|result| result.id == "corrupt_artifact_rejected")
        .expect("corrupt scenario should exist");

    assert!(corrupt.passed);
    assert_eq!(
        corrupt.observed_recoverable_code,
        Some(RecoverableUpdateErrorCode::PackageCorrupt)
    );
    assert_eq!(
        corrupt.observed_taxonomy_reason,
        Some(UpdaterErrorReasonCode::PackageCorrupt)
    );
}

#[test]
fn harness_output_is_reproducible() {
    let first = run_artifact_trust_suite();
    let second = run_artifact_trust_suite();
    assert_eq!(first.reproducible_output, second.reproducible_output);
}
