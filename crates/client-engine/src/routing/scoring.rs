//! Deterministic relay candidate scoring.

use super::RelayHealthStatus;
use super::RouteCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayHealthSnapshot {
    pub relay_id: String,
    pub status: RelayHealthStatus,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringWeights {
    pub health_points_ok: i32,
    pub health_points_warn: i32,
    pub health_points_dead: i32,
    pub preferred_region_bonus: i32,
    pub manifest_priority_weight: i32,
    pub latency_penalty_per_ms: i32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            health_points_ok: 120,
            health_points_warn: 50,
            health_points_dead: -500,
            preferred_region_bonus: 40,
            manifest_priority_weight: 5,
            latency_penalty_per_ms: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayScoringConfig {
    pub preferred_region: Option<String>,
    pub weights: ScoringWeights,
    pub exclude_dead_relays: bool,
    pub exclude_latency_over_ms: Option<u32>,
}

impl Default for RelayScoringConfig {
    fn default() -> Self {
        Self {
            preferred_region: None,
            weights: ScoringWeights::default(),
            exclude_dead_relays: true,
            exclude_latency_over_ms: Some(250),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateDisposition {
    Included,
    Excluded(ScoreExclusionReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreExclusionReason {
    DeadRelay,
    LatencyAboveThreshold,
    MissingHealthSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreBreakdown {
    pub health_points: i32,
    pub region_bonus_points: i32,
    pub priority_points: i32,
    pub latency_penalty_points: i32,
    pub total_points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScore {
    pub candidate: RouteCandidate,
    pub health_status: Option<RelayHealthStatus>,
    pub latency_ms: Option<u32>,
    pub disposition: CandidateDisposition,
    pub breakdown: ScoreBreakdown,
}

impl CandidateScore {
    pub fn is_eligible(&self) -> bool {
        matches!(self.disposition, CandidateDisposition::Included)
    }
}

pub fn score_candidates(
    candidates: &[RouteCandidate],
    health_snapshots: &[RelayHealthSnapshot],
    config: &RelayScoringConfig,
) -> Vec<CandidateScore> {
    let mut scored = candidates
        .iter()
        .map(|candidate| score_single_candidate(candidate, health_snapshots, config))
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| {
        right
            .is_eligible()
            .cmp(&left.is_eligible())
            .then_with(|| {
                right
                    .breakdown
                    .total_points
                    .cmp(&left.breakdown.total_points)
            })
            .then_with(|| left.candidate.priority.cmp(&right.candidate.priority))
            .then_with(|| left.candidate.relay_id.cmp(&right.candidate.relay_id))
    });

    scored
}

fn score_single_candidate(
    candidate: &RouteCandidate,
    health_snapshots: &[RelayHealthSnapshot],
    config: &RelayScoringConfig,
) -> CandidateScore {
    let health = health_snapshots
        .iter()
        .find(|snapshot| snapshot.relay_id == candidate.relay_id);

    let preferred_region = config
        .preferred_region
        .as_ref()
        .map(|region| region.eq_ignore_ascii_case(&candidate.region))
        .unwrap_or(false);

    let priority_points = config.weights.manifest_priority_weight * -(candidate.priority as i32);
    let region_bonus_points = if preferred_region {
        config.weights.preferred_region_bonus
    } else {
        0
    };

    let (health_points, latency_ms, latency_penalty_points, disposition) = match health {
        Some(snapshot) => {
            let health_points = match snapshot.status {
                RelayHealthStatus::Ok => config.weights.health_points_ok,
                RelayHealthStatus::Warn => config.weights.health_points_warn,
                RelayHealthStatus::Dead => config.weights.health_points_dead,
            };
            let latency_penalty_points =
                config.weights.latency_penalty_per_ms * (snapshot.latency_ms as i32);

            let dead_excluded =
                config.exclude_dead_relays && snapshot.status == RelayHealthStatus::Dead;
            let latency_excluded = config
                .exclude_latency_over_ms
                .map(|threshold| snapshot.latency_ms > threshold)
                .unwrap_or(false);
            let disposition = if dead_excluded {
                CandidateDisposition::Excluded(ScoreExclusionReason::DeadRelay)
            } else if latency_excluded {
                CandidateDisposition::Excluded(ScoreExclusionReason::LatencyAboveThreshold)
            } else {
                CandidateDisposition::Included
            };

            (
                health_points,
                Some(snapshot.latency_ms),
                latency_penalty_points,
                disposition,
            )
        }
        None => (
            -200,
            None,
            0,
            CandidateDisposition::Excluded(ScoreExclusionReason::MissingHealthSnapshot),
        ),
    };

    let total_points =
        health_points + region_bonus_points + priority_points - latency_penalty_points;
    CandidateScore {
        candidate: candidate.clone(),
        health_status: health.map(|snapshot| snapshot.status),
        latency_ms,
        disposition,
        breakdown: ScoreBreakdown {
            health_points,
            region_bonus_points,
            priority_points,
            latency_penalty_points,
            total_points,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        score_candidates, CandidateDisposition, RelayHealthSnapshot, RelayHealthStatus,
        RelayScoringConfig, ScoreExclusionReason,
    };
    use crate::routing::RouteCandidate;

    fn candidate(relay_id: &str, region: &str, priority: u16) -> RouteCandidate {
        RouteCandidate {
            relay_id: relay_id.to_string(),
            region: region.to_string(),
            hostname: format!("{relay_id}.example.net"),
            priority,
        }
    }

    fn health(relay_id: &str, status: RelayHealthStatus, latency_ms: u32) -> RelayHealthSnapshot {
        RelayHealthSnapshot {
            relay_id: relay_id.to_string(),
            status,
            latency_ms,
        }
    }

    #[test]
    fn stable_ordering_for_identical_inputs() {
        let candidates = vec![
            candidate("sin-01", "sin", 1),
            candidate("nrt-01", "nrt", 1),
            candidate("lax-01", "lax", 1),
        ];
        let healths = vec![
            health("sin-01", RelayHealthStatus::Ok, 50),
            health("nrt-01", RelayHealthStatus::Ok, 50),
            health("lax-01", RelayHealthStatus::Ok, 50),
        ];
        let config = RelayScoringConfig::default();

        let first = score_candidates(&candidates, &healths, &config);
        let second = score_candidates(&candidates, &healths, &config);

        let first_ids = first
            .iter()
            .map(|entry| entry.candidate.relay_id.clone())
            .collect::<Vec<_>>();
        let second_ids = second
            .iter()
            .map(|entry| entry.candidate.relay_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(first_ids, second_ids);
        assert_eq!(first_ids, vec!["lax-01", "nrt-01", "sin-01"]);
    }

    #[test]
    fn region_preference_affects_score() {
        let candidates = vec![candidate("sin-01", "sin", 2), candidate("nrt-01", "nrt", 1)];
        let healths = vec![
            health("sin-01", RelayHealthStatus::Ok, 60),
            health("nrt-01", RelayHealthStatus::Ok, 60),
        ];

        let config = RelayScoringConfig {
            preferred_region: Some("sin".to_string()),
            ..RelayScoringConfig::default()
        };

        let scored = score_candidates(&candidates, &healths, &config);
        assert_eq!(scored[0].candidate.relay_id, "sin-01");
        assert!(scored[0].breakdown.region_bonus_points > scored[1].breakdown.region_bonus_points);
    }

    #[test]
    fn unhealthy_relays_can_be_excluded_by_threshold_policy() {
        let candidates = vec![candidate("sin-01", "sin", 1), candidate("nrt-01", "nrt", 2)];
        let healths = vec![
            health("sin-01", RelayHealthStatus::Dead, 30),
            health("nrt-01", RelayHealthStatus::Warn, 400),
        ];
        let config = RelayScoringConfig::default();

        let scored = score_candidates(&candidates, &healths, &config);
        let sin = scored
            .iter()
            .find(|entry| entry.candidate.relay_id == "sin-01")
            .expect("sin candidate must exist");
        let nrt = scored
            .iter()
            .find(|entry| entry.candidate.relay_id == "nrt-01")
            .expect("nrt candidate must exist");

        assert_eq!(
            sin.disposition,
            CandidateDisposition::Excluded(ScoreExclusionReason::DeadRelay)
        );
        assert_eq!(
            nrt.disposition,
            CandidateDisposition::Excluded(ScoreExclusionReason::LatencyAboveThreshold)
        );
    }

    #[test]
    fn score_output_is_inspectable_for_debugging() {
        let candidates = vec![candidate("sin-01", "sin", 1)];
        let healths = vec![health("sin-01", RelayHealthStatus::Ok, 42)];
        let config = RelayScoringConfig {
            preferred_region: Some("sin".to_string()),
            ..RelayScoringConfig::default()
        };

        let scored = score_candidates(&candidates, &healths, &config);
        let entry = &scored[0];

        assert_eq!(entry.health_status, Some(RelayHealthStatus::Ok));
        assert_eq!(entry.latency_ms, Some(42));
        assert_eq!(entry.disposition, CandidateDisposition::Included);
        assert!(entry.breakdown.health_points > 0);
        assert!(entry.breakdown.total_points > 0);
        assert!(entry.is_eligible());
    }
}
