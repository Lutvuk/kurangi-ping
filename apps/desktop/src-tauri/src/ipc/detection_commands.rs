use client_engine::detection::commands::{
    trigger_rescan, DefaultRescanHandler, RescanCommandConfig, RescanCommandErrorCode,
    RescanCommandHandler, SystemTimeProvider, TimeProvider,
};
use client_engine::detection::scanner_windows::{
    ProcessEnumerator, SupportedGame, WindowsProcessEnumerator,
};
use client_engine::detection::state_resolver::{DetectionResolution, DetectionState};

use super::contracts::{DetectionStatusResponse, DetectionStatusState};

#[tauri::command]
pub fn detection_get_status() -> Result<DetectionStatusResponse, String> {
    let handler = DefaultRescanHandler::new(
        WindowsProcessEnumerator,
        SystemTimeProvider,
        RescanCommandConfig::default(),
    );

    Ok(query_detection_status(&handler, &supported_games()))
}

fn supported_games() -> Vec<SupportedGame> {
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
    ]
}

fn query_detection_status<E: ProcessEnumerator, T: TimeProvider>(
    handler: &RescanCommandHandler<E, T>,
    allowlist: &[SupportedGame],
) -> DetectionStatusResponse {
    match trigger_rescan(handler, allowlist) {
        Ok(output) => map_detection_resolution(output.resolution),
        Err(error) => map_rescan_error(error.code),
    }
}

fn map_detection_resolution(resolution: DetectionResolution) -> DetectionStatusResponse {
    match resolution.state {
        DetectionState::Detected => DetectionStatusResponse {
            state: DetectionStatusState::Detected,
            game_id: resolution.metadata.matched_game_id,
            process_name: resolution.metadata.matched_executable_name,
            detection_time_ms: Some(resolution.metadata.scanned_at_unix_ms),
            reason_code: None,
            message: None,
        },
        DetectionState::NotFound | DetectionState::Stale => DetectionStatusResponse {
            state: DetectionStatusState::NotDetected,
            game_id: None,
            process_name: None,
            detection_time_ms: Some(resolution.metadata.scanned_at_unix_ms),
            reason_code: None,
            message: None,
        },
        DetectionState::Error => DetectionStatusResponse {
            state: DetectionStatusState::NotDetected,
            game_id: None,
            process_name: None,
            detection_time_ms: Some(resolution.metadata.scanned_at_unix_ms),
            reason_code: Some("ipc_detection_scan_failed".to_string()),
            message: Some("game detection scan failed".to_string()),
        },
    }
}

