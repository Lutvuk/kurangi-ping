use crate::routing::RoutingState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeLoopConfig {
    pub interval_ms: u64,
    pub min_interval_ms: u64,
    pub max_interval_ms: u64,
}

impl ProbeLoopConfig {
    pub fn bounded_interval_ms(&self) -> u64 {
        let (min_bound, max_bound) = if self.min_interval_ms <= self.max_interval_ms {
            (self.min_interval_ms, self.max_interval_ms)
        } else {
            (self.max_interval_ms, self.min_interval_ms)
        };

        self.interval_ms.clamp(min_bound, max_bound)
    }
}

impl Default for ProbeLoopConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1_000,
            min_interval_ms: 250,
            max_interval_ms: 10_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeLoopErrorCode {
    Timeout,
    TransportUnavailable,
    InvalidSample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeLoopError {
    pub code: ProbeLoopErrorCode,
    pub message: String,
}

impl ProbeLoopError {
    pub fn new(code: ProbeLoopErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeLoopLease {
    token: u64,
    started_at_unix_ms: u64,
}

impl ProbeLoopLease {
    pub fn started_at_unix_ms(self) -> u64 {
        self.started_at_unix_ms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeLoopSkipReason {
    Paused,
    NotDue { next_due_at_unix_ms: u64 },
    ProbeInFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeLoopDecision {
    Started(ProbeLoopLease),
    Skipped(ProbeLoopSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeExecutionCompletion {
    Success,
    Failed { error_code: ProbeLoopErrorCode },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeLoopContext {
    pub running: bool,
    pub probe_in_flight: bool,
    pub interval_ms: u64,
    pub last_probed_at_unix_ms: Option<u64>,
    pub next_probe_due_at_unix_ms: Option<u64>,
    pub consecutive_failures: u32,
    pub last_error_code: Option<ProbeLoopErrorCode>,
}

#[derive(Debug, Clone)]
pub struct ProbeLoopScheduler {
    context: ProbeLoopContext,
    lease_seq: u64,
}

impl ProbeLoopScheduler {
    pub fn new(config: ProbeLoopConfig) -> Self {
        Self {
            context: ProbeLoopContext {
                running: false,
                probe_in_flight: false,
                interval_ms: config.bounded_interval_ms(),
                last_probed_at_unix_ms: None,
                next_probe_due_at_unix_ms: None,
                consecutive_failures: 0,
                last_error_code: None,
            },
            lease_seq: 0,
        }
    }

    pub fn context(&self) -> &ProbeLoopContext {
        &self.context
    }

    pub fn begin_probe(&mut self, now_unix_ms: u64) -> ProbeLoopDecision {
        if !self.context.running {
            return ProbeLoopDecision::Skipped(ProbeLoopSkipReason::Paused);
        }

        if self.context.probe_in_flight {
            return ProbeLoopDecision::Skipped(ProbeLoopSkipReason::ProbeInFlight);
        }

        if let Some(next_due) = self.context.next_probe_due_at_unix_ms {
            if now_unix_ms < next_due {
                return ProbeLoopDecision::Skipped(ProbeLoopSkipReason::NotDue {
                    next_due_at_unix_ms: next_due,
                });
            }
        }

        self.context.probe_in_flight = true;
        let lease = ProbeLoopLease {
            token: self.lease_seq,
            started_at_unix_ms: now_unix_ms,
        };
        self.lease_seq = self.lease_seq.wrapping_add(1);
        ProbeLoopDecision::Started(lease)
    }

    pub fn complete_probe(
        &mut self,
        lease: ProbeLoopLease,
        result: Result<(), ProbeLoopError>,
    ) -> ProbeExecutionCompletion {
        if !self.context.probe_in_flight || lease.token != self.lease_seq.wrapping_sub(1) {
            return ProbeExecutionCompletion::Failed {
                error_code: ProbeLoopErrorCode::InvalidSample,
            };
        }

        self.context.probe_in_flight = false;
        self.context.last_probed_at_unix_ms = Some(lease.started_at_unix_ms);
        self.context.next_probe_due_at_unix_ms = Some(
            lease
                .started_at_unix_ms
                .saturating_add(self.context.interval_ms),
        );

        match result {
            Ok(()) => {
                self.context.consecutive_failures = 0;
                self.context.last_error_code = None;
                ProbeExecutionCompletion::Success
            }
            Err(error) => {
                self.context.consecutive_failures =
                    self.context.consecutive_failures.saturating_add(1);
                self.context.last_error_code = Some(error.code);
                ProbeExecutionCompletion::Failed {
                    error_code: error.code,
                }
            }
        }
    }
}

pub fn start_probe_loop(
    scheduler: &mut ProbeLoopScheduler,
    routing_state: RoutingState,
    now_unix_ms: u64,
) -> bool {
    if !matches!(
        routing_state,
        RoutingState::Connected | RoutingState::Degraded
    ) {
        return false;
    }

    if scheduler.context.running {
        return false;
    }

    scheduler.context.running = true;
    scheduler.context.next_probe_due_at_unix_ms = Some(now_unix_ms);
    true
}

pub fn stop_probe_loop(scheduler: &mut ProbeLoopScheduler) {
    scheduler.context.running = false;
    scheduler.context.probe_in_flight = false;
    scheduler.context.next_probe_due_at_unix_ms = None;
}

#[cfg(test)]
mod tests {
    use super::{
        start_probe_loop, stop_probe_loop, ProbeExecutionCompletion, ProbeLoopConfig,
        ProbeLoopDecision, ProbeLoopError, ProbeLoopErrorCode, ProbeLoopScheduler,
        ProbeLoopSkipReason,
    };
    use crate::routing::RoutingState;

    #[test]
    fn probe_loop_starts_only_when_routing_lifecycle_is_active() {
        let mut scheduler = ProbeLoopScheduler::new(ProbeLoopConfig::default());
        let now = 1_700_000_000_000;

        assert!(!start_probe_loop(&mut scheduler, RoutingState::Off, now));
        assert!(!start_probe_loop(
            &mut scheduler,
            RoutingState::Connecting,
            now
        ));
        assert!(!start_probe_loop(&mut scheduler, RoutingState::Failed, now));
        assert!(!scheduler.context().running);

        assert!(start_probe_loop(
            &mut scheduler,
            RoutingState::Connected,
            now
        ));
        assert!(scheduler.context().running);
        assert_eq!(scheduler.context().next_probe_due_at_unix_ms, Some(now));
    }

    #[test]
    fn probe_loop_stops_cleanly_on_off_disconnect() {
        let mut scheduler = ProbeLoopScheduler::new(ProbeLoopConfig::default());
        let now = 1_700_000_010_000;

        assert!(start_probe_loop(
            &mut scheduler,
            RoutingState::Connected,
            now
        ));
        let lease = match scheduler.begin_probe(now) {
            ProbeLoopDecision::Started(lease) => lease,
            _ => panic!("probe should start when due"),
        };
        assert!(scheduler.context().probe_in_flight);

        stop_probe_loop(&mut scheduler);
        assert!(!scheduler.context().running);
        assert!(!scheduler.context().probe_in_flight);
        assert_eq!(scheduler.context().next_probe_due_at_unix_ms, None);

        let paused = scheduler.begin_probe(now.saturating_add(1));
        assert_eq!(
            paused,
            ProbeLoopDecision::Skipped(ProbeLoopSkipReason::Paused)
        );

        // Late completion cannot re-arm the loop after OFF.
        let completion = scheduler.complete_probe(lease, Ok(()));
        assert_eq!(
            completion,
            ProbeExecutionCompletion::Failed {
                error_code: ProbeLoopErrorCode::InvalidSample
            }
        );
    }

    #[test]
    fn interval_is_configurable_and_bounded() {
        let min_bounded = ProbeLoopScheduler::new(ProbeLoopConfig {
            interval_ms: 100,
            min_interval_ms: 250,
            max_interval_ms: 5_000,
        });
        assert_eq!(min_bounded.context().interval_ms, 250);

        let max_bounded = ProbeLoopScheduler::new(ProbeLoopConfig {
            interval_ms: 8_000,
            min_interval_ms: 250,
            max_interval_ms: 5_000,
        });
        assert_eq!(max_bounded.context().interval_ms, 5_000);

        let in_range = ProbeLoopScheduler::new(ProbeLoopConfig {
            interval_ms: 1_500,
            min_interval_ms: 250,
            max_interval_ms: 5_000,
        });
        assert_eq!(in_range.context().interval_ms, 1_500);
    }

    #[test]
    fn concurrent_loop_duplication_is_prevented() {
        let mut scheduler = ProbeLoopScheduler::new(ProbeLoopConfig::default());
        let now = 1_700_000_020_000;

        assert!(start_probe_loop(
            &mut scheduler,
            RoutingState::Connected,
            now
        ));
        assert!(!start_probe_loop(
            &mut scheduler,
            RoutingState::Connected,
            now.saturating_add(1)
        ));

        let first = match scheduler.begin_probe(now) {
            ProbeLoopDecision::Started(lease) => lease,
            _ => panic!("first probe should be started"),
        };
        assert_eq!(
            scheduler.begin_probe(now.saturating_add(1)),
            ProbeLoopDecision::Skipped(ProbeLoopSkipReason::ProbeInFlight)
        );

        let completion = scheduler.complete_probe(
            first,
            Err(ProbeLoopError::new(
                ProbeLoopErrorCode::Timeout,
                "probe timed out",
            )),
        );
        assert_eq!(
            completion,
            ProbeExecutionCompletion::Failed {
                error_code: ProbeLoopErrorCode::Timeout
            }
        );
        assert_eq!(scheduler.context().consecutive_failures, 1);
        assert_eq!(
            scheduler.context().last_error_code,
            Some(ProbeLoopErrorCode::Timeout)
        );
    }
}
