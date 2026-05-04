use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const MIGRATION_SUFFIX: &str = ".up.sql";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationSummary {
    pub applied_count: usize,
    pub skipped_count: usize,
    pub applied_files: Vec<String>,
    pub skipped_files: Vec<String>,
}

#[derive(Debug)]
pub enum MigrationError {
    MigrationsPathNotFound { path: PathBuf },
    ReadMigrationsDirectory { path: PathBuf, source: std::io::Error },
    QueryAppliedMigrations { source: rusqlite::Error },
    ReadMigrationFile { path: PathBuf, source: std::io::Error },
    BeginTransaction { filename: String, source: rusqlite::Error },
    ApplyMigration { filename: String, source: rusqlite::Error },
    RecordMigrationVersion { filename: String, source: rusqlite::Error },
    CommitMigration { filename: String, source: rusqlite::Error },
}

impl Display for MigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MigrationsPathNotFound { path } => {
                write!(f, "migrations path not found: {}", path.display())
            }
            Self::ReadMigrationsDirectory { path, source } => {
                write!(
                    f,
                    "failed to read migrations directory {}: {}",
                    path.display(),
                    source
                )
            }
            Self::QueryAppliedMigrations { source } => {
                write!(f, "failed to query applied migrations: {source}")
            }
            Self::ReadMigrationFile { path, source } => {
                write!(f, "failed to read migration file {}: {}", path.display(), source)
            }
            Self::BeginTransaction { filename, source } => {
                write!(
                    f,
                    "failed to begin transaction for migration {filename}: {source}"
                )
            }
            Self::ApplyMigration { filename, source } => {
                write!(f, "failed to apply migration {filename}: {source}")
            }
            Self::RecordMigrationVersion { filename, source } => {
                write!(
                    f,
                    "failed to record migration version for {filename}: {source}"
                )
            }
            Self::CommitMigration { filename, source } => {
                write!(f, "failed to commit migration {filename}: {source}")
            }
        }
    }
}

impl std::error::Error for MigrationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MigrationsPathNotFound { .. } => None,
            Self::ReadMigrationsDirectory { source, .. } => Some(source),
            Self::QueryAppliedMigrations { source } => Some(source),
            Self::ReadMigrationFile { source, .. } => Some(source),
            Self::BeginTransaction { source, .. } => Some(source),
            Self::ApplyMigration { source, .. } => Some(source),
            Self::RecordMigrationVersion { source, .. } => Some(source),
            Self::CommitMigration { source, .. } => Some(source),
        }
    }
}

pub fn run_migrations(
    conn: &mut Connection,
    migrations_path: &Path,
) -> Result<MigrationSummary, MigrationError> {
    if !migrations_path.exists() {
        return Err(MigrationError::MigrationsPathNotFound {
            path: migrations_path.to_path_buf(),
        });
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|source| MigrationError::QueryAppliedMigrations { source })?;

    let applied_versions = load_applied_versions(conn)?;
    let migration_files = collect_migration_files(migrations_path)?;

    let mut summary = MigrationSummary::default();

    for file_path in migration_files {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let version = file_name.trim_end_matches(MIGRATION_SUFFIX).to_string();

        if applied_versions.contains(&version) {
            summary.skipped_count += 1;
            summary.skipped_files.push(file_name);
            continue;
        }

        let sql = fs::read_to_string(&file_path).map_err(|source| MigrationError::ReadMigrationFile {
            path: file_path.clone(),
            source,
        })?;

        let tx = conn
            .transaction()
            .map_err(|source| MigrationError::BeginTransaction {
                filename: file_name.clone(),
                source,
            })?;

        tx.execute_batch(&sql)
            .map_err(|source| MigrationError::ApplyMigration {
                filename: file_name.clone(),
                source,
            })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, filename, applied_at)
             VALUES (?1, ?2, datetime('now'))",
            params![version, file_name],
        )
        .map_err(|source| MigrationError::RecordMigrationVersion {
            filename: file_name.clone(),
            source,
        })?;
        tx.commit().map_err(|source| MigrationError::CommitMigration {
            filename: file_name.clone(),
            source,
        })?;

        summary.applied_count += 1;
        summary.applied_files.push(file_name);
    }

    summary.applied_files.sort();
    summary.skipped_files.sort();

    Ok(summary)
}

fn collect_migration_files(migrations_path: &Path) -> Result<Vec<PathBuf>, MigrationError> {
    let entries = fs::read_dir(migrations_path).map_err(|source| MigrationError::ReadMigrationsDirectory {
        path: migrations_path.to_path_buf(),
        source,
    })?;
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| MigrationError::ReadMigrationsDirectory {
            path: migrations_path.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(MIGRATION_SUFFIX))
        {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn load_applied_versions(conn: &Connection) -> Result<HashSet<String>, MigrationError> {
    let mut stmt = conn
        .prepare("SELECT version FROM schema_migrations")
        .map_err(|source| MigrationError::QueryAppliedMigrations { source })?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| MigrationError::QueryAppliedMigrations { source })?;

    let mut versions = HashSet::new();
    for row in rows {
        let version = row.map_err(|source| MigrationError::QueryAppliedMigrations { source })?;
        versions.insert(version);
    }
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::run_migrations;
    use rusqlite::Connection;
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
    fn run_migrations_is_idempotent() {
        let temp_root = unique_temp_dir("kp-db-migrate-idempotent");
        let db_path = temp_root.join("db.sqlite");
        let migrations_path = temp_root.join("migrations");
        fs::create_dir_all(&migrations_path).expect("failed to create migrations dir");

        fs::write(
            migrations_path.join("0001_create_alpha.up.sql"),
            "CREATE TABLE IF NOT EXISTS alpha(id INTEGER PRIMARY KEY, value TEXT NOT NULL);",
        )
        .expect("failed to write migration");
        fs::write(
            migrations_path.join("0002_create_beta.up.sql"),
            "CREATE TABLE IF NOT EXISTS beta(id INTEGER PRIMARY KEY, alpha_id INTEGER NOT NULL);",
        )
        .expect("failed to write migration");

        let mut conn = Connection::open(&db_path).expect("failed to open sqlite db");
        let first = run_migrations(&mut conn, &migrations_path).expect("first migration run failed");
        let second = run_migrations(&mut conn, &migrations_path).expect("second migration run failed");

        assert_eq!(first.applied_count, 2);
        assert_eq!(first.skipped_count, 0);
        assert_eq!(second.applied_count, 0);
        assert_eq!(second.skipped_count, 2);

        let applied_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
            .expect("failed to count schema_migrations");
        assert_eq!(applied_rows, 2);
        drop(conn);

        if temp_root.exists() {
            fs::remove_dir_all(&temp_root).expect("failed to remove temp directory");
        }
    }
}
