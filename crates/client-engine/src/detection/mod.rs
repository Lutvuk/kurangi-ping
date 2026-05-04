//! Detection boundary for supported game process discovery.

pub mod commands;
pub mod scanner_windows;
pub mod state_resolver;

/// Represents detection lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionStatus {
    Idle,
    Scanning,
    Detected,
}

/// Public detection service interface for higher layers.
#[derive(Debug, Clone)]
pub struct DetectionService {
    status: DetectionStatus,
}

impl DetectionService {
    pub fn new() -> Self {
        Self {
            status: DetectionStatus::Idle,
        }
    }

    pub fn status(&self) -> &'static str {
        match self.status {
            DetectionStatus::Idle => "idle",
            DetectionStatus::Scanning => "scanning",
            DetectionStatus::Detected => "detected",
        }
    }

    pub fn request_scan(&mut self) {
        // TODO(KP-040/KP-041): implement Windows process scanner and freshness window.
        self.status = DetectionStatus::Scanning;
    }
}

impl Default for DetectionService {
    fn default() -> Self {
        Self::new()
    }
}
