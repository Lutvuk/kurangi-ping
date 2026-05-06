use client_engine::telemetry::batch_queue::{BackpressurePolicy, BatchQueueConfig};
use client_engine::telemetry::delivery_client::{
    TelemetryDeliveryOutcome, TelemetryTransportError, TelemetryTransportErrorCode,
};
use client_engine::telemetry::retry_state_machine::{
    next_batch_state, TelemetryBatchSignal, TelemetryBatchState, TelemetryBatchStatus,
    TelemetryBatchTransitionReason, TelemetryRetryPolicy,
};
use client_engine::telemetry::{TelemetryEvent, TelemetryService};

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeliveryLifecycleFixture {
    name: String,
    queue_max_depth: usize,
    enqueue_events: usize,
    batches: Vec<BatchLifecycleFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchLifecycleFixture {
    batch_id: String,
    created_at_ms: u64,
    steps: Vec<BatchStepFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchStepFixture {
    at_ms: u64,
    signal: DeliverySignalFixture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeliverySignalFixture {
    Accepted,
    RetryableFailure,
    NonRetryableFailure,
    TransportError,
    Tick,
}

#[derive(Debug, Clone, PartialEq)]
struct DeliveryReliabilityKpiReport {
    scenario: String,
    success_rate: f64,
    retry_rate: f64,
    drop_rate: f64,
    total_batches: usize,
    delivered_batches: usize,
    retried_batches: usize,
    dropped_events: usize,
    missing_transition_count: usize,
}

impl DeliveryReliabilityKpiReport {
    fn release_readiness_summary(&self) -> String {
        let release_ready = self.success_rate >= 0.99
            && self.retry_rate <= 0.40
            && self.drop_rate <= 0.05
            && self.missing_transition_count == 0;
        format!(
            "scenario={}; release_ready={}; success_rate={:.3}; retry_rate={:.3}; drop_rate={:.3}; delivered_batches={}/{}; retried_batches={}; dropped_events={}; missing_transitions={}",
            self.scenario,
            release_ready,
            self.success_rate,
            self.retry_rate,
            self.drop_rate,
            self.delivered_batches,
            self.total_batches,
            self.retried_batches,
            self.dropped_events,
            self.missing_transition_count
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchLifecycleResult {
    final_status: TelemetryBatchStatus,
    had_retry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeliveryKpiValidationError {
    NonMonotonicTimestamp {
        scenario: String,
        batch_id: String,
        previous_ms: u64,
        current_ms: u64,
    },
    MissingRetryReadyBeforeTerminal {
        scenario: String,
        batch_id: String,
        terminal_reason: TelemetryBatchTransitionReason,
    },
    RetryReadyWithoutRetryScheduled {
        scenario: String,
        batch_id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum ScenarioOutcome {
    Passed(DeliveryReliabilityKpiReport),
    Failed {
        scenario: String,
        error: DeliveryKpiValidationError,
    },
}

fn run_delivery_reliability_kpi_validation_suite() -> Vec<ScenarioOutcome> {
    let mut fixtures = delivery_lifecycle_fixtures();
    fixtures.sort_by(|left, right| left.name.cmp(&right.name));

    fixtures
        .into_iter()
        .map(|fixture| {
            let scenario_name = fixture.name.clone();
            match run_delivery_reliability_kpi_validation(&fixture) {
                Ok(report) => ScenarioOutcome::Passed(report),
                Err(error) => ScenarioOutcome::Failed {
                    scenario: scenario_name,
                    error,
                },
            }
        })
        .collect()
}

fn run_delivery_reliability_kpi_validation(
    fixture: &DeliveryLifecycleFixture,
) -> Result<DeliveryReliabilityKpiReport, DeliveryKpiValidationError> {
    let retry_policy = TelemetryRetryPolicy {
        max_retry_attempts: 3,
        base_backoff_ms: 100,
        max_backoff_ms: 500,
        batch_ttl_ms: 5_000,
    };

    let mut delivered_batches = 0_usize;
    let mut retried_batches = 0_usize;

    for batch in &fixture.batches {
        let result = simulate_batch_lifecycle(fixture, batch, retry_policy)?;
        if result.final_status == TelemetryBatchStatus::Delivered {
            delivered_batches = delivered_batches.saturating_add(1);
        }
        if result.had_retry {
            retried_batches = retried_batches.saturating_add(1);
        }
    }

    let dropped_events = simulate_backpressure_drops(fixture.queue_max_depth, fixture.enqueue_events);
    let total_batches = fixture.batches.len();
    let success_rate = ratio(delivered_batches, total_batches);
    let retry_rate = ratio(retried_batches, total_batches);
    let drop_rate = ratio(dropped_events, fixture.enqueue_events);

    Ok(DeliveryReliabilityKpiReport {
        scenario: fixture.name.clone(),
        success_rate,
        retry_rate,
        drop_rate,
        total_batches,
        delivered_batches,
        retried_batches,
        dropped_events,
        missing_transition_count: 0,
    })
}

fn simulate_batch_lifecycle(
    fixture: &DeliveryLifecycleFixture,
    batch: &BatchLifecycleFixture,
    retry_policy: TelemetryRetryPolicy,
) -> Result<BatchLifecycleResult, DeliveryKpiValidationError> {
    let mut state = TelemetryBatchState::new_queued(batch.created_at_ms, retry_policy);
    let mut reasons = Vec::new();
    let mut previous_at_ms = batch.created_at_ms;
    let mut had_retry = false;

    for step in &batch.steps {
        if step.at_ms < previous_at_ms {
            return Err(DeliveryKpiValidationError::NonMonotonicTimestamp {
                scenario: fixture.name.clone(),
                batch_id: batch.batch_id.clone(),
                previous_ms: previous_at_ms,
                current_ms: step.at_ms,
            });
        }
        previous_at_ms = step.at_ms;

        let signal = signal_for_step(step, &batch.batch_id, step.at_ms);
        let transition = next_batch_state(&state, signal, step.at_ms, retry_policy);
        if matches!(
            transition.reason,
            TelemetryBatchTransitionReason::RetryScheduled
                | TelemetryBatchTransitionReason::TransportRetryScheduled
        ) {
            had_retry = true;
        }
        reasons.push(transition.reason);
        state = transition.next_state;
    }

    validate_delivery_transition_completeness(fixture, batch, &reasons)?;
    Ok(BatchLifecycleResult {
        final_status: state.status,
        had_retry,
    })
}

fn signal_for_step(step: &BatchStepFixture, batch_id: &str, at_ms: u64) -> TelemetryBatchSignal {
    let request_id = format!("req-{batch_id}-{at_ms}");
    let idempotency_key = format!("idem-{batch_id}-{at_ms}");

    match step.signal {
        DeliverySignalFixture::Accepted => {
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Accepted {
                accepted: 1,
                rejected: 0,
                request_id,
                idempotency_key,
            })
        }
        DeliverySignalFixture::RetryableFailure => {
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 503,
                retryable: true,
                error_code: Some("INTERNAL_ERROR".to_string()),
                request_id: Some(request_id),
                retry_after_seconds: None,
                idempotency_key,
            })
        }
        DeliverySignalFixture::NonRetryableFailure => {
            TelemetryBatchSignal::DeliveryOutcome(TelemetryDeliveryOutcome::Failed {
                http_status: 422,
                retryable: false,
                error_code: Some("TELEMETRY_SCHEMA_VIOLATION".to_string()),
                request_id: Some(request_id),
                retry_after_seconds: None,
                idempotency_key,
            })
        }
        DeliverySignalFixture::TransportError => {
            TelemetryBatchSignal::TransportError(TelemetryTransportError {
                code: TelemetryTransportErrorCode::Timeout,
                message: "timeout".to_string(),
            })
        }
        DeliverySignalFixture::Tick => TelemetryBatchSignal::Tick,
    }
}

fn validate_delivery_transition_completeness(
    fixture: &DeliveryLifecycleFixture,
    batch: &BatchLifecycleFixture,
    reasons: &[TelemetryBatchTransitionReason],
) -> Result<(), DeliveryKpiValidationError> {
    let mut pending_retry = false;

    for reason in reasons {
        match reason {
            TelemetryBatchTransitionReason::RetryScheduled
            | TelemetryBatchTransitionReason::TransportRetryScheduled => {
                pending_retry = true;
            }
            TelemetryBatchTransitionReason::RetryReady => {
                if !pending_retry {
                    return Err(DeliveryKpiValidationError::RetryReadyWithoutRetryScheduled {
                        scenario: fixture.name.clone(),
                        batch_id: batch.batch_id.clone(),
                    });
                }
                pending_retry = false;
            }
            TelemetryBatchTransitionReason::Accepted
            | TelemetryBatchTransitionReason::PartiallyAccepted
            | TelemetryBatchTransitionReason::NonRetryableFailure
            | TelemetryBatchTransitionReason::RetryBudgetExhausted
            | TelemetryBatchTransitionReason::Expired => {
                if pending_retry {
                    return Err(
                        DeliveryKpiValidationError::MissingRetryReadyBeforeTerminal {
                            scenario: fixture.name.clone(),
                            batch_id: batch.batch_id.clone(),
                            terminal_reason: *reason,
                        },
                    );
                }
            }
            TelemetryBatchTransitionReason::Noop => {}
        }
    }

    if pending_retry {
        return Err(DeliveryKpiValidationError::MissingRetryReadyBeforeTerminal {
            scenario: fixture.name.clone(),
            batch_id: batch.batch_id.clone(),
            terminal_reason: TelemetryBatchTransitionReason::RetryScheduled,
        });
    }

    Ok(())
}

fn simulate_backpressure_drops(queue_max_depth: usize, enqueue_events: usize) -> usize {
    let mut service = TelemetryService::with_batch_queue_config(BatchQueueConfig {
        max_queue_depth: queue_max_depth,
        max_batch_size: 200,
        backpressure_policy: BackpressurePolicy::DropOldest,
    });

    for index in 0..enqueue_events {
        service.enqueue(TelemetryEvent::new(format!("telemetry_event_{index}"), Default::default()));
    }

    service.take_backpressure_diagnostics().len()
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn delivery_lifecycle_fixtures() -> Vec<DeliveryLifecycleFixture> {
    vec![
        DeliveryLifecycleFixture {
            name: "nominal_release_ready".to_string(),
            queue_max_depth: 10,
            enqueue_events: 5,
            batches: vec![
                BatchLifecycleFixture {
                    batch_id: "b-1".to_string(),
                    created_at_ms: 1_000,
                    steps: vec![BatchStepFixture {
                        at_ms: 1_100,
                        signal: DeliverySignalFixture::Accepted,
                    }],
                },
                BatchLifecycleFixture {
                    batch_id: "b-2".to_string(),
                    created_at_ms: 2_000,
                    steps: vec![
                        BatchStepFixture {
                            at_ms: 2_100,
                            signal: DeliverySignalFixture::RetryableFailure,
                        },
                        BatchStepFixture {
                            at_ms: 2_250,
                            signal: DeliverySignalFixture::Tick,
                        },
                        BatchStepFixture {
                            at_ms: 2_300,
                            signal: DeliverySignalFixture::Accepted,
                        },
                    ],
                },
                BatchLifecycleFixture {
                    batch_id: "b-3".to_string(),
                    created_at_ms: 3_000,
                    steps: vec![BatchStepFixture {
                        at_ms: 3_100,
                        signal: DeliverySignalFixture::Accepted,
                    }],
                },
            ],
        },
        DeliveryLifecycleFixture {
            name: "drop_heavy_degraded".to_string(),
            queue_max_depth: 2,
            enqueue_events: 5,
            batches: vec![
                BatchLifecycleFixture {
                    batch_id: "b-10".to_string(),
                    created_at_ms: 10_000,
                    steps: vec![
                        BatchStepFixture {
                            at_ms: 10_100,
                            signal: DeliverySignalFixture::TransportError,
                        },
                        BatchStepFixture {
                            at_ms: 10_250,
                            signal: DeliverySignalFixture::Tick,
                        },
                        BatchStepFixture {
                            at_ms: 10_300,
                            signal: DeliverySignalFixture::Accepted,
                        },
                    ],
                },
                BatchLifecycleFixture {
                    batch_id: "b-11".to_string(),
                    created_at_ms: 11_000,
                    steps: vec![BatchStepFixture {
                        at_ms: 11_100,
                        signal: DeliverySignalFixture::NonRetryableFailure,
                    }],
                },
            ],
        },
        DeliveryLifecycleFixture {
            name: "missing_retry_ready".to_string(),
            queue_max_depth: 10,
            enqueue_events: 1,
            batches: vec![BatchLifecycleFixture {
                batch_id: "b-broken".to_string(),
                created_at_ms: 20_000,
                steps: vec![
                    BatchStepFixture {
                        at_ms: 20_100,
                        signal: DeliverySignalFixture::RetryableFailure,
                    },
                    BatchStepFixture {
                        at_ms: 20_120,
                        signal: DeliverySignalFixture::Accepted,
                    },
                ],
            }],
        },
    ]
}

fn approx_eq(left: f64, right: f64) {
    let delta = (left - right).abs();
    assert!(
        delta < 0.000_001,
        "expected {left} ~= {right}, delta={delta}"
    );
}

#[test]
fn kpi_derivation_for_success_retry_and_drop_rate_is_possible() {
    let fixture = delivery_lifecycle_fixtures()
        .into_iter()
        .find(|scenario| scenario.name == "drop_heavy_degraded")
        .expect("drop-heavy fixture should exist");
    let report =
        run_delivery_reliability_kpi_validation(&fixture).expect("valid fixture should aggregate");

    assert_eq!(report.scenario, "drop_heavy_degraded");
    assert_eq!(report.total_batches, 2);
    assert_eq!(report.delivered_batches, 1);
    assert_eq!(report.retried_batches, 1);
    assert_eq!(report.dropped_events, 3);
    approx_eq(report.success_rate, 0.5);
    approx_eq(report.retry_rate, 0.5);
    approx_eq(report.drop_rate, 0.6);
}

#[test]
fn missing_delivery_transitions_are_detected_by_assertions() {
    let fixture = delivery_lifecycle_fixtures()
        .into_iter()
        .find(|scenario| scenario.name == "missing_retry_ready")
        .expect("broken fixture should exist");
    let error = run_delivery_reliability_kpi_validation(&fixture)
        .expect_err("missing retry-ready transition should fail validation");

    assert_eq!(
        error,
        DeliveryKpiValidationError::MissingRetryReadyBeforeTerminal {
            scenario: "missing_retry_ready".to_string(),
            batch_id: "b-broken".to_string(),
            terminal_reason: TelemetryBatchTransitionReason::Accepted,
        }
    );
}

#[test]
fn metrics_are_reproducible_across_test_runs() {
    let first = run_delivery_reliability_kpi_validation_suite();
    let second = run_delivery_reliability_kpi_validation_suite();
    assert_eq!(first, second);
}

#[test]
fn report_output_is_actionable_for_release_readiness() {
    let fixture = delivery_lifecycle_fixtures()
        .into_iter()
        .find(|scenario| scenario.name == "nominal_release_ready")
        .expect("nominal fixture should exist");
    let report =
        run_delivery_reliability_kpi_validation(&fixture).expect("nominal fixture should pass");
    let summary = report.release_readiness_summary();

    assert!(summary.contains("release_ready=true"));
    assert!(summary.contains("success_rate=1.000"));
    assert!(summary.contains("retry_rate=0.333"));
    assert!(summary.contains("drop_rate=0.000"));
    assert!(summary.contains("missing_transitions=0"));
}
