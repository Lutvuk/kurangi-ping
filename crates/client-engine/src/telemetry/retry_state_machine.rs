//! Retry/expiry state machine for telemetry batch delivery.

use super::delivery_client::{TelemetryDeliveryOutcome, TelemetryTransportError};

pub const DEFAULT_MAX_RETRY_ATTEMPTS: u8 = 5;
pub const DEFAULT_BASE_BACKOFF_MS: u64 = 1_000;
pub const DEFAULT_MAX_BACKOFF_MS: u64 = 30_000;
pub const DEFAULT_BATCH_TTL_MS: u64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryRetryPolicy {
    pub max_retry_attempts: u8,
    pub base_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub batch_ttl_ms: u64,
}

impl TelemetryRetryPolicy {
    pub fn sanitized(self) -> Self {
        let max_retry_attempts = self.max_retry_attempts.max(1);
        let base_backoff_ms = self.base_backoff_ms.max(1);
        let max_backoff_ms = self.max_backoff_ms.max(base_backoff_ms);
        let batch_ttl_ms = self.batch_ttl_ms.max(1);

        Self {
            max_retry_attempts,
            base_backoff_ms,
            max_backoff_ms,
            batch_ttl_ms,
        }
    }
}

impl Default for TelemetryRetryPolicy {
    fn default() -> Self {
        Self {
            max_retry_attempts: DEFAULT_MAX_RETRY_ATTEMPTS,
            base_backoff_ms: DEFAULT_BASE_BACKOFF_MS,
            max_backoff_ms: DEFAULT_MAX_BACKOFF_MS,
            batch_ttl_ms: DEFAULT_BATCH_TTL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryBatchStatus {
    Queued,
    RetryScheduled,
    Delivered,
    TerminalFailed,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryBatchState {
    pub status: TelemetryBatchStatus,
    pub created_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub attempt_count: u8,
    pub last_attempt_at_unix_ms: Option<u64>,
    pub next_retry_at_unix_ms: Option<u64>,
    pub last_failure_code: Option<String>,
    pub last_request_id: Option<String>,
    pub purge_eligible: bool,
}

impl TelemetryBatchState {
    pub fn new_queued(created_at_unix_ms: u64, policy: TelemetryRetryPolicy) -> Self {
        let policy = policy.sanitized();
        Self {
            status: TelemetryBatchStatus::Queued,
            created_at_unix_ms,
            expires_at_unix_ms: created_at_unix_ms.saturating_add(policy.batch_ttl_ms),
            attempt_count: 0,
            last_attempt_at_unix_ms: None,
            next_retry_at_unix_ms: None,
            last_failure_code: None,
            last_request_id: None,
            purge_eligible: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryBatchSignal {
    Tick,
    DeliveryOutcome(TelemetryDeliveryOutcome),
    TransportError(TelemetryTransportError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryBatchTransitionReason {
    Noop,
    Accepted,
    PartiallyAccepted,
    RetryScheduled,
    RetryReady,
    NonRetryableFailure,
    RetryBudgetExhausted,
    Expired,
    TransportRetryScheduled,
}

impl TelemetryBatchTransitionReason {
    pub fn as_code(self) -> &'static str {
        match self {
            Self::Noop => "noop",
            Self::Accepted => "accepted",
            Self::PartiallyAccepted => "partially_accepted",
            Self::RetryScheduled => "retry_scheduled",
            Self::RetryReady => "retry_ready",
            Self::NonRetryableFailure => "non_retryable_failure",
            Self::RetryBudgetExhausted => "retry_budget_exhausted",
            Self::Expired => "expired",
            Self::TransportRetryScheduled => "transport_retry_scheduled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryBatchStateTransition {
    pub next_state: TelemetryBatchState,
    pub reason: TelemetryBatchTransitionReason,
}

pub fn next_batch_state(
    current: &TelemetryBatchState,
    signal: TelemetryBatchSignal,
    now_unix_ms: u64,
    policy: TelemetryRetryPolicy,
) -> TelemetryBatchStateTransition {
    let policy = policy.sanitized();

    if current.status != TelemetryBatchStatus::Delivered
        && current.status != TelemetryBatchStatus::Expired
        && now_unix_ms >= current.expires_at_unix_ms
    {
        let mut expired = current.clone();
        expired.status = TelemetryBatchStatus::Expired;
        expired.next_retry_at_unix_ms = None;
        expired.purge_eligible = true;
        return TelemetryBatchStateTransition {
            next_state: expired,
            reason: TelemetryBatchTransitionReason::Expired,
        };
    }

    match signal {
        TelemetryBatchSignal::Tick => {
            if current.status == TelemetryBatchStatus::RetryScheduled
                && current
                    .next_retry_at_unix_ms
                    .is_some_and(|next_retry| next_retry <= now_unix_ms)
            {
                let mut ready = current.clone();
                ready.status = TelemetryBatchStatus::Queued;
                ready.next_retry_at_unix_ms = None;
                return TelemetryBatchStateTransition {
                    next_state: ready,
                    reason: TelemetryBatchTransitionReason::RetryReady,
                };
            }

            TelemetryBatchStateTransition {
                next_state: current.clone(),
                reason: TelemetryBatchTransitionReason::Noop,
            }
        }
        TelemetryBatchSignal::DeliveryOutcome(outcome) => {
            handle_delivery_outcome(current, outcome, now_unix_ms, policy)
        }
        TelemetryBatchSignal::TransportError(error) => {
            schedule_retry_from_failure(
                current,
                now_unix_ms,
                policy,
                "transport_error",
                Some(error.code.as_code().to_string()),
                None,
                TelemetryBatchTransitionReason::TransportRetryScheduled,
            )
        }
    }
}

pub fn compute_next_retry_at(
    now_unix_ms: u64,
    retry_index: u8,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
    retry_after_seconds: Option<u64>,
) -> u64 {
    if let Some(seconds) = retry_after_seconds {
        if seconds > 0 {
            return now_unix_ms.saturating_add(seconds.saturating_mul(1_000));
        }
    }

    if retry_index == 0 {
        return now_unix_ms;
    }

    let base = base_backoff_ms.max(1);
    let max = max_backoff_ms.max(base);
    let multiplier = 2_u64.saturating_pow((retry_index - 1) as u32);
    let delay_ms = base.saturating_mul(multiplier).min(max);
    now_unix_ms.saturating_add(delay_ms)
}

fn handle_delivery_outcome(
    current: &TelemetryBatchState,
    outcome: TelemetryDeliveryOutcome,
    now_unix_ms: u64,
    policy: TelemetryRetryPolicy,
) -> TelemetryBatchStateTransition {
    match outcome {
        TelemetryDeliveryOutcome::Accepted { request_id, .. } => {
            let mut delivered = current.clone();
            delivered.status = TelemetryBatchStatus::Delivered;
            delivered.last_attempt_at_unix_ms = Some(now_unix_ms);
            delivered.next_retry_at_unix_ms = None;
            delivered.last_failure_code = None;
            delivered.last_request_id = Some(request_id);
            delivered.purge_eligible = false;

            TelemetryBatchStateTransition {
                next_state: delivered,
                reason: TelemetryBatchTransitionReason::Accepted,
            }
        }
        TelemetryDeliveryOutcome::PartiallyAccepted { request_id, .. } => {
            let mut delivered = current.clone();
            delivered.status = TelemetryBatchStatus::Delivered;
            delivered.last_attempt_at_unix_ms = Some(now_unix_ms);
            delivered.next_retry_at_unix_ms = None;
            delivered.last_failure_code = None;
            delivered.last_request_id = Some(request_id);
            delivered.purge_eligible = false;

            TelemetryBatchStateTransition {
                next_state: delivered,
                reason: TelemetryBatchTransitionReason::PartiallyAccepted,
            }
        }
        TelemetryDeliveryOutcome::Failed {
            retryable,
            error_code,
            retry_after_seconds,
            request_id,
            ..
        } => {
            if !retryable {
                let mut terminal = current.clone();
                terminal.status = TelemetryBatchStatus::TerminalFailed;
                terminal.last_attempt_at_unix_ms = Some(now_unix_ms);
                terminal.next_retry_at_unix_ms = None;
                terminal.last_failure_code = Some(
                    error_code.unwrap_or_else(|| "delivery_failed_non_retryable".to_string()),
                );
                terminal.last_request_id = request_id;
                terminal.purge_eligible = false;

                return TelemetryBatchStateTransition {
                    next_state: terminal,
                    reason: TelemetryBatchTransitionReason::NonRetryableFailure,
                };
            }

            schedule_retry_from_failure(
                current,
                now_unix_ms,
                policy,
                "delivery_failed_retryable",
                error_code,
                retry_after_seconds,
                TelemetryBatchTransitionReason::RetryScheduled,
            )
        }
    }
}

fn schedule_retry_from_failure(
    current: &TelemetryBatchState,
    now_unix_ms: u64,
    policy: TelemetryRetryPolicy,
    fallback_failure_code: &str,
    failure_code: Option<String>,
    retry_after_seconds: Option<u64>,
    scheduled_reason: TelemetryBatchTransitionReason,
) -> TelemetryBatchStateTransition {
    let next_attempt = current.attempt_count.saturating_add(1);
    if next_attempt > policy.max_retry_attempts {
        let mut exhausted = current.clone();
        exhausted.status = TelemetryBatchStatus::TerminalFailed;
        exhausted.last_attempt_at_unix_ms = Some(now_unix_ms);
        exhausted.next_retry_at_unix_ms = None;
        exhausted.last_failure_code =
            Some(failure_code.unwrap_or_else(|| fallback_failure_code.to_string()));
        exhausted.purge_eligible = false;

        return TelemetryBatchStateTransition {
            next_state: exhausted,
            reason: TelemetryBatchTransitionReason::RetryBudgetExhausted,
        };
    }

    let next_retry_at = compute_next_retry_at(
        now_unix_ms,
        next_attempt,
        policy.base_backoff_ms,
        policy.max_backoff_ms,
        retry_after_seconds,
    );

    let mut scheduled = current.clone();
    scheduled.status = TelemetryBatchStatus::RetryScheduled;
    scheduled.attempt_count = next_attempt;
    scheduled.last_attempt_at_unix_ms = Some(now_unix_ms);
    scheduled.next_retry_at_unix_ms = Some(next_retry_at);
    scheduled.last_failure_code =
        Some(failure_code.unwrap_or_else(|| fallback_failure_code.to_string()));
    scheduled.purge_eligible = false;

    TelemetryBatchStateTransition {
        next_state: scheduled,
        reason: scheduled_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_next_retry_at, next_batch_state, TelemetryBatchSignal, TelemetryBatchState,
        TelemetryBatchStatus, TelemetryBatchTransitionReason, TelemetryRetryPolicy,
    };
    use crate::telemetry::delivery_client::{
        TelemetryDeliveryOutcome, TelemetryTransportError, TelemetryTransportErrorCode,
    };

    fn default_policy() -> TelemetryRetryPolicy {
        TelemetryRetryPolicy {
            max_retry_attempts: 2,
            base_backoff_ms: 250,
            max_backoff_ms: 1_000,
            batch_ttl_ms: 5_000,
        }
    }

    #[test]
    fn retry_budget_is_respected() {
        let policy = default_policy();
        let state = TelemetryBatchState::new_queued(1_000, policy);

        let first = next_batch_state(
            &state,
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 503,
                retryable: true,
                error_code: Some("INTERNAL_ERROR".to_string()),
                request_id: Some("req-1".to_string()),
                retry_after_seconds: None,
                idempotency_key: "idem-1".to_string(),
            }),
            1_500,
            policy,
        );
        assert_eq!(first.reason, TelemetryBatchTransitionReason::RetryScheduled);
        assert_eq!(first.next_state.attempt_count, 1);
        assert_eq!(first.next_state.status, TelemetryBatchStatus::RetryScheduled);

        let second = next_batch_state(
            &first.next_state,
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 503,
                retryable: true,
                error_code: Some("INTERNAL_ERROR".to_string()),
                request_id: Some("req-2".to_string()),
                retry_after_seconds: None,
                idempotency_key: "idem-2".to_string(),
            }),
            2_000,
            policy,
        );
        assert_eq!(second.reason, TelemetryBatchTransitionReason::RetryScheduled);
        assert_eq!(second.next_state.attempt_count, 2);

        let exhausted = next_batch_state(
            &second.next_state,
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 503,
                retryable: true,
                error_code: Some("INTERNAL_ERROR".to_string()),
                request_id: Some("req-3".to_string()),
                retry_after_seconds: None,
                idempotency_key: "idem-3".to_string(),
            }),
            2_500,
            policy,
        );
        assert_eq!(
            exhausted.reason,
            TelemetryBatchTransitionReason::RetryBudgetExhausted
        );
        assert_eq!(exhausted.next_state.status, TelemetryBatchStatus::TerminalFailed);
    }

    #[test]
    fn expired_batches_become_purge_eligible() {
        let policy = default_policy();
        let state = TelemetryBatchState::new_queued(10_000, policy);

        let expired = next_batch_state(&state, TelemetryBatchSignal::Tick, 16_000, policy);
        assert_eq!(expired.reason, TelemetryBatchTransitionReason::Expired);
        assert_eq!(expired.next_state.status, TelemetryBatchStatus::Expired);
        assert!(expired.next_state.purge_eligible);
    }

    #[test]
    fn backoff_schedule_is_deterministic() {
        let now = 100_000;
        assert_eq!(compute_next_retry_at(now, 0, 250, 1_000, None), 100_000);
        assert_eq!(compute_next_retry_at(now, 1, 250, 1_000, None), 100_250);
        assert_eq!(compute_next_retry_at(now, 2, 250, 1_000, None), 100_500);
        assert_eq!(compute_next_retry_at(now, 3, 250, 1_000, None), 101_000);
        assert_eq!(compute_next_retry_at(now, 4, 250, 1_000, None), 101_000);
        assert_eq!(
            compute_next_retry_at(now, 1, 250, 1_000, Some(5)),
            105_000
        );
    }

    #[test]
    fn state_transitions_are_fully_covered_for_success_non_retryable_and_transport() {
        let policy = default_policy();
        let state = TelemetryBatchState::new_queued(1_000, policy);

        let accepted = next_batch_state(
            &state,
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Accepted {
                accepted: 1,
                rejected: 0,
                request_id: "req-accepted".to_string(),
                idempotency_key: "idem-accepted".to_string(),
            }),
            1_100,
            policy,
        );
        assert_eq!(accepted.reason, TelemetryBatchTransitionReason::Accepted);
        assert_eq!(accepted.next_state.status, TelemetryBatchStatus::Delivered);

        let non_retryable = next_batch_state(
            &state,
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 422,
                retryable: false,
                error_code: Some("TELEMETRY_SCHEMA_VIOLATION".to_string()),
                request_id: Some("req-reject".to_string()),
                retry_after_seconds: None,
                idempotency_key: "idem-reject".to_string(),
            }),
            1_200,
            policy,
        );
        assert_eq!(
            non_retryable.reason,
            TelemetryBatchTransitionReason::NonRetryableFailure
        );
        assert_eq!(
            non_retryable.next_state.status,
            TelemetryBatchStatus::TerminalFailed
        );

        let transport_retry = next_batch_state(
            &state,
            TelemetryBatchSignal::TransportError(TelemetryTransportError {
                code: TelemetryTransportErrorCode::Timeout,
                message: "timeout".to_string(),
            }),
            1_300,
            policy,
        );
        assert_eq!(
            transport_retry.reason,
            TelemetryBatchTransitionReason::TransportRetryScheduled
        );
        assert_eq!(
            transport_retry.next_state.status,
            TelemetryBatchStatus::RetryScheduled
        );

        let queued_again = next_batch_state(
            &transport_retry.next_state,
            TelemetryBatchSignal::Tick,
            transport_retry
                .next_state
                .next_retry_at_unix_ms
                .expect("retry timestamp should exist"),
            policy,
        );
        assert_eq!(queued_again.reason, TelemetryBatchTransitionReason::RetryReady);
        assert_eq!(queued_again.next_state.status, TelemetryBatchStatus::Queued);
    }
}
