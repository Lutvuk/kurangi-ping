use super::{MetricsComputationErrorCode, PingMetrics};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricFreshnessConfig {
    pub stale_after_ms: u64,
}

impl Default for MetricFreshnessConfig {
    fn default() -> Self {
        Self {
            stale_after_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricFreshnessInput {
    pub now_unix_ms: u64,
    pub last_sample_at_unix_ms: Option<u64>,
    pub config: MetricFreshnessConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricFreshnessStatus {
    Missing,
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricFreshnessResult {
    pub status: MetricFreshnessStatus,
    pub stale_after_ms: u64,
    pub sample_age_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsState {
    Idle,
    Live,
    Degraded,
    Error,
}

impl MetricsState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Live => "live",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsStateReasonCode {
    NoSamplesYet,
    FreshnessTimeout,
    ProbeFailed,
    InvalidComputationWindow,
    InvalidSampleValues,
}

impl MetricsStateReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoSamplesYet => "no_samples_yet",
            Self::FreshnessTimeout => "freshness_timeout",
            Self::ProbeFailed => "probe_failed",
            Self::InvalidComputationWindow => "invalid_computation_window",
            Self::InvalidSampleValues => "invalid_sample_values",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsStateSnapshot {
    pub state: MetricsState,
    pub reason_code: Option<MetricsStateReasonCode>,
    pub freshness: MetricFreshnessResult,
    pub latest_metrics: Option<PingMetrics>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricsStateResolutionInput {
    pub freshness: MetricFreshnessResult,
    pub latest_metrics: Option<PingMetrics>,
    pub last_probe_failed: bool,
    pub computation_error_code: Option<MetricsComputationErrorCode>,
}

pub fn evaluate_metric_freshness(input: &MetricFreshnessInput) -> MetricFreshnessResult {
    let sample_age_ms = input
        .last_sample_at_unix_ms
        .map(|last| input.now_unix_ms.saturating_sub(last));

    let status = match sample_age_ms {
        None => MetricFreshnessStatus::Missing,
        Some(age_ms) if age_ms > input.config.stale_after_ms => MetricFreshnessStatus::Stale,
        Some(_) => MetricFreshnessStatus::Fresh,
    };

    MetricFreshnessResult {
        status,
        stale_after_ms: input.config.stale_after_ms,
        sample_age_ms,
    }
}

pub fn resolve_metrics_state(input: &MetricsStateResolutionInput) -> MetricsStateSnapshot {
    if let Some(error_code) = input.computation_error_code {
        return MetricsStateSnapshot {
            state: MetricsState::Error,
            reason_code: Some(map_computation_error(error_code)),
            freshness: input.freshness,
            latest_metrics: input.latest_metrics.clone(),
        };
    }

    match input.freshness.status {
        MetricFreshnessStatus::Missing => MetricsStateSnapshot {
            state: MetricsState::Idle,
            reason_code: Some(MetricsStateReasonCode::NoSamplesYet),
            freshness: input.freshness,
            latest_metrics: input.latest_metrics.clone(),
        },
        MetricFreshnessStatus::Fresh => {
            if input.last_probe_failed {
                MetricsStateSnapshot {
                    state: MetricsState::Degraded,
                    reason_code: Some(MetricsStateReasonCode::ProbeFailed),
                    freshness: input.freshness,
                    latest_metrics: input.latest_metrics.clone(),
                }
            } else {
                MetricsStateSnapshot {
                    state: MetricsState::Live,
                    reason_code: None,
                    freshness: input.freshness,
                    latest_metrics: input.latest_metrics.clone(),
                }
            }
        }
        MetricFreshnessStatus::Stale => MetricsStateSnapshot {
            state: MetricsState::Degraded,
            reason_code: Some(MetricsStateReasonCode::FreshnessTimeout),
            freshness: input.freshness,
            latest_metrics: input.latest_metrics.clone(),
        },
    }
}

fn map_computation_error(error_code: MetricsComputationErrorCode) -> MetricsStateReasonCode {
    match error_code {
        MetricsComputationErrorCode::EmptyWindow => MetricsStateReasonCode::NoSamplesYet,
        MetricsComputationErrorCode::MissingBaselineSamples
        | MetricsComputationErrorCode::InvalidObservationWindow => {
            MetricsStateReasonCode::InvalidComputationWindow
        }
        MetricsComputationErrorCode::InvalidLatencySample => {
            MetricsStateReasonCode::InvalidSampleValues
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_metric_freshness, resolve_metrics_state, MetricFreshnessConfig,
        MetricFreshnessInput, MetricFreshnessResult, MetricFreshnessStatus, MetricsState,
        MetricsStateReasonCode, MetricsStateResolutionInput,
    };
    use crate::metrics::{MetricsComputationErrorCode, PingMetrics};

    fn sample_metrics() -> PingMetrics {
        PingMetrics {
            baseline_ping_ms: 208.0,
            routed_ping_ms: Some(154.0),
            jitter_ms: Some(4.0),
            packet_loss_pct: Some(0.0),
            sample_count: 5,
        }
    }

    #[test]
    fn freshness_timeout_is_configurable() {
        let now: u64 = 1_700_000_000_000;
        let recent_sample = Some(now.saturating_sub(2_000));

        let strict = evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: now,
            last_sample_at_unix_ms: recent_sample,
            config: MetricFreshnessConfig { stale_after_ms: 1_000 },
        });
        assert_eq!(strict.status, MetricFreshnessStatus::Stale);

        let relaxed = evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: now,
            last_sample_at_unix_ms: recent_sample,
            config: MetricFreshnessConfig { stale_after_ms: 3_000 },
        });
        assert_eq!(relaxed.status, MetricFreshnessStatus::Fresh);
    }

    #[test]
    fn stale_metrics_transition_to_degraded_deterministically() {
        let now: u64 = 1_700_000_000_000;
        let at_boundary = evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: now,
            last_sample_at_unix_ms: Some(now.saturating_sub(5_000)),
            config: MetricFreshnessConfig { stale_after_ms: 5_000 },
        });
        assert_eq!(at_boundary.status, MetricFreshnessStatus::Fresh);

        let stale = evaluate_metric_freshness(&MetricFreshnessInput {
            now_unix_ms: now,
            last_sample_at_unix_ms: Some(now.saturating_sub(5_001)),
            config: MetricFreshnessConfig { stale_after_ms: 5_000 },
        });
        assert_eq!(stale.status, MetricFreshnessStatus::Stale);

        let resolved = resolve_metrics_state(&MetricsStateResolutionInput {
            freshness: stale,
            latest_metrics: Some(sample_metrics()),
            last_probe_failed: false,
            computation_error_code: None,
        });
        assert_eq!(resolved.state, MetricsState::Degraded);
        assert_eq!(
            resolved.reason_code,
            Some(MetricsStateReasonCode::FreshnessTimeout)
        );
    }

    #[test]
    fn last_known_values_remain_available_for_ui_rendering() {
        let stale_freshness = MetricFreshnessResult {
            status: MetricFreshnessStatus::Stale,
            stale_after_ms: 5_000,
            sample_age_ms: Some(11_000),
        };
        let last_known = sample_metrics();

        let snapshot = resolve_metrics_state(&MetricsStateResolutionInput {
            freshness: stale_freshness,
            latest_metrics: Some(last_known.clone()),
            last_probe_failed: true,
            computation_error_code: None,
        });

        assert_eq!(snapshot.state, MetricsState::Degraded);
        assert_eq!(snapshot.latest_metrics, Some(last_known));
    }

    #[test]
    fn error_states_include_reason_codes_for_diagnostics() {
        let freshness = MetricFreshnessResult {
            status: MetricFreshnessStatus::Fresh,
            stale_after_ms: 5_000,
            sample_age_ms: Some(200),
        };

        let invalid_window = resolve_metrics_state(&MetricsStateResolutionInput {
            freshness,
            latest_metrics: None,
            last_probe_failed: false,
            computation_error_code: Some(MetricsComputationErrorCode::InvalidObservationWindow),
        });
        assert_eq!(invalid_window.state, MetricsState::Error);
        assert_eq!(
            invalid_window.reason_code,
            Some(MetricsStateReasonCode::InvalidComputationWindow)
        );

        let invalid_sample = resolve_metrics_state(&MetricsStateResolutionInput {
            freshness,
            latest_metrics: None,
            last_probe_failed: false,
            computation_error_code: Some(MetricsComputationErrorCode::InvalidLatencySample),
        });
        assert_eq!(invalid_sample.state, MetricsState::Error);
        assert_eq!(
            invalid_sample.reason_code,
            Some(MetricsStateReasonCode::InvalidSampleValues)
        );
    }
}
