use super::check_scheduler::UpdateChannel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOperationPhase {
    Check,
    Download,
    Verify,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverableUpdateErrorCode {
    InvalidCurrentVersion,
    InvalidReleaseVersion,
    ReleaseFeedUnavailable,
    DownloadTransportFailed,
    SignatureMismatch,
    PackageCorrupt,
    ApplyFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverableUpdateError {
    pub phase: UpdateOperationPhase,
    pub code: RecoverableUpdateErrorCode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRelease {
    pub version: String,
    pub channel: UpdateChannel,
    pub package_url: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateFeedSnapshot {
    pub available: bool,
    pub releases: Vec<UpdateRelease>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePlan {
    pub target_version: String,
    pub channel: UpdateChannel,
    pub package_url: String,
    pub expected_signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckOutcome {
    UpToDate,
    UpdateAvailable(UpdatePlan),
    RecoverableError(RecoverableUpdateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCheckResult {
    pub outcome: UpdateCheckOutcome,
    pub core_app_operational: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedUpdate {
    pub plan: UpdatePlan,
    pub package_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateDownloadOutcome {
    Downloaded(DownloadedUpdate),
    RecoverableError(RecoverableUpdateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDownloadResult {
    pub outcome: UpdateDownloadOutcome,
    pub core_app_operational: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateApplyOutcome {
    ReadyToRestart { target_version: String },
    RecoverableError(RecoverableUpdateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateApplyResult {
    pub outcome: UpdateApplyOutcome,
    pub core_app_operational: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadTransportErrorCode {
    Timeout,
    ConnectionRefused,
    BadGateway,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageVerificationErrorCode {
    SignatureMismatch,
    CorruptArchive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyUpdateErrorCode {
    PermissionDenied,
    AtomicSwapFailed,
}

pub trait UpdateDownloader {
    fn download(&mut self, package_url: &str) -> Result<Vec<u8>, DownloadTransportErrorCode>;
}

pub trait UpdateVerifier {
    fn verify(
        &self,
        package_bytes: &[u8],
        expected_signature: &str,
    ) -> Result<(), PackageVerificationErrorCode>;
}

pub trait UpdateApplier {
    fn apply(&mut self, package_bytes: &[u8], target_version: &str) -> Result<(), ApplyUpdateErrorCode>;
}

pub fn check_for_update(
    current_version: &str,
    channel: UpdateChannel,
    feed: &UpdateFeedSnapshot,
) -> UpdateCheckResult {
    if !feed.available {
        return UpdateCheckResult {
            outcome: UpdateCheckOutcome::RecoverableError(RecoverableUpdateError {
                phase: UpdateOperationPhase::Check,
                code: RecoverableUpdateErrorCode::ReleaseFeedUnavailable,
            }),
            core_app_operational: true,
        };
    }

    let Some(current) = parse_version_triplet(current_version) else {
        return UpdateCheckResult {
            outcome: UpdateCheckOutcome::RecoverableError(RecoverableUpdateError {
                phase: UpdateOperationPhase::Check,
                code: RecoverableUpdateErrorCode::InvalidCurrentVersion,
            }),
            core_app_operational: true,
        };
    };

    let mut selected_release: Option<(&UpdateRelease, (u32, u32, u32))> = None;
    for release in &feed.releases {
        if release.channel != channel {
            continue;
        }

        let Some(candidate_version) = parse_version_triplet(&release.version) else {
            return UpdateCheckResult {
                outcome: UpdateCheckOutcome::RecoverableError(RecoverableUpdateError {
                    phase: UpdateOperationPhase::Check,
                    code: RecoverableUpdateErrorCode::InvalidReleaseVersion,
                }),
                core_app_operational: true,
            };
        };

        if candidate_version <= current {
            continue;
        }

        match selected_release {
            None => selected_release = Some((release, candidate_version)),
            Some((_, selected_version)) if candidate_version > selected_version => {
                selected_release = Some((release, candidate_version))
            }
            Some(_) => {}
        }
    }

    let Some((release, _)) = selected_release else {
        return UpdateCheckResult {
            outcome: UpdateCheckOutcome::UpToDate,
            core_app_operational: true,
        };
    };

    UpdateCheckResult {
        outcome: UpdateCheckOutcome::UpdateAvailable(UpdatePlan {
            target_version: release.version.clone(),
            channel: release.channel,
            package_url: release.package_url.clone(),
            expected_signature: release.signature.clone(),
        }),
        core_app_operational: true,
    }
}

pub fn download_update(
    plan: &UpdatePlan,
    downloader: &mut dyn UpdateDownloader,
) -> UpdateDownloadResult {
    match downloader.download(&plan.package_url) {
        Ok(package_bytes) => UpdateDownloadResult {
            outcome: UpdateDownloadOutcome::Downloaded(DownloadedUpdate {
                plan: plan.clone(),
                package_bytes,
            }),
            core_app_operational: true,
        },
        Err(_) => UpdateDownloadResult {
            outcome: UpdateDownloadOutcome::RecoverableError(RecoverableUpdateError {
                phase: UpdateOperationPhase::Download,
                code: RecoverableUpdateErrorCode::DownloadTransportFailed,
            }),
            core_app_operational: true,
        },
    }
}

pub fn apply_update(
    downloaded: &DownloadedUpdate,
    verifier: &dyn UpdateVerifier,
    applier: &mut dyn UpdateApplier,
) -> UpdateApplyResult {
    if let Err(verify_error) =
        verifier.verify(&downloaded.package_bytes, &downloaded.plan.expected_signature)
    {
        return UpdateApplyResult {
            outcome: UpdateApplyOutcome::RecoverableError(RecoverableUpdateError {
                phase: UpdateOperationPhase::Verify,
                code: map_verify_error(verify_error),
            }),
            core_app_operational: true,
        };
    }

    match applier.apply(&downloaded.package_bytes, &downloaded.plan.target_version) {
        Ok(()) => UpdateApplyResult {
            outcome: UpdateApplyOutcome::ReadyToRestart {
                target_version: downloaded.plan.target_version.clone(),
            },
            core_app_operational: true,
        },
        Err(_) => UpdateApplyResult {
            outcome: UpdateApplyOutcome::RecoverableError(RecoverableUpdateError {
                phase: UpdateOperationPhase::Apply,
                code: RecoverableUpdateErrorCode::ApplyFailed,
            }),
            core_app_operational: true,
        },
    }
}

fn map_verify_error(code: PackageVerificationErrorCode) -> RecoverableUpdateErrorCode {
    match code {
        PackageVerificationErrorCode::SignatureMismatch => {
            RecoverableUpdateErrorCode::SignatureMismatch
        }
        PackageVerificationErrorCode::CorruptArchive => RecoverableUpdateErrorCode::PackageCorrupt,
    }
}

fn parse_version_triplet(raw: &str) -> Option<(u32, u32, u32)> {
    let normalized = raw.trim();
    let mut parts = normalized.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_update, check_for_update, download_update, ApplyUpdateErrorCode, DownloadTransportErrorCode,
        PackageVerificationErrorCode, UpdateApplier, UpdateApplyOutcome, UpdateChannel, UpdateCheckOutcome,
        UpdateDownloadOutcome, UpdateDownloader, UpdateFeedSnapshot, UpdateRelease, UpdateVerifier,
    };
    use std::cell::RefCell;
    use std::rc::Rc;

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
    struct OrderedVerifier {
        order: Rc<RefCell<Vec<&'static str>>>,
        result: Result<(), PackageVerificationErrorCode>,
    }

    impl UpdateVerifier for OrderedVerifier {
        fn verify(
            &self,
            _package_bytes: &[u8],
            _expected_signature: &str,
        ) -> Result<(), PackageVerificationErrorCode> {
            self.order.borrow_mut().push("verify");
            self.result
        }
    }

    #[derive(Debug)]
    struct OrderedApplier {
        order: Rc<RefCell<Vec<&'static str>>>,
        result: Result<(), ApplyUpdateErrorCode>,
    }

    impl UpdateApplier for OrderedApplier {
        fn apply(
            &mut self,
            _package_bytes: &[u8],
            _target_version: &str,
        ) -> Result<(), ApplyUpdateErrorCode> {
            self.order.borrow_mut().push("apply");
            self.result
        }
    }

    fn feed_with_release(version: &str, channel: UpdateChannel) -> UpdateFeedSnapshot {
        UpdateFeedSnapshot {
            available: true,
            releases: vec![UpdateRelease {
                version: version.to_string(),
                channel,
                package_url: "https://updates.example.com/kurangi-ping.zip".to_string(),
                signature: "sig-v1".to_string(),
            }],
        }
    }

    #[test]
    fn update_package_verification_occurs_before_apply() {
        let check = check_for_update("1.0.0", UpdateChannel::Stable, &feed_with_release("1.1.0", UpdateChannel::Stable));
        let plan = match check.outcome {
            UpdateCheckOutcome::UpdateAvailable(plan) => plan,
            other => panic!("expected update available, got: {other:?}"),
        };

        let mut downloader = StubDownloader {
            result: Ok(vec![1, 2, 3, 4]),
        };
        let downloaded = match download_update(&plan, &mut downloader).outcome {
            UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
            other => panic!("expected downloaded update, got: {other:?}"),
        };

        let order = Rc::new(RefCell::new(Vec::new()));
        let verifier = OrderedVerifier {
            order: Rc::clone(&order),
            result: Ok(()),
        };
        let mut applier = OrderedApplier {
            order: Rc::clone(&order),
            result: Ok(()),
        };

        let apply = apply_update(&downloaded, &verifier, &mut applier);
        assert!(matches!(apply.outcome, UpdateApplyOutcome::ReadyToRestart { .. }));
        assert_eq!(&*order.borrow(), &["verify", "apply"]);
    }

    #[test]
    fn failed_downloads_and_applies_move_to_recoverable_error_state() {
        let check = check_for_update("1.0.0", UpdateChannel::Stable, &feed_with_release("1.1.0", UpdateChannel::Stable));
        let plan = match check.outcome {
            UpdateCheckOutcome::UpdateAvailable(plan) => plan,
            other => panic!("expected update available, got: {other:?}"),
        };

        let mut failing_downloader = StubDownloader {
            result: Err(DownloadTransportErrorCode::Timeout),
        };
        let download_result = download_update(&plan, &mut failing_downloader);
        assert!(matches!(
            download_result.outcome,
            UpdateDownloadOutcome::RecoverableError(_)
        ));

        let mut ok_downloader = StubDownloader {
            result: Ok(vec![9, 9, 9]),
        };
        let downloaded = match download_update(&plan, &mut ok_downloader).outcome {
            UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
            other => panic!("expected downloaded update, got: {other:?}"),
        };

        let order = Rc::new(RefCell::new(Vec::new()));
        let verifier = OrderedVerifier {
            order: Rc::clone(&order),
            result: Ok(()),
        };
        let mut failing_applier = OrderedApplier {
            order: Rc::clone(&order),
            result: Err(ApplyUpdateErrorCode::AtomicSwapFailed),
        };

        let apply_result = apply_update(&downloaded, &verifier, &mut failing_applier);
        assert!(matches!(
            apply_result.outcome,
            UpdateApplyOutcome::RecoverableError(_)
        ));
        assert_eq!(&*order.borrow(), &["verify", "apply"]);
    }

    #[test]
    fn core_app_remains_operational_after_updater_failure() {
        let check = check_for_update(
            "invalid-version",
            UpdateChannel::Stable,
            &feed_with_release("1.1.0", UpdateChannel::Stable),
        );
        assert!(!matches!(check.outcome, UpdateCheckOutcome::UpdateAvailable(_)));
        assert!(check.core_app_operational);

        let plan = super::UpdatePlan {
            target_version: "1.1.0".to_string(),
            channel: UpdateChannel::Stable,
            package_url: "https://updates.example.com/kurangi-ping.zip".to_string(),
            expected_signature: "sig-v1".to_string(),
        };
        let mut failing_downloader = StubDownloader {
            result: Err(DownloadTransportErrorCode::ConnectionRefused),
        };
        let download_result = download_update(&plan, &mut failing_downloader);
        assert!(download_result.core_app_operational);
    }

    #[test]
    fn apply_flow_is_deterministic_and_test_covered() {
        let feed = UpdateFeedSnapshot {
            available: true,
            releases: vec![
                UpdateRelease {
                    version: "1.1.0".to_string(),
                    channel: UpdateChannel::Stable,
                    package_url: "https://updates.example.com/stable.zip".to_string(),
                    signature: "sig-stable".to_string(),
                },
                UpdateRelease {
                    version: "1.2.0".to_string(),
                    channel: UpdateChannel::Stable,
                    package_url: "https://updates.example.com/stable-1-2.zip".to_string(),
                    signature: "sig-stable-1-2".to_string(),
                },
            ],
        };

        let run_once = || {
            let check = check_for_update("1.0.0", UpdateChannel::Stable, &feed);
            let plan = match &check.outcome {
                UpdateCheckOutcome::UpdateAvailable(plan) => plan.clone(),
                other => panic!("expected update available, got: {other:?}"),
            };

            let mut downloader = StubDownloader {
                result: Ok(vec![7, 7, 7, 7]),
            };
            let downloaded = match download_update(&plan, &mut downloader).outcome {
                UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
                other => panic!("expected downloaded update, got: {other:?}"),
            };

            let order = Rc::new(RefCell::new(Vec::new()));
            let verifier = OrderedVerifier {
                order: Rc::clone(&order),
                result: Ok(()),
            };
            let mut applier = OrderedApplier {
                order: Rc::clone(&order),
                result: Ok(()),
            };
            let apply = apply_update(&downloaded, &verifier, &mut applier);
            let order_log = order.borrow().clone();
            (check, downloaded, apply, order_log)
        };

        let first = run_once();
        let second = run_once();
        assert_eq!(first, second);
    }

    #[test]
    fn apply_is_blocked_when_verification_fails() {
        let check =
            check_for_update("1.0.0", UpdateChannel::Stable, &feed_with_release("1.1.0", UpdateChannel::Stable));
        let plan = match check.outcome {
            UpdateCheckOutcome::UpdateAvailable(plan) => plan,
            other => panic!("expected update available, got: {other:?}"),
        };
        let mut downloader = StubDownloader {
            result: Ok(vec![1, 2, 3]),
        };
        let downloaded = match download_update(&plan, &mut downloader).outcome {
            UpdateDownloadOutcome::Downloaded(downloaded) => downloaded,
            other => panic!("expected downloaded update, got: {other:?}"),
        };

        let order = Rc::new(RefCell::new(Vec::new()));
        let verifier = OrderedVerifier {
            order: Rc::clone(&order),
            result: Err(PackageVerificationErrorCode::SignatureMismatch),
        };
        let mut applier = OrderedApplier {
            order: Rc::clone(&order),
            result: Ok(()),
        };

        let apply = apply_update(&downloaded, &verifier, &mut applier);
        assert!(matches!(
            apply.outcome,
            UpdateApplyOutcome::RecoverableError(_)
        ));
        assert_eq!(&*order.borrow(), &["verify"]);
    }
}
