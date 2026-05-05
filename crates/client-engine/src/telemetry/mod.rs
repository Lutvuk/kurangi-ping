//! Telemetry batching boundary for privacy-safe event delivery.

pub mod events;

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum TelemetryValue {
    Text(String),
    Integer(i64),
    Float(f64),
}

impl PartialEq for TelemetryValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(left), Self::Text(right)) => left == right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left.to_bits() == right.to_bits(),
            _ => false,
        }
    }
}

impl Eq for TelemetryValue {}

pub type TelemetryPayload = BTreeMap<String, TelemetryValue>;

/// Lightweight telemetry event placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryEvent {
    pub name: String,
    pub payload: TelemetryPayload,
}

impl TelemetryEvent {
    pub fn new(name: impl Into<String>, payload: TelemetryPayload) -> Self {
        Self {
            name: name.into(),
            payload,
        }
    }
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
