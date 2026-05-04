//! Telemetry batching boundary for privacy-safe event delivery.

pub mod events;

/// Lightweight telemetry event placeholder.
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub name: String,
}

/// Public batching interface used by upper orchestration layers.
#[derive(Debug, Clone)]
pub struct TelemetryService {
    queue: Vec<TelemetryEvent>,
}

impl TelemetryService {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    pub fn enqueue(&mut self, event: TelemetryEvent) {
        // TODO(KP-080/KP-082): apply schema allowlist and bounded queue policy.
        self.queue.push(event);
    }

    pub fn drain_batch(&mut self, max_items: usize) -> Vec<TelemetryEvent> {
        // TODO(KP-083/KP-084): replace with retry-aware batch lifecycle.
        let take = max_items.min(self.queue.len());
        self.queue.drain(0..take).collect()
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}
