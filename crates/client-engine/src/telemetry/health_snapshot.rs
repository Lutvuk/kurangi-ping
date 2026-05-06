use super::batch_queue::BackpressureDiagnostic;
use super::delivery_client::TelemetryDeliveryDiagnostic;
use super::retry_state_machine::{TelemetryBatchState, TelemetryBatchStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastDeliveryStatus {
    Accepted,
    Partial,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryHealthSnapshot {
    pub queue_depth: usize,
    pub drop_count: usize,
    pub retry_scheduled_batches: usize,
    pub total_retry_attempts: u64,
    pub delivered_batches: usize,
    pub terminal_failed_batches: usize,
    pub expired_batches: usize,
    pub purge_eligible_batches: usize,
    pub last_delivery_status: Option<LastDeliveryStatus>,
    pub last_delivery_http_status: Option<u16>,
    pub last_delivery_retryable: Option<bool>,
    pub last_delivery_error_code: Option<String>,
    pub last_delivery_request_id: Option<String>,
}

pub fn build_telemetry_health_snapshot(
    queue_depth: usize,
    backpressure_diagnostics: &[BackpressureDiagnostic],
    delivery_diagnostics: &[TelemetryDeliveryDiagnostic],
    batch_states: &[TelemetryBatchState],
) -> TelemetryHealthSnapshot {
    let drop_count = backpressure_diagnostics.len();
    let retry_scheduled_batches = batch_states
        .iter()
        .filter(|state| state.status == TelemetryBatchStatus::RetryScheduled)
        .count();
    let total_retry_attempts = batch_states
        .iter()
        .map(|state| u64::from(state.attempt_count))
        .sum::<u64>();
    let delivered_batches = batch_states
        .iter()
        .filter(|state| state.status == TelemetryBatchStatus::Delivered)
        .count();
    let terminal_failed_batches = batch_states
        .iter()
        .filter(|state| state.status == TelemetryBatchStatus::TerminalFailed)
        .count();
    let expired_batches = batch_states
        .iter()
        .filter(|state| state.status == TelemetryBatchStatus::Expired)
        .count();
    let purge_eligible_batches = batch_states.iter().filter(|state| state.purge_eligible).count();

    let (
        last_delivery_status,
        last_delivery_http_status,
        last_delivery_retryable,
        last_delivery_error_code,
        last_delivery_request_id,
    ) = match delivery_diagnostics.last() {
        Some(last) => (
            Some(parse_delivery_status(&last.status)),
            last.http_status,
            Some(last.retryable),
            last.error_code.clone(),
            last.request_id.clone(),
        ),
        None => (None, None, None, None, None),
    };

    TelemetryHealthSnapshot {
        queue_depth,
        drop_count,
        retry_scheduled_batches,
        total_retry_attempts,
        delivered_batches,
        terminal_failed_batches,
        expired_batches,
        purge_eligible_batches,
        last_delivery_status,
        last_delivery_http_status,
        last_delivery_retryable,
        last_delivery_error_code,
        last_delivery_request_id,
    }
}

fn parse_delivery_status(raw: &str) -> LastDeliveryStatus {
    match raw {
        "accepted" => LastDeliveryStatus::Accepted,
        "partial" => LastDeliveryStatus::Partial,
        "failed" => LastDeliveryStatus::Failed,
        _ => LastDeliveryStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_telemetry_health_snapshot, LastDeliveryStatus};
    use crate::telemetry::batch_queue::{BackpressureDiagnostic, BackpressurePolicy};
    use crate::telemetry::delivery_client::TelemetryDeliveryDiagnostic;
    use crate::telemetry::retry_state_machine::{
        TelemetryBatchState, TelemetryBatchStatus, TelemetryRetryPolicy,
    };

    fn state(
        status: TelemetryBatchStatus,
        attempt_count: u8,
        purge_eligible: bool,
    ) -> TelemetryBatchState {
        let mut snapshot = TelemetryBatchState::new_queued(1_000, TelemetryRetryPolicy::default());
        snapshot.status = status;
        snapshot.attempt_count = attempt_count;
        snapshot.purge_eligible = purge_eligible;
        snapshot
    }

    #[test]
    fn snapshot_includes_queue_retry_drop_and_last_delivery_status() {
        let backpressure = vec![
            BackpressureDiagnostic {
                dropped_event_name: "old-1".to_string(),
                incoming_event_name: "new-1".to_string(),
                queue_depth_before: 100,
                queue_depth_after: 99,
                policy: BackpressurePolicy::DropOldest,
            },
            BackpressureDiagnostic {
                dropped_event_name: "old-2".to_string(),
                incoming_event_name: "new-2".to_string(),
                queue_depth_before: 100,
                queue_depth_after: 99,
                policy: BackpressurePolicy::DropOldest,
            },
        ];
        let deliveries = vec![
            TelemetryDeliveryDiagnostic {
                status: "accepted".to_string(),
                http_status: Some(202),
                retryable: false,
                request_id: Some("req-1".to_string()),
                error_code: None,
                idempotency_key: "idem-1".to_string(),
            },
            TelemetryDeliveryDiagnostic {
                status: "failed".to_string(),
                http_status: Some(503),
                retryable: true,
                request_id: Some("req-2".to_string()),
                error_code: Some("INTERNAL_ERROR".to_string()),
                idempotency_key: "idem-2".to_string(),
            },
        ];
        let states = vec![
            state(TelemetryBatchStatus::RetryScheduled, 2, false),
            state(TelemetryBatchStatus::Delivered, 1, false),
            state(TelemetryBatchStatus::TerminalFailed, 3, false),
            state(TelemetryBatchStatus::Expired, 4, true),
        ];

        let snapshot = build_telemetry_health_snapshot(88, &backpressure, &deliveries, &states);
        assert_eq!(snapshot.queue_depth, 88);
        assert_eq!(snapshot.drop_count, 2);
        assert_eq!(snapshot.retry_scheduled_batches, 1);
        assert_eq!(snapshot.total_retry_attempts, 10);
        assert_eq!(snapshot.delivered_batches, 1);
        assert_eq!(snapshot.terminal_failed_batches, 1);
        assert_eq!(snapshot.expired_batches, 1);
        assert_eq!(snapshot.purge_eligible_batches, 1);
        assert_eq!(snapshot.last_delivery_status, Some(LastDeliveryStatus::Failed));
        assert_eq!(snapshot.last_delivery_http_status, Some(503));
        assert_eq!(snapshot.last_delivery_retryable, Some(true));
        assert_eq!(
            snapshot.last_delivery_error_code.as_deref(),
            Some("INTERNAL_ERROR")
        );
        assert_eq!(snapshot.last_delivery_request_id.as_deref(), Some("req-2"));
    }

    #[test]
    fn snapshot_output_excludes_sensitive_payload_details() {
        let deliveries = vec![TelemetryDeliveryDiagnostic {
            status: "failed".to_string(),
            http_status: Some(422),
            retryable: false,
            request_id: Some("req-4".to_string()),
            error_code: Some("TELEMETRY_SCHEMA_VIOLATION".to_string()),
            idempotency_key: "idem-4".to_string(),
        }];
        let snapshot = build_telemetry_health_snapshot(1, &[], &deliveries, &[]);
        let rendered = format!("{snapshot:?}");

        assert!(!rendered.contains("payload"));
        assert!(!rendered.contains("private_host"));
        assert!(!rendered.contains("secret.internal.example"));
    }

    #[test]
    fn snapshot_handles_empty_sources_without_blocking_or_failure() {
        let snapshot = build_telemetry_health_snapshot(0, &[], &[], &[]);
        assert_eq!(snapshot.queue_depth, 0);
        assert_eq!(snapshot.drop_count, 0);
        assert_eq!(snapshot.total_retry_attempts, 0);
        assert_eq!(snapshot.last_delivery_status, None);
        assert_eq!(snapshot.last_delivery_http_status, None);
        assert_eq!(snapshot.last_delivery_retryable, None);
    }
}
