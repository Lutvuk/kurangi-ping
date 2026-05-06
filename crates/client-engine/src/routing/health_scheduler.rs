use super::RelayHealthSnapshot;

pub const RELAY_HEALTH_ENDPOINT_PATH: &str = "/v1/relay/health";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthPollingConfig {
    pub interval_ms: u64,
    pub min_interval_ms: u64,
    pub max_interval_ms: u64,
}

impl HealthPollingConfig {
    pub fn bounded_interval_ms(&self) -> u64 {
        let (min_bound, max_bound) = if self.min_interval_ms <= self.max_interval_ms {
            (self.min_interval_ms, self.max_interval_ms)
        } else {
            (self.max_interval_ms, self.min_interval_ms)
        };

        self.interval_ms.clamp(min_bound, max_bound)
    }
}

impl Default for HealthPollingConfig {
    fn default() -> Self {
        Self {
            interval_ms: 2_000,
            min_interval_ms: 500,
            max_interval_ms: 10_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthPollErrorCode {
    TransportUnavailable,
    Timeout,
    InvalidResponse,
    Unauthorized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthPollError {
    pub code: HealthPollErrorCode,
    pub message: String,
}

impl HealthPollError {
    pub fn new(code: HealthPollErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthPollingContext {
    pub running: bool,
    pub poll_in_flight: bool,
    pub interval_ms: u64,
    pub last_polled_at_unix_ms: Option<u64>,
    pub next_poll_due_at_unix_ms: Option<u64>,
    pub consecutive_failures: u32,
    pub last_error_code: Option<HealthPollErrorCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthPollSkipReason {
    Paused,
    NotDue { next_due_at_unix_ms: u64 },
    PollInFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthPollingLease {
    token: u64,
    started_at_unix_ms: u64,
}

impl HealthPollingLease {
    pub fn started_at_unix_ms(&self) -> u64 {
        self.started_at_unix_ms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthPollDecision {
    Started(HealthPollingLease),
    Skipped(HealthPollSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthPollCompletion {
    Success { relay_count: usize },
    Failed { error_code: HealthPollErrorCode },
}

pub trait RelayHealthPoller {
    fn poll_relay_health(&mut self) -> Result<Vec<RelayHealthSnapshot>, HealthPollError>;
}

pub trait FailoverEvaluatorSink {
    fn on_health_poll_success(
        &mut self,
        snapshots: &[RelayHealthSnapshot],
        context: &HealthPollingContext,
    );
    fn on_health_poll_failure(&mut self, error: &HealthPollError, context: &HealthPollingContext);
}

#[derive(Debug, Clone)]
pub struct HealthPollingScheduler {
    context: HealthPollingContext,
    lease_seq: u64,
}

impl HealthPollingScheduler {
    pub fn new(config: HealthPollingConfig) -> Self {
        let bounded_interval = config.bounded_interval_ms();
        Self {
            context: HealthPollingContext {
                running: false,
                poll_in_flight: false,
                interval_ms: bounded_interval,
                last_polled_at_unix_ms: None,
                next_poll_due_at_unix_ms: None,
                consecutive_failures: 0,
                last_error_code: None,
            },
            lease_seq: 0,
        }
    }

    pub fn context(&self) -> &HealthPollingContext {
        &self.context
    }

    pub fn begin_poll(&mut self, now_unix_ms: u64) -> HealthPollDecision {
        if !self.context.running {
            return HealthPollDecision::Skipped(HealthPollSkipReason::Paused);
        }

        if self.context.poll_in_flight {
            return HealthPollDecision::Skipped(HealthPollSkipReason::PollInFlight);
        }

        if let Some(next_due) = self.context.next_poll_due_at_unix_ms {
            if now_unix_ms < next_due {
                return HealthPollDecision::Skipped(HealthPollSkipReason::NotDue {
                    next_due_at_unix_ms: next_due,
                });
            }
        }

        self.context.poll_in_flight = true;
        let lease = HealthPollingLease {
            token: self.lease_seq,
            started_at_unix_ms: now_unix_ms,
        };
        self.lease_seq = self.lease_seq.wrapping_add(1);
        HealthPollDecision::Started(lease)
    }

    pub fn complete_poll(
        &mut self,
        lease: HealthPollingLease,
        result: Result<Vec<RelayHealthSnapshot>, HealthPollError>,
        evaluator: &mut dyn FailoverEvaluatorSink,
    ) -> HealthPollCompletion {
        // Ignore mismatched completion and keep the current in-flight poll protected.
        if !self.context.poll_in_flight || lease.token != self.lease_seq.wrapping_sub(1) {
            return HealthPollCompletion::Failed {
                error_code: HealthPollErrorCode::InvalidResponse,
            };
        }

        self.context.poll_in_flight = false;
        self.context.last_polled_at_unix_ms = Some(lease.started_at_unix_ms);
        self.context.next_poll_due_at_unix_ms = Some(
            lease
                .started_at_unix_ms
                .saturating_add(self.context.interval_ms),
        );

        match result {
            Ok(snapshots) => {
                self.context.consecutive_failures = 0;
                self.context.last_error_code = None;
                evaluator.on_health_poll_success(&snapshots, &self.context);
                HealthPollCompletion::Success {
                    relay_count: snapshots.len(),
                }
            }
            Err(error) => {
                self.context.consecutive_failures =
                    self.context.consecutive_failures.saturating_add(1);
                self.context.last_error_code = Some(error.code);
                evaluator.on_health_poll_failure(&error, &self.context);
                HealthPollCompletion::Failed {
                    error_code: error.code,
                }
            }
        }
    }

    pub fn poll_if_due(
        &mut self,
        now_unix_ms: u64,
        poller: &mut dyn RelayHealthPoller,
        evaluator: &mut dyn FailoverEvaluatorSink,
    ) -> HealthPollDecision {
        let decision = self.begin_poll(now_unix_ms);
        if let HealthPollDecision::Started(lease) = decision {
            let result = poller.poll_relay_health();
            let _ = self.complete_poll(lease, result, evaluator);
            HealthPollDecision::Started(lease)
        } else {
            decision
        }
    }
}

pub fn start_health_polling(scheduler: &mut HealthPollingScheduler, now_unix_ms: u64) {
    scheduler.context.running = true;
    scheduler.context.next_poll_due_at_unix_ms = Some(now_unix_ms);
}

pub fn stop_health_polling(scheduler: &mut HealthPollingScheduler) {
    scheduler.context.running = false;
    scheduler.context.poll_in_flight = false;
}

#[cfg(test)]
mod tests {
    use super::{
        start_health_polling, stop_health_polling, FailoverEvaluatorSink, HealthPollCompletion,
        HealthPollDecision, HealthPollError, HealthPollErrorCode, HealthPollSkipReason,
        HealthPollingConfig, HealthPollingScheduler, RelayHealthPoller,
    };
    use crate::routing::{RelayHealthSnapshot, RelayHealthStatus};

    #[derive(Default)]
    struct RecordingEvaluator {
        success_count: usize,
        failure_codes: Vec<HealthPollErrorCode>,
    }

    impl FailoverEvaluatorSink for RecordingEvaluator {
        fn on_health_poll_success(
            &mut self,
            _snapshots: &[RelayHealthSnapshot],
            _context: &super::HealthPollingContext,
        ) {
            self.success_count += 1;
        }

        fn on_health_poll_failure(
            &mut self,
            error: &HealthPollError,
            _context: &super::HealthPollingContext,
        ) {
            self.failure_codes.push(error.code);
        }
    }

    struct StubPoller {
        result: Result<Vec<RelayHealthSnapshot>, HealthPollError>,
    }

    impl RelayHealthPoller for StubPoller {
        fn poll_relay_health(&mut self) -> Result<Vec<RelayHealthSnapshot>, HealthPollError> {
            self.result.clone()
        }
    }

    fn ok_snapshot() -> RelayHealthSnapshot {
        RelayHealthSnapshot {
            relay_id: "sin-01".to_string(),
            status: RelayHealthStatus::Ok,
            latency_ms: 48,
        }
    }

    #[test]
    fn polling_interval_is_configurable_and_bounded() {
        let scheduler_min = HealthPollingScheduler::new(HealthPollingConfig {
            interval_ms: 100,
            min_interval_ms: 500,
            max_interval_ms: 5_000,
        });
        assert_eq!(scheduler_min.context().interval_ms, 500);

        let scheduler_max = HealthPollingScheduler::new(HealthPollingConfig {
            interval_ms: 8_000,
            min_interval_ms: 500,
            max_interval_ms: 5_000,
        });
        assert_eq!(scheduler_max.context().interval_ms, 5_000);

        let scheduler_valid = HealthPollingScheduler::new(HealthPollingConfig {
            interval_ms: 2_000,
            min_interval_ms: 500,
            max_interval_ms: 5_000,
        });
        assert_eq!(scheduler_valid.context().interval_ms, 2_000);
    }

    #[test]
    fn scheduler_avoids_concurrent_overlapping_polls() {
        let mut scheduler = HealthPollingScheduler::new(HealthPollingConfig::default());
        start_health_polling(&mut scheduler, 10_000);

        let lease = match scheduler.begin_poll(10_000) {
            HealthPollDecision::Started(lease) => lease,
            _ => panic!("first due poll must start"),
        };

        let second_attempt = scheduler.begin_poll(10_001);
        assert_eq!(
            second_attempt,
            HealthPollDecision::Skipped(HealthPollSkipReason::PollInFlight)
        );

        let mut evaluator = RecordingEvaluator::default();
        let completion = scheduler.complete_poll(lease, Ok(vec![ok_snapshot()]), &mut evaluator);
        assert_eq!(completion, HealthPollCompletion::Success { relay_count: 1 });
        assert!(!scheduler.context().poll_in_flight);
    }

    #[test]
    fn poll_failures_propagate_to_failover_evaluator_path() {
        let mut scheduler = HealthPollingScheduler::new(HealthPollingConfig::default());
        start_health_polling(&mut scheduler, 20_000);

        let mut poller = StubPoller {
            result: Err(HealthPollError::new(
                HealthPollErrorCode::Timeout,
                "relay health endpoint timed out",
            )),
        };
        let mut evaluator = RecordingEvaluator::default();

        let decision = scheduler.poll_if_due(20_000, &mut poller, &mut evaluator);
        assert!(matches!(decision, HealthPollDecision::Started(_)));
        assert_eq!(evaluator.failure_codes, vec![HealthPollErrorCode::Timeout]);
        assert_eq!(scheduler.context().consecutive_failures, 1);
        assert_eq!(
            scheduler.context().last_error_code,
            Some(HealthPollErrorCode::Timeout)
        );
    }

    #[test]
    fn polling_can_be_paused_and_resumed_by_routing_lifecycle() {
        let mut scheduler = HealthPollingScheduler::new(HealthPollingConfig::default());
        let mut evaluator = RecordingEvaluator::default();
        let mut poller = StubPoller {
            result: Ok(vec![ok_snapshot()]),
        };

        start_health_polling(&mut scheduler, 30_000);
        let first = scheduler.poll_if_due(30_000, &mut poller, &mut evaluator);
        assert!(matches!(first, HealthPollDecision::Started(_)));
        assert_eq!(evaluator.success_count, 1);

        stop_health_polling(&mut scheduler);
        let paused = scheduler.poll_if_due(30_001, &mut poller, &mut evaluator);
        assert_eq!(
            paused,
            HealthPollDecision::Skipped(HealthPollSkipReason::Paused)
        );
        assert_eq!(evaluator.success_count, 1);

        start_health_polling(&mut scheduler, 40_000);
        let resumed = scheduler.poll_if_due(40_000, &mut poller, &mut evaluator);
        assert!(matches!(resumed, HealthPollDecision::Started(_)));
        assert_eq!(evaluator.success_count, 2);
    }
}
