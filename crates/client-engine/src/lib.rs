//! Client engine scaffold for Kurangi Ping.
//! This crate intentionally contains no live routing side effects in foundation phase.

use std::path::{Path, PathBuf};

pub mod db;
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

/// Startup configuration for engine + DB bootstrap.
#[derive(Debug, Clone)]
pub struct EngineStartupConfig {
    pub db_path: PathBuf,
    pub migrations_path: PathBuf,
}

impl EngineStartupConfig {
    pub fn new(db_path: impl AsRef<Path>, migrations_path: impl AsRef<Path>) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            migrations_path: migrations_path.as_ref().to_path_buf(),
        }
    }
}

/// Initializes DB lifecycle first, then builds the in-memory engine services.
pub fn initialize_engine_with_db(
    config: &EngineStartupConfig,
) -> Result<(ClientEngine, db::EngineDb), db::DbInitError> {
    let engine_db = db::init_db(&config.db_path, &config.migrations_path)?;
    let engine = initialize_engine();
    Ok((engine, engine_db))
}

#[cfg(test)]
mod tests {
    use super::{initialize_engine, initialize_engine_with_db, EngineStartupConfig};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock moved backwards")
            .as_nanos();
        path.push(format!("{}-{}-{}", prefix, std::process::id(), nanos));
        fs::create_dir_all(&path).expect("failed to create temp directory");
        path
    }

    #[test]
    fn initialize_engine_builds_default_services() {
        let engine = initialize_engine();
        assert_eq!(engine.detection.status(), "idle");
        assert_eq!(engine.routing.state(), "disabled");
        assert_eq!(engine.telemetry.queue_depth(), 0);
    }

    #[test]
    fn initialize_engine_with_db_runs_bootstrap_path() {
        let temp_root = unique_temp_dir("kp-engine-db-startup");
        let db_path = temp_root.join("client.sqlite");
        let migrations_path = temp_root.join("migrations");
        fs::create_dir_all(&migrations_path).expect("failed to create migrations dir");
        fs::write(
            migrations_path.join("0001_create_probe.up.sql"),
            "CREATE TABLE IF NOT EXISTS probe(id INTEGER PRIMARY KEY);",
        )
        .expect("failed to write migration");

        let config = EngineStartupConfig::new(&db_path, &migrations_path);
        let (engine, engine_db) =
            initialize_engine_with_db(&config).expect("db startup initialization should succeed");

        assert_eq!(engine.detection.status(), "idle");
        assert_eq!(engine_db.migration_summary.applied_count, 1);
        drop(engine_db);

        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("failed to remove temp test directory");
        }
    }
}
