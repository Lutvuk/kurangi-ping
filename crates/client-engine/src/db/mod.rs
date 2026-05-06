//! SQLite bootstrap and migration lifecycle for client engine startup.

use rusqlite::Connection;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

mod metrics_repo;
mod migrate;
mod route_session_repo;

pub use metrics_repo::{MetricSampleRecord, MetricsRepoError, SqliteMetricsRepo};
pub use migrate::{run_migrations, MigrationError, MigrationSummary};
pub use route_session_repo::{
    RouteSessionCloseRecord, RouteSessionRepoError, RouteSessionStartRecord, SqliteRouteSessionRepo,
};

#[derive(Debug)]
pub struct EngineDb {
    connection: Connection,
    pub migration_summary: MigrationSummary,
}

impl EngineDb {
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

#[derive(Debug)]
pub enum DbInitError {
    CreateDbDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    OpenConnection {
        path: PathBuf,
        source: rusqlite::Error,
    },
    EnableForeignKeys {
        source: rusqlite::Error,
    },
    ForeignKeysNotEnabled,
    MigrationFailed {
        source: MigrationError,
    },
}

impl Display for DbInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateDbDirectory { path, source } => {
                write!(
                    f,
                    "failed to create sqlite database directory {}: {}",
                    path.display(),
                    source
                )
            }
            Self::OpenConnection { path, source } => {
                write!(
                    f,
                    "failed to open sqlite database {}: {}",
                    path.display(),
                    source
                )
            }
            Self::EnableForeignKeys { source } => {
                write!(f, "failed to enable sqlite foreign keys: {source}")
            }
            Self::ForeignKeysNotEnabled => write!(
                f,
                "sqlite foreign_keys pragma was not enabled after initialization"
            ),
            Self::MigrationFailed { source } => {
                write!(f, "failed to run sqlite migrations: {source}")
            }
        }
    }
}

impl std::error::Error for DbInitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateDbDirectory { source, .. } => Some(source),
            Self::OpenConnection { source, .. } => Some(source),
            Self::EnableForeignKeys { source } => Some(source),
            Self::ForeignKeysNotEnabled => None,
            Self::MigrationFailed { source } => Some(source),
        }
    }
}

pub fn init_db(db_path: &Path, migrations_path: &Path) -> Result<EngineDb, DbInitError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).map_err(|source| DbInitError::CreateDbDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut connection =
        Connection::open(db_path).map_err(|source| DbInitError::OpenConnection {
            path: db_path.to_path_buf(),
            source,
        })?;
    enforce_foreign_keys(&connection)?;

    let migration_summary = run_migrations(&mut connection, migrations_path)
        .map_err(|source| DbInitError::MigrationFailed { source })?;

    Ok(EngineDb {
        connection,
        migration_summary,
    })
}

fn enforce_foreign_keys(conn: &Connection) -> Result<(), DbInitError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|source| DbInitError::EnableForeignKeys { source })?;

    let enabled: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .map_err(|source| DbInitError::EnableForeignKeys { source })?;

    if enabled != 1 {
        return Err(DbInitError::ForeignKeysNotEnabled);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{init_db, DbInitError};
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
    fn init_db_enables_fk_and_runs_migrations() {
        let temp_root = unique_temp_dir("kp-db-init-success");
        let db_path = temp_root.join("db.sqlite");
        let migrations_path = temp_root.join("migrations");
        fs::create_dir_all(&migrations_path).expect("failed to create migrations dir");

        fs::write(
            migrations_path.join("0001_create_parent.up.sql"),
            "CREATE TABLE IF NOT EXISTS parent(id TEXT PRIMARY KEY);",
        )
        .expect("failed to write migration 1");
        fs::write(
            migrations_path.join("0002_create_child.up.sql"),
            "CREATE TABLE IF NOT EXISTS child(id TEXT PRIMARY KEY, parent_id TEXT NOT NULL, FOREIGN KEY(parent_id) REFERENCES parent(id));",
        )
        .expect("failed to write migration 2");

        let db = init_db(&db_path, &migrations_path).expect("init_db should succeed");

        let fk_enabled: i64 = db
            .connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("failed to query PRAGMA foreign_keys");
        assert_eq!(fk_enabled, 1);
        assert_eq!(db.migration_summary.applied_count, 2);
        drop(db);

        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("failed to remove temp directory");
        }
    }

    #[test]
    fn init_db_fails_fast_when_migration_is_invalid() {
        let temp_root = unique_temp_dir("kp-db-init-failure");
        let db_path = temp_root.join("db.sqlite");
        let migrations_path = temp_root.join("migrations");
        fs::create_dir_all(&migrations_path).expect("failed to create migrations dir");

        fs::write(
            migrations_path.join("0001_invalid.up.sql"),
            "CREATE TABLE broken_sql (",
        )
        .expect("failed to write invalid migration");

        let err = init_db(&db_path, &migrations_path).expect_err("init_db should fail");
        match err {
            DbInitError::MigrationFailed { source } => {
                let message = source.to_string();
                assert!(
                    message.contains("0001_invalid.up.sql"),
                    "expected migration filename in error, got: {message}"
                );
            }
            other => panic!("expected migration failure, got: {other}"),
        }

        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("failed to remove temp directory");
        }
    }
}