fn map_rescan_error(error_code: RescanCommandErrorCode) -> DetectionStatusResponse {
    let reason = match error_code {
        RescanCommandErrorCode::ScanAlreadyInProgress => "ipc_command_rejected",
        RescanCommandErrorCode::InternalStateUnavailable => "ipc_unknown_failure",
    };

    DetectionStatusResponse {
        state: DetectionStatusState::NotDetected,
        game_id: None,
        process_name: None,
        detection_time_ms: None,
        reason_code: Some(reason.to_string()),
        message: Some("detection status is temporarily unavailable".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{map_detection_resolution, map_rescan_error, query_detection_status};
    use client_engine::detection::commands::{
        RescanCommandConfig, RescanCommandHandler, RescanRuntimeState, TimeProvider,
    };
    use client_engine::detection::scanner_windows::{
        ProcessEntry, ProcessEnumerator, ScanError, ScanErrorCode, SupportedGame,
    };
    use client_engine::detection::state_resolver::{
        DetectionReasonCode, DetectionResolution, DetectionState, FreshnessWindowConfig,
    };

    #[derive(Clone, Copy)]
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
    }

    impl ProcessEnumerator for StubScanner {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
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

    fn allowlist() -> Vec<SupportedGame> {
        vec![SupportedGame {
            game_id: "ffxiv".to_string(),
            executable_name: "ffxiv_dx11.exe".to_string(),
            enabled: true,
        }]
    }

    #[test]
    fn detected_scan_is_mapped_to_detected_response() {
        let handler = RescanCommandHandler::new(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 777,
                    executable_name: "ffxiv_dx11.exe".to_string(),
                }],
            },
            FixedClock {
                now_unix_ms: 170_000,
            },
            RescanCommandConfig::default(),
        );

        let response = query_detection_status(&handler, &allowlist());
        assert_eq!(response.state, super::DetectionStatusState::Detected);
        assert_eq!(response.game_id.as_deref(), Some("ffxiv"));
        assert_eq!(response.process_name.as_deref(), Some("ffxiv_dx11.exe"));
        assert_eq!(response.reason_code, None);
    }

    #[test]
    fn not_found_scan_is_mapped_to_not_detected_response() {
        let handler = RescanCommandHandler::new(
            StubScanner {
                processes: vec![ProcessEntry {
                    process_id: 111,
                    executable_name: "explorer.exe".to_string(),
                }],
            },
            FixedClock {
                now_unix_ms: 180_000,
            },
            RescanCommandConfig::default(),
        );

        let response = query_detection_status(&handler, &allowlist());
        assert_eq!(response.state, super::DetectionStatusState::NotDetected);
        assert_eq!(response.game_id, None);
        assert_eq!(response.reason_code, None);
    }

    #[test]
    fn stale_resolution_is_mapped_to_not_detected_response() {
        let stale_resolution = DetectionResolution {
            state: DetectionState::Stale,
            metadata: client_engine::detection::state_resolver::DetectionMetadata {
                scanned_at_unix_ms: 200_000,
                last_detected_at_unix_ms: Some(150_000),
                stale_after_ms: 15_000,
                stale_age_ms: Some(50_000),
                reason_code: Some(DetectionReasonCode::StaleWindowExceeded),
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        };

        let response = map_detection_resolution(stale_resolution);
        assert_eq!(response.state, super::DetectionStatusState::NotDetected);
        assert_eq!(response.reason_code, None);
    }

    #[test]
    fn scanner_failures_are_mapped_to_ui_safe_reason_code() {
        let handler = RescanCommandHandler::new(
            ErrorScanner {
                code: ScanErrorCode::PermissionDenied,
            },
            FixedClock {
                now_unix_ms: 190_000,
            },
            RescanCommandConfig::default(),
        );

        let response = query_detection_status(&handler, &allowlist());
        assert_eq!(response.state, super::DetectionStatusState::NotDetected);
        assert_eq!(
            response.reason_code.as_deref(),
            Some("ipc_detection_scan_failed")
        );
        assert!(response
            .message
            .as_deref()
            .is_some_and(|value| !value.contains("Win32 error")));
    }

    #[test]
    fn busy_runtime_maps_to_command_rejected_reason() {
        let handler = RescanCommandHandler::with_runtime_state(
            StubScanner { processes: vec![] },
            FixedClock {
                now_unix_ms: 210_000,
            },
            RescanCommandConfig {
                scan_interval_ms: 3_000,
                freshness: FreshnessWindowConfig::default(),
            },
            RescanRuntimeState {
                in_progress: true,
                last_detected_at_unix_ms: None,
                next_scheduled_scan_at_unix_ms: None,
            },
        );

        let response = query_detection_status(&handler, &allowlist());
        assert_eq!(response.state, super::DetectionStatusState::NotDetected);
        assert_eq!(
            response.reason_code.as_deref(),
            Some("ipc_command_rejected")
        );
    }

    #[test]
    fn rescan_error_mapping_is_deterministic() {
        let rejected = map_rescan_error(
            client_engine::detection::commands::RescanCommandErrorCode::ScanAlreadyInProgress,
        );
        let unavailable = map_rescan_error(
            client_engine::detection::commands::RescanCommandErrorCode::InternalStateUnavailable,
        );

        assert_eq!(
            rejected.reason_code.as_deref(),
            Some("ipc_command_rejected")
        );
        assert_eq!(
            unavailable.reason_code.as_deref(),
            Some("ipc_unknown_failure")
        );
    }
}
