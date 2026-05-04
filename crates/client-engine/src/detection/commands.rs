//! Manual rescan command handler.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::scanner_windows::{
    scan_processes_with, ProcessEnumerator, SupportedGame, WindowsProcessEnumerator,
};
use super::state_resolver::{
    resolve_detection_state, DetectionResolution, DetectionResolverInput, FreshnessWindowConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescanCommandErrorCode {
    ScanAlreadyInProgress,
    InternalStateUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RescanCommandError {
    pub code: RescanCommandErrorCode,
    pub message: String,
}

impl RescanCommandError {
    fn new(code: RescanCommandErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RescanCommandConfig {
    pub scan_interval_ms: u64,
    pub freshness: FreshnessWindowConfig,
}

impl Default for RescanCommandConfig {
    fn default() -> Self {
        Self {
            scan_interval_ms: 3_000,
            freshness: FreshnessWindowConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RescanRuntimeState {
    pub in_progress: bool,
    pub last_detected_at_unix_ms: Option<u64>,
    pub next_scheduled_scan_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RescanCommandOutput {
    pub resolution: DetectionResolution,
    pub next_scheduled_scan_at_unix_ms: u64,
}

pub trait TimeProvider {
    fn now_unix_ms(&self) -> u64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now_unix_ms(&self) -> u64 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        duration.as_millis() as u64
    }
}

pub struct RescanCommandHandler<E: ProcessEnumerator, T: TimeProvider> {
    scanner: E,
    clock: T,
    config: RescanCommandConfig,
    runtime: Mutex<RescanRuntimeState>,
}

impl<E: ProcessEnumerator, T: TimeProvider> RescanCommandHandler<E, T> {
    pub fn new(scanner: E, clock: T, config: RescanCommandConfig) -> Self {
        Self {
            scanner,
            clock,
            config,
            runtime: Mutex::new(RescanRuntimeState::default()),
        }
    }

    pub fn with_runtime_state(
        scanner: E,
        clock: T,
        config: RescanCommandConfig,
        runtime: RescanRuntimeState,
    ) -> Self {
        Self {
            scanner,
            clock,
            config,
            runtime: Mutex::new(runtime),
        }
    }

    pub fn trigger_rescan(
        &self,
        allowlist: &[SupportedGame],
    ) -> Result<RescanCommandOutput, RescanCommandError> {
        let last_detected_at_unix_ms = {
            let mut runtime = self.runtime.lock().map_err(|_| {
                RescanCommandError::new(
                    RescanCommandErrorCode::InternalStateUnavailable,
                    "failed to acquire rescan runtime lock",
                )
            })?;

            if runtime.in_progress {
                return Err(RescanCommandError::new(
                    RescanCommandErrorCode::ScanAlreadyInProgress,
                    "manual rescan rejected: scan already in progress",
                ));
            }

            runtime.in_progress = true;
            runtime.last_detected_at_unix_ms
        };

        let scanned_at_unix_ms = self.clock.now_unix_ms();
        let scan_result = scan_processes_with(&self.scanner, allowlist);
        let (matches, scan_error) = match scan_result {
            Ok(matches) => (matches, None),
            Err(error) => (Vec::new(), Some(error.code)),
        };

        let resolution = resolve_detection_state(&DetectionResolverInput {
            matches: matches.clone(),
            scan_error,
            scanned_at_unix_ms,
            last_detected_at_unix_ms,
            freshness: self.config.freshness,
        });

        let has_live_match = !matches.is_empty();
        let next_scheduled_scan_at_unix_ms = scanned_at_unix_ms.saturating_add(self.config.scan_interval_ms);

        {
            let mut runtime = self.runtime.lock().map_err(|_| {
                RescanCommandError::new(
                    RescanCommandErrorCode::InternalStateUnavailable,
                    "failed to acquire rescan runtime lock for commit",
                )
            })?;

            runtime.in_progress = false;
            runtime.next_scheduled_scan_at_unix_ms = Some(next_scheduled_scan_at_unix_ms);
            if has_live_match {
                runtime.last_detected_at_unix_ms = Some(scanned_at_unix_ms);
            }
        }

        Ok(RescanCommandOutput {
            resolution,
            next_scheduled_scan_at_unix_ms,
        })
    }

    pub fn snapshot_runtime(&self) -> Result<RescanRuntimeState, RescanCommandError> {
        self.runtime
            .lock()
            .map(|state| *state)
            .map_err(|_| {
                RescanCommandError::new(
                    RescanCommandErrorCode::InternalStateUnavailable,
                    "failed to read rescan runtime state",
                )
            })
    }
}

pub type DefaultRescanHandler = RescanCommandHandler<WindowsProcessEnumerator, SystemTimeProvider>;

pub fn trigger_rescan<E: ProcessEnumerator, T: TimeProvider>(
    handler: &RescanCommandHandler<E, T>,
    allowlist: &[SupportedGame],
) -> Result<RescanCommandOutput, RescanCommandError> {
    handler.trigger_rescan(allowlist)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use super::{
        trigger_rescan, RescanCommandConfig, RescanCommandErrorCode, RescanCommandHandler,
        RescanRuntimeState, TimeProvider,
    };
    use crate::detection::scanner_windows::{
        ProcessEntry, ProcessEnumerator, ScanError, ScanErrorCode, SupportedGame,
    };
    use crate::detection::state_resolver::{DetectionReasonCode, DetectionState, FreshnessWindowConfig};

    #[derive(Clone)]
    struct FixedClock {
        now_unix_ms: u64,
    }

    impl TimeProvider for FixedClock {
        fn now_unix_ms(&self) -> u64 {
            self.now_unix_ms
        }
    }

    struct StubScanner {
        processes: Vec<ProcessEntry>,
        calls: Arc<AtomicUsize>,
    }

    impl ProcessEnumerator for StubScanner {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.processes.clone())
        }
    }

    struct ErrorScanner {
        code: ScanErrorCode,
    }

    impl ProcessEnumerator for ErrorScanner {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
            Err(ScanError {
                code: self.code,
                message: "simulated scanner failure".to_string(),
            })
        }
    }

    struct BlockingScanner {
        started_tx: Sender<()>,
        release_rx: Mutex<Receiver<()>>,
    }

    impl ProcessEnumerator for BlockingScanner {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
            let _ = self.started_tx.send(());
            let _ = self.release_rx.lock().expect("release lock").recv();
            Ok(vec![ProcessEntry {
                process_id: 99,
                executable_name: "ffxiv_dx11.exe".to_string(),
            }])
        }
    }

    fn allowlist() -> Vec<SupportedGame> {
        vec![SupportedGame {
            game_id: "ffxiv".to_string(),
            executable_name: "ffxiv_dx11.exe".to_string(),
            enabled: true,
        }]
    }

    #[test]
    fn rescan_bypasses_periodic_interval_wait() {
        let calls = Arc::new(AtomicUsize::new(0));
        let handler = RescanCommandHandler::with_runtime_state(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 222,
                    executable_name: "ffxiv_dx11.exe".to_string(),
                }],
                calls: calls.clone(),
            },
            FixedClock { now_unix_ms: 8_000 },
            RescanCommandConfig {
                scan_interval_ms: 10_000,
                freshness: FreshnessWindowConfig {
                    stale_after_ms: 5_000,
                },
            },
            RescanRuntimeState {
                in_progress: false,
                last_detected_at_unix_ms: None,
                next_scheduled_scan_at_unix_ms: Some(20_000),
            },
        );

        let output = handler
            .trigger_rescan(&allowlist())
            .expect("manual rescan should bypass periodic wait");

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(output.resolution.state, DetectionState::Detected);
    }

    #[test]
    fn command_returns_updated_normalized_state() {
        let calls = Arc::new(AtomicUsize::new(0));
        let handler = RescanCommandHandler::new(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 777,
                    executable_name: "ffxiv_dx11.exe".to_string(),
                }],
                calls,
            },
            FixedClock { now_unix_ms: 30_000 },
            RescanCommandConfig::default(),
        );

        let output = trigger_rescan(&handler, &allowlist()).expect("rescan should succeed");
        assert_eq!(output.resolution.state, DetectionState::Detected);
        assert_eq!(
            output.resolution.metadata.matched_process_id,
            Some(777),
            "resolution should carry scan metadata for UI"
        );
    }

    #[test]
    fn concurrent_rescans_are_debounced_while_in_progress() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let handler = Arc::new(RescanCommandHandler::new(
            BlockingScanner {
                started_tx,
                release_rx: Mutex::new(release_rx),
            },
            FixedClock { now_unix_ms: 45_000 },
            RescanCommandConfig::default(),
        ));

        let allowlist = allowlist();
        let thread_handler = handler.clone();
        let thread_allowlist = allowlist.clone();

        let worker = thread::spawn(move || thread_handler.trigger_rescan(&thread_allowlist));
        started_rx.recv().expect("scanner should start");

        let second = handler.trigger_rescan(&allowlist).expect_err("second rescan must debounce");
        assert_eq!(second.code, RescanCommandErrorCode::ScanAlreadyInProgress);

        release_tx.send(()).expect("release scanner");
        let first = worker.join().expect("worker join").expect("first rescan ok");
        assert_eq!(first.resolution.state, DetectionState::Detected);
    }

    #[test]
    fn scanner_failures_return_actionable_error_state_reason_code() {
        let handler = RescanCommandHandler::new(
            ErrorScanner {
                code: ScanErrorCode::PermissionDenied,
            },
            FixedClock { now_unix_ms: 60_000 },
            RescanCommandConfig::default(),
        );

        let output = handler
            .trigger_rescan(&allowlist())
            .expect("scan failure should still return normalized state");

        assert_eq!(output.resolution.state, DetectionState::Error);
        assert_eq!(
            output.resolution.metadata.reason_code,
            Some(DetectionReasonCode::PermissionDenied)
        );
    }

    #[test]
    fn runtime_state_updates_after_successful_detection() {
        let handler = RescanCommandHandler::new(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 888,
                    executable_name: "ffxiv_dx11.exe".to_string(),
                }],
                calls: Arc::new(AtomicUsize::new(0)),
            },
            FixedClock { now_unix_ms: 90_000 },
            RescanCommandConfig {
                scan_interval_ms: 2_000,
                freshness: FreshnessWindowConfig {
                    stale_after_ms: 10_000,
                },
            },
        );

        let output = handler
            .trigger_rescan(&allowlist())
            .expect("rescan should succeed");

        let runtime = handler
            .snapshot_runtime()
            .expect("runtime state should be readable");
        assert!(!runtime.in_progress);
        assert_eq!(runtime.last_detected_at_unix_ms, Some(90_000));
        assert_eq!(
            runtime.next_scheduled_scan_at_unix_ms,
            Some(output.next_scheduled_scan_at_unix_ms)
        );
    }

    #[test]
    fn runtime_state_keeps_last_detected_when_no_live_match() {
        let handler = RescanCommandHandler::with_runtime_state(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 321,
                    executable_name: "explorer.exe".to_string(),
                }],
                calls: Arc::new(AtomicUsize::new(0)),
            },
            FixedClock { now_unix_ms: 110_000 },
            RescanCommandConfig::default(),
            RescanRuntimeState {
                in_progress: false,
                last_detected_at_unix_ms: Some(100_000),
                next_scheduled_scan_at_unix_ms: Some(103_000),
            },
        );

        let output = handler
            .trigger_rescan(&allowlist())
            .expect("rescan should complete with no live match");
        assert_eq!(output.resolution.state, DetectionState::Detected);

        let runtime = handler
            .snapshot_runtime()
            .expect("runtime state should be readable");
        assert_eq!(runtime.last_detected_at_unix_ms, Some(100_000));
    }

    #[test]
    fn command_metadata_exposes_actionable_failure_for_busy_state() {
        let handler = RescanCommandHandler::with_runtime_state(
            StubScanner {
                processes: Vec::new(),
                calls: Arc::new(AtomicUsize::new(0)),
            },
            FixedClock { now_unix_ms: 123_000 },
            RescanCommandConfig::default(),
            RescanRuntimeState {
                in_progress: true,
                last_detected_at_unix_ms: None,
                next_scheduled_scan_at_unix_ms: None,
            },
        );

        let err = handler
            .trigger_rescan(&allowlist())
            .expect_err("busy state should return actionable error");
        assert_eq!(err.code, RescanCommandErrorCode::ScanAlreadyInProgress);
    }
}
