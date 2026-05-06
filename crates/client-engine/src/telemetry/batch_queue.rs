use std::collections::VecDeque;

use super::TelemetryEvent;

pub const CONTRACT_MAX_BATCH_ITEMS: usize = 200;
pub const DEFAULT_MAX_QUEUE_DEPTH: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressurePolicy {
    DropOldest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchQueueConfig {
    pub max_queue_depth: usize,
    pub max_batch_size: usize,
    pub backpressure_policy: BackpressurePolicy,
}

impl BatchQueueConfig {
    pub fn sanitized(self) -> Self {
        Self {
            max_queue_depth: self.max_queue_depth.max(1),
            max_batch_size: self.max_batch_size.clamp(1, CONTRACT_MAX_BATCH_ITEMS),
            backpressure_policy: self.backpressure_policy,
        }
    }
}

impl Default for BatchQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_depth: DEFAULT_MAX_QUEUE_DEPTH,
            max_batch_size: CONTRACT_MAX_BATCH_ITEMS,
            backpressure_policy: BackpressurePolicy::DropOldest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackpressureDiagnostic {
    pub dropped_event_name: String,
    pub incoming_event_name: String,
    pub queue_depth_before: usize,
    pub queue_depth_after: usize,
    pub policy: BackpressurePolicy,
}

#[derive(Debug, Clone)]
pub struct BatchQueue {
    config: BatchQueueConfig,
    queue: VecDeque<TelemetryEvent>,
    backpressure_diagnostics: Vec<BackpressureDiagnostic>,
}

impl BatchQueue {
    pub fn new(config: BatchQueueConfig) -> Self {
        Self {
            config: config.sanitized(),
            queue: VecDeque::new(),
            backpressure_diagnostics: Vec::new(),
        }
    }

    pub fn depth(&self) -> usize {
        self.queue.len()
    }

    pub fn backpressure_diagnostics(&self) -> &[BackpressureDiagnostic] {
        &self.backpressure_diagnostics
    }

    pub fn take_backpressure_diagnostics(&mut self) -> Vec<BackpressureDiagnostic> {
        std::mem::take(&mut self.backpressure_diagnostics)
    }

    pub fn enqueue_event(&mut self, event: TelemetryEvent) {
        if self.queue.len() >= self.config.max_queue_depth {
            self.apply_backpressure_policy(&event.name);
        }
        self.queue.push_back(event);
    }

    pub fn build_batch(&mut self, max_items: usize) -> Vec<TelemetryEvent> {
        let bounded_size = max_items
            .min(self.config.max_batch_size)
            .min(CONTRACT_MAX_BATCH_ITEMS);

        let mut batch = Vec::with_capacity(bounded_size);
        for _ in 0..bounded_size {
            match self.queue.pop_front() {
                Some(event) => batch.push(event),
                None => break,
            }
        }
        batch
    }

    pub fn apply_backpressure_policy(&mut self, incoming_event_name: &str) {
        while self.queue.len() >= self.config.max_queue_depth {
            let queue_depth_before = self.queue.len();
            match self.config.backpressure_policy {
                BackpressurePolicy::DropOldest => {
                    if let Some(dropped) = self.queue.pop_front() {
                        self.backpressure_diagnostics.push(BackpressureDiagnostic {
                            dropped_event_name: dropped.name,
                            incoming_event_name: incoming_event_name.to_string(),
                            queue_depth_before,
                            queue_depth_after: self.queue.len(),
                            policy: BackpressurePolicy::DropOldest,
                        });
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

impl Default for BatchQueue {
    fn default() -> Self {
        Self::new(BatchQueueConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{BackpressurePolicy, BatchQueue, BatchQueueConfig, CONTRACT_MAX_BATCH_ITEMS};
    use crate::telemetry::TelemetryEvent;

    fn event(name: &str) -> TelemetryEvent {
        TelemetryEvent::new(name, Default::default())
    }

    #[test]
    fn queue_depth_limit_is_enforced_with_deterministic_overflow_policy() {
        let mut queue = BatchQueue::new(BatchQueueConfig {
            max_queue_depth: 2,
            max_batch_size: CONTRACT_MAX_BATCH_ITEMS,
            backpressure_policy: BackpressurePolicy::DropOldest,
        });

        queue.enqueue_event(event("event_1"));
        queue.enqueue_event(event("event_2"));
        queue.enqueue_event(event("event_3"));

        assert_eq!(queue.depth(), 2);
        let batch = queue.build_batch(2);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].name, "event_2");
        assert_eq!(batch[1].name, "event_3");

        let diagnostics = queue.take_backpressure_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].dropped_event_name, "event_1");
        assert_eq!(diagnostics[0].incoming_event_name, "event_3");
        assert_eq!(diagnostics[0].policy, BackpressurePolicy::DropOldest);
    }

    #[test]
    fn build_batch_caps_at_contract_limit() {
        let mut queue = BatchQueue::new(BatchQueueConfig {
            max_queue_depth: 500,
            max_batch_size: 500,
            backpressure_policy: BackpressurePolicy::DropOldest,
        });

        for index in 0..250 {
            queue.enqueue_event(event(&format!("event_{index}")));
        }

        let batch = queue.build_batch(500);
        assert_eq!(batch.len(), CONTRACT_MAX_BATCH_ITEMS);
        assert_eq!(queue.depth(), 50);
    }

    #[test]
    fn zero_or_oversized_config_is_safely_sanitized() {
        let mut queue = BatchQueue::new(BatchQueueConfig {
            max_queue_depth: 0,
            max_batch_size: 0,
            backpressure_policy: BackpressurePolicy::DropOldest,
        });

        queue.enqueue_event(event("a"));
        queue.enqueue_event(event("b"));

        assert_eq!(queue.depth(), 1);
        assert_eq!(queue.build_batch(1000).len(), 1);
    }
}
