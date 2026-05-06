#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayHealthStatus {
    Ok,
    Warn,
    Dead,
}

impl RelayHealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelayHealthStatus::Ok => "ok",
            RelayHealthStatus::Warn => "warn",
            RelayHealthStatus::Dead => "dead",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayHealthMetrics {
    pub latency_ms: Option<u32>,
    pub consecutive_timeout_count: u32,
    pub last_updated_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthClassificationThresholds {
    pub ok_max_latency_ms: u32,
    pub warn_max_latency_ms: u32,
    pub max_timeout_streak: u32,
    pub max_sample_age_ms: u64,
    pub unknown_fallback_status: RelayHealthStatus,
}

impl Default for HealthClassificationThresholds {
    fn default() -> Self {
        Self {
            ok_max_latency_ms: 50,
            warn_max_latency_ms: 150,
            max_timeout_streak: 1,
            max_sample_age_ms: 10_000,
            unknown_fallback_status: RelayHealthStatus::Dead,
        }
    }
}

pub fn classify_relay_health(
    metrics: &RelayHealthMetrics,
    thresholds: &HealthClassificationThresholds,
    now_unix_ms: u64,
) -> RelayHealthStatus {
    let stale = match metrics.last_updated_unix_ms {
        Some(last_updated_unix_ms) => {
            now_unix_ms.saturating_sub(last_updated_unix_ms) > thresholds.max_sample_age_ms
        }
        None => true,
    };

    if stale {
        return thresholds.unknown_fallback_status;
    }

    if metrics.consecutive_timeout_count > thresholds.max_timeout_streak {
        return RelayHealthStatus::Dead;
    }

    let latency_ms = match metrics.latency_ms {
        Some(value) => value,
        None => return thresholds.unknown_fallback_status,
    };

    if latency_ms <= thresholds.ok_max_latency_ms {
        RelayHealthStatus::Ok
    } else if latency_ms <= thresholds.warn_max_latency_ms {
        RelayHealthStatus::Warn
    } else {
        RelayHealthStatus::Dead
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_relay_health, HealthClassificationThresholds, RelayHealthMetrics,
        RelayHealthStatus,
    };

    fn thresholds() -> HealthClassificationThresholds {
        HealthClassificationThresholds::default()
    }

    fn metrics(
        latency_ms: Option<u32>,
        timeout_count: u32,
        last_updated_unix_ms: Option<u64>,
    ) -> RelayHealthMetrics {
        RelayHealthMetrics {
            latency_ms,
            consecutive_timeout_count: timeout_count,
            last_updated_unix_ms,
        }
    }

    #[test]
    fn classification_outputs_only_ok_warn_dead() {
        let now = 100_000;
        let config = thresholds();
        let outputs = [
            classify_relay_health(&metrics(Some(40), 0, Some(now)), &config, now),
            classify_relay_health(&metrics(Some(120), 0, Some(now)), &config, now),
            classify_relay_health(&metrics(Some(220), 0, Some(now)), &config, now),
            classify_relay_health(&metrics(Some(40), 5, Some(now)), &config, now),
        ];

        for status in outputs {
            assert!(matches!(
                status,
                RelayHealthStatus::Ok | RelayHealthStatus::Warn | RelayHealthStatus::Dead
            ));
        }
    }

    #[test]
    fn threshold_configuration_is_centralized_and_typed() {
        let custom = HealthClassificationThresholds {
            ok_max_latency_ms: 30,
            warn_max_latency_ms: 90,
            max_timeout_streak: 2,
            max_sample_age_ms: 3_000,
            unknown_fallback_status: RelayHealthStatus::Warn,
        };

        let now = 10_000;
        let ok = classify_relay_health(&metrics(Some(20), 0, Some(now)), &custom, now);
        let warn = classify_relay_health(&metrics(Some(80), 0, Some(now)), &custom, now);
        let dead = classify_relay_health(&metrics(Some(120), 0, Some(now)), &custom, now);

        assert_eq!(ok, RelayHealthStatus::Ok);
        assert_eq!(warn, RelayHealthStatus::Warn);
        assert_eq!(dead, RelayHealthStatus::Dead);
    }

    #[test]
    fn missing_or_unknown_metrics_use_deterministic_fallback_status() {
        let custom = HealthClassificationThresholds {
            unknown_fallback_status: RelayHealthStatus::Warn,
            ..thresholds()
        };
        let now = 25_000;

        let missing_latency = classify_relay_health(&metrics(None, 0, Some(now)), &custom, now);
        let missing_timestamp = classify_relay_health(&metrics(Some(42), 0, None), &custom, now);
        let stale = classify_relay_health(&metrics(Some(42), 0, Some(now - 11_000)), &custom, now);

        assert_eq!(missing_latency, RelayHealthStatus::Warn);
        assert_eq!(missing_timestamp, RelayHealthStatus::Warn);
        assert_eq!(stale, RelayHealthStatus::Warn);
    }

    #[test]
    fn threshold_boundaries_are_deterministic() {
        let now = 40_000;
        let config = thresholds();

        let at_ok_boundary = classify_relay_health(&metrics(Some(50), 0, Some(now)), &config, now);
        let at_warn_boundary =
            classify_relay_health(&metrics(Some(150), 0, Some(now)), &config, now);
        let above_warn_boundary =
            classify_relay_health(&metrics(Some(151), 0, Some(now)), &config, now);
        let timeout_streak_boundary =
            classify_relay_health(&metrics(Some(40), 1, Some(now)), &config, now);
        let timeout_streak_above =
            classify_relay_health(&metrics(Some(40), 2, Some(now)), &config, now);

        assert_eq!(at_ok_boundary, RelayHealthStatus::Ok);
        assert_eq!(at_warn_boundary, RelayHealthStatus::Warn);
        assert_eq!(above_warn_boundary, RelayHealthStatus::Dead);
        assert_eq!(timeout_streak_boundary, RelayHealthStatus::Ok);
        assert_eq!(timeout_streak_above, RelayHealthStatus::Dead);
    }
}
