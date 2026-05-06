#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl UpdateChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }
}

pub fn resolve_update_channel(raw: Option<&str>) -> UpdateChannel {
    let normalized = raw.unwrap_or("stable").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "beta" => UpdateChannel::Beta,
        "stable" => UpdateChannel::Stable,
        _ => UpdateChannel::Stable,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateCheckConfig {
    pub interval_ms: u64,
    pub min_interval_ms: u64,
    pub max_interval_ms: u64,
    pub persist_last_check_metadata: bool,
}

impl UpdateCheckConfig {
    pub fn bounded_interval_ms(&self) -> u64 {
        let (min_bound, max_bound) = if self.min_interval_ms <= self.max_interval_ms {
            (self.min_interval_ms, self.max_interval_ms)
        } else {
            (self.max_interval_ms, self.min_interval_ms)
        };
        self.interval_ms.clamp(min_bound, max_bound)
    }
}

impl Default for UpdateCheckConfig {
    fn default() -> Self {
        Self {
            interval_ms: 3_600_000,
            min_interval_ms: 60_000,
            max_interval_ms: 86_400_000,
            persist_last_check_metadata: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckTrigger {
    Scheduled,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckSkipReason {
    NotDue { next_due_at_unix_ms: u64 },
    CheckInFlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateCheckLease {
    token: u64,
    started_at_unix_ms: u64,
    trigger: UpdateCheckTrigger,
}

impl UpdateCheckLease {
    pub fn started_at_unix_ms(self) -> u64 {
        self.started_at_unix_ms
    }

    pub fn trigger(self) -> UpdateCheckTrigger {
        self.trigger
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckDecision {
    Started(UpdateCheckLease),
    Skipped(UpdateCheckSkipReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckErrorCode {
    TransportFailed,
    FeedRejected,
    ApplyDeferred,
    InvalidLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateCheckCompletion {
    Success,
    Failed { code: UpdateCheckErrorCode },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCheckContext {
    pub channel: UpdateChannel,
    pub cadence_ms: u64,
    pub check_in_flight: bool,
    pub next_due_at_unix_ms: Option<u64>,
    pub last_check_started_at_unix_ms: Option<u64>,
    pub last_check_completed_at_unix_ms: Option<u64>,
    pub last_trigger: Option<UpdateCheckTrigger>,
    pub last_error_code: Option<UpdateCheckErrorCode>,
}

#[derive(Debug, Clone)]
pub struct UpdateCheckScheduler {
    config: UpdateCheckConfig,
    context: UpdateCheckContext,
    lease_seq: u64,
}

impl UpdateCheckScheduler {
    pub fn new(channel: UpdateChannel, config: UpdateCheckConfig) -> Self {
        let bounded = config.bounded_interval_ms();
        Self {
            config,
            context: UpdateCheckContext {
                channel,
                cadence_ms: bounded,
                check_in_flight: false,
                next_due_at_unix_ms: Some(0),
                last_check_started_at_unix_ms: None,
                last_check_completed_at_unix_ms: None,
                last_trigger: None,
                last_error_code: None,
            },
            lease_seq: 0,
        }
    }

    pub fn context(&self) -> &UpdateCheckContext {
        &self.context
    }
}

pub fn schedule_update_check(
    scheduler: &mut UpdateCheckScheduler,
    trigger: UpdateCheckTrigger,
    now_unix_ms: u64,
) -> UpdateCheckDecision {
    if scheduler.context.check_in_flight {
        return UpdateCheckDecision::Skipped(UpdateCheckSkipReason::CheckInFlight);
    }

    if matches!(trigger, UpdateCheckTrigger::Scheduled)
        && scheduler
            .context
            .next_due_at_unix_ms
            .is_some_and(|next_due| now_unix_ms < next_due)
    {
        return UpdateCheckDecision::Skipped(UpdateCheckSkipReason::NotDue {
            next_due_at_unix_ms: scheduler.context.next_due_at_unix_ms.unwrap_or(now_unix_ms),
        });
    }

    scheduler.context.check_in_flight = true;
    scheduler.context.last_trigger = Some(trigger);
    let lease = UpdateCheckLease {
        token: scheduler.lease_seq,
        started_at_unix_ms: now_unix_ms,
        trigger,
    };
    scheduler.lease_seq = scheduler.lease_seq.wrapping_add(1);
    UpdateCheckDecision::Started(lease)
}

pub fn complete_update_check(
    scheduler: &mut UpdateCheckScheduler,
    lease: UpdateCheckLease,
    completed_at_unix_ms: u64,
    result: Result<(), UpdateCheckErrorCode>,
) -> UpdateCheckCompletion {
    if !scheduler.context.check_in_flight || lease.token != scheduler.lease_seq.wrapping_sub(1) {
        return UpdateCheckCompletion::Failed {
            code: UpdateCheckErrorCode::InvalidLease,
        };
    }

    scheduler.context.check_in_flight = false;
    scheduler.context.next_due_at_unix_ms =
        Some(completed_at_unix_ms.saturating_add(scheduler.context.cadence_ms));

    if scheduler.config.persist_last_check_metadata {
        scheduler.context.last_check_started_at_unix_ms = Some(lease.started_at_unix_ms);
        scheduler.context.last_check_completed_at_unix_ms = Some(completed_at_unix_ms);
    }

    match result {
        Ok(()) => {
            scheduler.context.last_error_code = None;
            UpdateCheckCompletion::Success
        }
        Err(code) => {
            scheduler.context.last_error_code = Some(code);
            UpdateCheckCompletion::Failed { code }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        complete_update_check, resolve_update_channel, schedule_update_check, UpdateChannel,
        UpdateCheckCompletion, UpdateCheckConfig, UpdateCheckDecision, UpdateCheckErrorCode,
        UpdateCheckScheduler, UpdateCheckSkipReason, UpdateCheckTrigger,
    };

    #[test]
    fn supports_beta_and_stable_channels() {
        assert_eq!(resolve_update_channel(Some("beta")), UpdateChannel::Beta);
        assert_eq!(resolve_update_channel(Some("stable")), UpdateChannel::Stable);
        assert_eq!(resolve_update_channel(Some(" BETA ")), UpdateChannel::Beta);
        assert_eq!(
            resolve_update_channel(Some("unsupported")),
            UpdateChannel::Stable
        );
        assert_eq!(resolve_update_channel(None), UpdateChannel::Stable);
    }

    #[test]
    fn check_cadence_is_configurable_and_bounded() {
        let low = UpdateCheckScheduler::new(
            UpdateChannel::Stable,
            UpdateCheckConfig {
                interval_ms: 5_000,
                min_interval_ms: 10_000,
                max_interval_ms: 50_000,
                persist_last_check_metadata: true,
            },
        );
        assert_eq!(low.context().cadence_ms, 10_000);

        let high = UpdateCheckScheduler::new(
            UpdateChannel::Stable,
            UpdateCheckConfig {
                interval_ms: 60_000,
                min_interval_ms: 10_000,
                max_interval_ms: 50_000,
                persist_last_check_metadata: true,
            },
        );
        assert_eq!(high.context().cadence_ms, 50_000);
    }

    #[test]
    fn manual_check_trigger_coexists_safely_with_scheduled_checks() {
        let mut scheduler = UpdateCheckScheduler::new(UpdateChannel::Stable, UpdateCheckConfig::default());
        let now = 1_700_000_000_000;

        let scheduled = schedule_update_check(&mut scheduler, UpdateCheckTrigger::Scheduled, now);
        let lease = match scheduled {
            UpdateCheckDecision::Started(lease) => lease,
            _ => panic!("scheduled check should start when due"),
        };

        assert_eq!(
            schedule_update_check(
                &mut scheduler,
                UpdateCheckTrigger::Manual,
                now.saturating_add(1)
            ),
            UpdateCheckDecision::Skipped(UpdateCheckSkipReason::CheckInFlight)
        );

        assert_eq!(
            complete_update_check(&mut scheduler, lease, now.saturating_add(2_000), Ok(())),
            UpdateCheckCompletion::Success
        );

        let not_due = schedule_update_check(
            &mut scheduler,
            UpdateCheckTrigger::Scheduled,
            now.saturating_add(2_100),
        );
        assert!(matches!(
            not_due,
            UpdateCheckDecision::Skipped(UpdateCheckSkipReason::NotDue { .. })
        ));

        let manual = schedule_update_check(
            &mut scheduler,
            UpdateCheckTrigger::Manual,
            now.saturating_add(2_100),
        );
        assert!(matches!(manual, UpdateCheckDecision::Started(_)));
    }

    #[test]
    fn last_check_metadata_persists_when_configured() {
        let now = 1_700_100_000_000;

        let mut with_persist = UpdateCheckScheduler::new(
            UpdateChannel::Beta,
            UpdateCheckConfig {
                persist_last_check_metadata: true,
                ..UpdateCheckConfig::default()
            },
        );
        let lease = match schedule_update_check(&mut with_persist, UpdateCheckTrigger::Manual, now) {
            UpdateCheckDecision::Started(lease) => lease,
            _ => panic!("manual check should start"),
        };
        let completed_at = now.saturating_add(1_500);
        assert_eq!(
            complete_update_check(&mut with_persist, lease, completed_at, Ok(())),
            UpdateCheckCompletion::Success
        );
        assert_eq!(with_persist.context().last_check_started_at_unix_ms, Some(now));
        assert_eq!(
            with_persist.context().last_check_completed_at_unix_ms,
            Some(completed_at)
        );

        let mut without_persist = UpdateCheckScheduler::new(
            UpdateChannel::Stable,
            UpdateCheckConfig {
                persist_last_check_metadata: false,
                ..UpdateCheckConfig::default()
            },
        );
        let no_persist_lease =
            match schedule_update_check(&mut without_persist, UpdateCheckTrigger::Manual, now) {
                UpdateCheckDecision::Started(lease) => lease,
                _ => panic!("manual check should start"),
            };
        assert_eq!(
            complete_update_check(
                &mut without_persist,
                no_persist_lease,
                completed_at,
                Err(UpdateCheckErrorCode::TransportFailed)
            ),
            UpdateCheckCompletion::Failed {
                code: UpdateCheckErrorCode::TransportFailed
            }
        );
        assert_eq!(without_persist.context().last_check_started_at_unix_ms, None);
        assert_eq!(without_persist.context().last_check_completed_at_unix_ms, None);
        assert_eq!(
            without_persist.context().last_error_code,
            Some(UpdateCheckErrorCode::TransportFailed)
        );
    }
}
