//! Detection state resolver with configurable freshness window.

use super::scanner_windows::{DetectionMatch, ScanErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionState {
    NotFound,
    Detected,
    Stale,
    Error,
}

impl DetectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectionState::NotFound => "not_found",
            DetectionState::Detected => "detected",
            DetectionState::Stale => "stale",
            DetectionState::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionReasonCode {
    PermissionDenied,
    SnapshotUnavailable,
    EnumerationFailed,
    UnsupportedPlatform,
    StaleWindowExceeded,
}

impl DetectionReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectionReasonCode::PermissionDenied => "permission_denied",
            DetectionReasonCode::SnapshotUnavailable => "snapshot_unavailable",
            DetectionReasonCode::EnumerationFailed => "enumeration_failed",
            DetectionReasonCode::UnsupportedPlatform => "unsupported_platform",
            DetectionReasonCode::StaleWindowExceeded => "stale_window_exceeded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreshnessWindowConfig {
    pub stale_after_ms: u64,
}

impl Default for FreshnessWindowConfig {
    fn default() -> Self {
        Self {
            stale_after_ms: 15_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResolverInput {
    pub matches: Vec<DetectionMatch>,
    pub scan_error: Option<ScanErrorCode>,
    pub scanned_at_unix_ms: u64,
    pub last_detected_at_unix_ms: Option<u64>,
    pub freshness: FreshnessWindowConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionMetadata {
    pub scanned_at_unix_ms: u64,
    pub last_detected_at_unix_ms: Option<u64>,
    pub stale_after_ms: u64,
    pub stale_age_ms: Option<u64>,
    pub reason_code: Option<DetectionReasonCode>,
    pub match_count: usize,
    pub matched_process_id: Option<u32>,
    pub matched_game_id: Option<String>,
    pub matched_executable_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResolution {
    pub state: DetectionState,
    pub metadata: DetectionMetadata,
}

pub fn is_stale(now_unix_ms: u64, last_detected_at_unix_ms: u64, stale_after_ms: u64) -> bool {
    now_unix_ms.saturating_sub(last_detected_at_unix_ms) > stale_after_ms
}

pub fn resolve_detection_state(input: &DetectionResolverInput) -> DetectionResolution {
    if let Some(scan_error) = input.scan_error {
        return DetectionResolution {
            state: DetectionState::Error,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: input.scanned_at_unix_ms,
                last_detected_at_unix_ms: input.last_detected_at_unix_ms,
                stale_after_ms: input.freshness.stale_after_ms,
                stale_age_ms: input
                    .last_detected_at_unix_ms
                    .map(|last| input.scanned_at_unix_ms.saturating_sub(last)),
                reason_code: Some(map_scan_error_code(scan_error)),
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        };
    }

    if let Some(selected_match) = select_primary_match(&input.matches) {
        return DetectionResolution {
            state: DetectionState::Detected,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: input.scanned_at_unix_ms,
                last_detected_at_unix_ms: Some(input.scanned_at_unix_ms),
                stale_after_ms: input.freshness.stale_after_ms,
                stale_age_ms: Some(0),
                reason_code: None,
                match_count: input.matches.len(),
                matched_process_id: Some(selected_match.process_id),
                matched_game_id: Some(selected_match.game_id.clone()),
                matched_executable_name: Some(selected_match.executable_name.clone()),
            },
        };
    }

    if let Some(last_detected_at) = input.last_detected_at_unix_ms {
        let age = input.scanned_at_unix_ms.saturating_sub(last_detected_at);
        if is_stale(
            input.scanned_at_unix_ms,
            last_detected_at,
            input.freshness.stale_after_ms,
        ) {
            return DetectionResolution {
                state: DetectionState::Stale,
                metadata: DetectionMetadata {
                    scanned_at_unix_ms: input.scanned_at_unix_ms,
                    last_detected_at_unix_ms: Some(last_detected_at),
                    stale_after_ms: input.freshness.stale_after_ms,
                    stale_age_ms: Some(age),
                    reason_code: Some(DetectionReasonCode::StaleWindowExceeded),
                    match_count: 0,
                    matched_process_id: None,
                    matched_game_id: None,
                    matched_executable_name: None,
                },
            };
        }

        return DetectionResolution {
            state: DetectionState::Detected,
            metadata: DetectionMetadata {
                scanned_at_unix_ms: input.scanned_at_unix_ms,
                last_detected_at_unix_ms: Some(last_detected_at),
                stale_after_ms: input.freshness.stale_after_ms,
                stale_age_ms: Some(age),
                reason_code: None,
                match_count: 0,
                matched_process_id: None,
                matched_game_id: None,
                matched_executable_name: None,
            },
        };
    }

    DetectionResolution {
        state: DetectionState::NotFound,
        metadata: DetectionMetadata {
            scanned_at_unix_ms: input.scanned_at_unix_ms,
            last_detected_at_unix_ms: None,
            stale_after_ms: input.freshness.stale_after_ms,
            stale_age_ms: None,
            reason_code: None,
            match_count: 0,
            matched_process_id: None,
            matched_game_id: None,
            matched_executable_name: None,
        },
    }
}

fn select_primary_match(matches: &[DetectionMatch]) -> Option<&DetectionMatch> {
    matches.iter().min_by(|left, right| {
        left.process_id
            .cmp(&right.process_id)
            .then_with(|| left.game_id.cmp(&right.game_id))
            .then_with(|| left.executable_name.cmp(&right.executable_name))
    })
}

fn map_scan_error_code(scan_error_code: ScanErrorCode) -> DetectionReasonCode {
    match scan_error_code {
        ScanErrorCode::PermissionDenied => DetectionReasonCode::PermissionDenied,
        ScanErrorCode::SnapshotUnavailable => DetectionReasonCode::SnapshotUnavailable,
        ScanErrorCode::EnumerationFailed => DetectionReasonCode::EnumerationFailed,
        ScanErrorCode::UnsupportedPlatform => DetectionReasonCode::UnsupportedPlatform,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_stale, resolve_detection_state, DetectionReasonCode, DetectionResolverInput,
        DetectionState, FreshnessWindowConfig,
    };
    use crate::detection::scanner_windows::{DetectionMatch, ScanErrorCode};

    fn default_input() -> DetectionResolverInput {
        DetectionResolverInput {
            matches: Vec::new(),
            scan_error: None,
            scanned_at_unix_ms: 20_000,
            last_detected_at_unix_ms: None,
            freshness: FreshnessWindowConfig {
                stale_after_ms: 5_000,
            },
        }
    }

    #[test]
    fn is_stale_is_deterministic_with_threshold_boundary() {
        assert!(!is_stale(10_000, 5_000, 5_000));
        assert!(is_stale(10_001, 5_000, 5_000));
        assert!(!is_stale(4_000, 5_000, 5_000));
    }

    #[test]
    fn resolve_maps_to_detected_when_match_exists() {
        let mut input = default_input();
        input.matches = vec![DetectionMatch {
            process_id: 222,
            game_id: "ffxiv".to_string(),
            executable_name: "ffxiv_dx11.exe".to_string(),
        }];

        let resolution = resolve_detection_state(&input);
        assert_eq!(resolution.state, DetectionState::Detected);
        assert_eq!(resolution.metadata.match_count, 1);
        assert_eq!(resolution.metadata.matched_process_id, Some(222));
        assert_eq!(
            resolution.metadata.matched_game_id.as_deref(),
            Some("ffxiv")
        );
        assert_eq!(resolution.metadata.reason_code, None);
    }

    #[test]
    fn resolve_detected_uses_deterministic_primary_match() {
        let mut input = default_input();
        input.matches = vec![
            DetectionMatch {
                process_id: 501,
                game_id: "valorant".to_string(),
                executable_name: "valorant-win64-shipping.exe".to_string(),
            },
            DetectionMatch {
                process_id: 222,
                game_id: "ffxiv".to_string(),
                executable_name: "ffxiv_dx11.exe".to_string(),
            },
        ];

        let resolution = resolve_detection_state(&input);
        assert_eq!(resolution.state, DetectionState::Detected);
        assert_eq!(resolution.metadata.matched_process_id, Some(222));
        assert_eq!(
            resolution.metadata.matched_game_id.as_deref(),
            Some("ffxiv")
        );
    }

    #[test]
    fn resolve_maps_to_detected_when_recent_history_is_within_freshness() {
        let mut input = default_input();
        input.last_detected_at_unix_ms = Some(16_000);

        let resolution = resolve_detection_state(&input);
        assert_eq!(resolution.state, DetectionState::Detected);
        assert_eq!(resolution.metadata.stale_age_ms, Some(4_000));
    }

    #[test]
    fn resolve_maps_to_stale_when_freshness_window_exceeded() {
        let mut input = default_input();
        input.last_detected_at_unix_ms = Some(10_000);

        let resolution = resolve_detection_state(&input);
        assert_eq!(resolution.state, DetectionState::Stale);
        assert_eq!(
            resolution.metadata.reason_code,
            Some(DetectionReasonCode::StaleWindowExceeded)
        );
        assert_eq!(resolution.metadata.stale_age_ms, Some(10_000));
    }

    #[test]
    fn resolve_maps_to_not_found_without_match_or_history() {
        let input = default_input();
        let resolution = resolve_detection_state(&input);

        assert_eq!(resolution.state, DetectionState::NotFound);
        assert_eq!(resolution.metadata.reason_code, None);
        assert_eq!(resolution.metadata.match_count, 0);
    }

    #[test]
    fn resolve_maps_to_error_with_non_sensitive_reason_code() {
        let mut input = default_input();
        input.scan_error = Some(ScanErrorCode::PermissionDenied);
        input.last_detected_at_unix_ms = Some(19_000);

        let resolution = resolve_detection_state(&input);
        assert_eq!(resolution.state, DetectionState::Error);
        assert_eq!(
            resolution.metadata.reason_code,
            Some(DetectionReasonCode::PermissionDenied)
        );
        assert_eq!(resolution.metadata.stale_age_ms, Some(1_000));
        assert_eq!(resolution.metadata.matched_game_id, None);
    }
}
