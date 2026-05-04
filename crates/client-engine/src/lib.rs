//! Client engine scaffold for Kurangi Ping.
//! This crate intentionally contains no live routing side effects in foundation phase.

pub mod detection;
pub mod routing;
pub mod telemetry;

/// High-level entry point state for dependent modules.
#[derive(Debug, Clone)]
pub struct ClientEngine {
    pub detection: detection::DetectionService,
    pub routing: routing::RoutingService,
    pub telemetry: telemetry::TelemetryService,
}

/// Initializes an in-memory scaffold instance.
/// No system-level route changes are performed in this phase.
pub fn initialize_engine() -> ClientEngine {
    ClientEngine {
        detection: detection::DetectionService::new(),
        routing: routing::RoutingService::new(),
        telemetry: telemetry::TelemetryService::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::initialize_engine;

    #[test]
    fn initialize_engine_builds_default_services() {
        let engine = initialize_engine();
        assert_eq!(engine.detection.status(), "idle");
        assert_eq!(engine.routing.state(), "disabled");
        assert_eq!(engine.telemetry.queue_depth(), 0);
    }
}
