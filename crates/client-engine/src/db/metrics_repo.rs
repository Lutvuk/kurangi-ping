use rusqlite::{params, Connection};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct MetricSampleRecord {
    pub sample_id: String,
    pub session_id: String,
    pub baseline_ping_ms: f64,
    pub routed_ping_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub sampled_at: String,
}

#[derive(Debug)]
pub enum MetricsRepoError {
    InsertFailed { source: rusqlite::Error },
    PruneFailed { source: rusqlite::Error },
    QueryFailed { source: rusqlite::Error },
}

impl Display for MetricsRepoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsertFailed { source } => write!(f, "failed to insert ping sample: {source}"),
            Self::PruneFailed { source } => write!(f, "failed to prune ping samples: {source}"),
            Self::QueryFailed { source } => write!(f, "failed to query ping samples: {source}"),
        }
    }
}

impl std::error::Error for MetricsRepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InsertFailed { source } => Some(source),
            Self::PruneFailed { source } => Some(source),
            Self::QueryFailed { source } => Some(source),
        }
    }
}

pub struct SqliteMetricsRepo<'conn> {
    conn: &'conn Connection,
}

impl<'conn> SqliteMetricsRepo<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn append_metric_sample(
        &self,
        record: &MetricSampleRecord,
    ) -> Result<(), MetricsRepoError> {
        self.conn
            .execute(
                r#"
                INSERT INTO ping_sample(
                  sample_id,
                  session_id,
                  baseline_ping_ms,
                  routed_ping_ms,
                  jitter_ms,
                  packet_loss_pct,
                  sampled_at
                ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    record.sample_id,
                    record.session_id,
                    record.baseline_ping_ms,
                    record.routed_ping_ms,
                    record.jitter_ms,
                    record.packet_loss_pct,
                    record.sampled_at
                ],
            )
            .map_err(|source| MetricsRepoError::InsertFailed { source })?;

        Ok(())
    }

    pub fn prune_old_samples(
        &self,
        session_id: &str,
        retention_limit: usize,
    ) -> Result<usize, MetricsRepoError> {
        let retention_limit_i64 =
            i64::try_from(retention_limit).map_err(|_| MetricsRepoError::PruneFailed {
                source: rusqlite::Error::InvalidQuery,
            })?;

        let deleted = if retention_limit == 0 {
            self.conn
                .execute(
                    "DELETE FROM ping_sample WHERE session_id = ?1",
                    params![session_id],
                )
                .map_err(|source| MetricsRepoError::PruneFailed { source })?
        } else {
            self.conn
                .execute(
                    r#"
                    DELETE FROM ping_sample
                    WHERE sample_id IN (
                      SELECT sample_id
                      FROM ping_sample
                      WHERE session_id = ?1
                      ORDER BY sampled_at DESC, sample_id DESC
                      LIMIT -1 OFFSET ?2
                    )
                    "#,
                    params![session_id, retention_limit_i64],
                )
                .map_err(|source| MetricsRepoError::PruneFailed { source })?
        };

        Ok(deleted)
    }

    pub fn recent_metric_samples(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<MetricSampleRecord>, MetricsRepoError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let limit_i64 = i64::try_from(limit).map_err(|_| MetricsRepoError::QueryFailed {
            source: rusqlite::Error::InvalidQuery,
        })?;

        let mut statement = self
            .conn
            .prepare(
                r#"
                SELECT
                  sample_id,
                  session_id,
                  baseline_ping_ms,
                  routed_ping_ms,
                  jitter_ms,
                  packet_loss_pct,
                  sampled_at
                FROM ping_sample
                WHERE session_id = ?1
                ORDER BY sampled_at DESC, sample_id DESC
                LIMIT ?2
                "#,
            )
            .map_err(|source| MetricsRepoError::QueryFailed { source })?;

        let rows = statement
            .query_map(params![session_id, limit_i64], |row| {
                Ok(MetricSampleRecord {
                    sample_id: row.get(0)?,
                    session_id: row.get(1)?,
                    baseline_ping_ms: row.get(2)?,
                    routed_ping_ms: row.get(3)?,
                    jitter_ms: row.get(4)?,
                    packet_loss_pct: row.get(5)?,
                    sampled_at: row.get(6)?,
                })
            })
            .map_err(|source| MetricsRepoError::QueryFailed { source })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|source| MetricsRepoError::QueryFailed { source })
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricSampleRecord, MetricsRepoError, SqliteMetricsRepo};
    use rusqlite::Connection;

    fn setup_fixture_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE user_settings (
              installation_id TEXT PRIMARY KEY
            );
            CREATE TABLE supported_games (
              game_id TEXT PRIMARY KEY
            );
            CREATE TABLE relay_node (
              relay_id TEXT PRIMARY KEY
            );
            CREATE TABLE route_session (
              session_id TEXT PRIMARY KEY,
              installation_id TEXT NOT NULL,
              game_id TEXT NOT NULL,
              relay_id TEXT,
              route_protocol TEXT NOT NULL CHECK (route_protocol IN ('wireguard', 'tcp_tls', 'quic')),
              started_at TEXT NOT NULL CHECK (datetime(started_at) IS NOT NULL AND started_at GLOB '????-??-??T??:??:??*Z'),
              ended_at TEXT CHECK (
                ended_at IS NULL
                OR (
                  datetime(ended_at) IS NOT NULL
                  AND ended_at GLOB '????-??-??T??:??:??*Z'
                  AND datetime(ended_at) >= datetime(started_at)
                )
              ),
              end_reason TEXT,
              FOREIGN KEY (installation_id) REFERENCES user_settings(installation_id),
              FOREIGN KEY (game_id) REFERENCES supported_games(game_id),
              FOREIGN KEY (relay_id) REFERENCES relay_node(relay_id)
            );
            CREATE TABLE ping_sample (
              sample_id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              baseline_ping_ms REAL NOT NULL CHECK (baseline_ping_ms >= 0),
              routed_ping_ms REAL CHECK (routed_ping_ms IS NULL OR routed_ping_ms >= 0),
              jitter_ms REAL CHECK (jitter_ms IS NULL OR jitter_ms >= 0),
              packet_loss_pct REAL CHECK (packet_loss_pct IS NULL OR (packet_loss_pct >= 0 AND packet_loss_pct <= 100)),
              sampled_at TEXT NOT NULL CHECK (datetime(sampled_at) IS NOT NULL AND sampled_at GLOB '????-??-??T??:??:??*Z'),
              FOREIGN KEY (session_id) REFERENCES route_session(session_id)
            );

            INSERT INTO user_settings(installation_id) VALUES ('inst-1');
            INSERT INTO supported_games(game_id) VALUES ('ffxiv');
            INSERT INTO relay_node(relay_id) VALUES ('sin-01');
            INSERT INTO route_session(session_id, installation_id, game_id, relay_id, route_protocol, started_at)
            VALUES ('sess-1', 'inst-1', 'ffxiv', 'sin-01', 'wireguard', '2026-08-01T00:00:00Z');
            "#,
        )
        .expect("fixture schema should be created");
        conn
    }

    fn sample(index: usize) -> MetricSampleRecord {
        MetricSampleRecord {
            sample_id: format!("sample-{index}"),
            session_id: "sess-1".to_string(),
            baseline_ping_ms: 210.0 + index as f64,
            routed_ping_ms: Some(160.0 + index as f64),
            jitter_ms: Some(4.0),
            packet_loss_pct: Some(0.0),
            sampled_at: format!("2026-08-01T00:00:{:02}Z", index),
        }
    }

    #[test]
    fn samples_persist_with_session_context_and_timestamp() {
        let conn = setup_fixture_db();
        let repo = SqliteMetricsRepo::new(&conn);

        let record = sample(1);
        repo.append_metric_sample(&record)
            .expect("sample append should persist");

        let stored: (String, String, f64) = conn
            .query_row(
                "SELECT session_id, sampled_at, baseline_ping_ms FROM ping_sample WHERE sample_id = 'sample-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("persisted sample should exist");

        assert_eq!(stored.0, "sess-1");
        assert_eq!(stored.1, "2026-08-01T00:00:01Z");
        assert_eq!(stored.2, 211.0);
    }

    #[test]
    fn retention_cap_enforces_oldest_first_eviction() {
        let conn = setup_fixture_db();
        let repo = SqliteMetricsRepo::new(&conn);

        for index in 0..5 {
            repo.append_metric_sample(&sample(index))
                .expect("sample append should persist");
        }

        let deleted = repo
            .prune_old_samples("sess-1", 3)
            .expect("prune should complete");
        assert_eq!(deleted, 2);

        let remaining = repo
            .recent_metric_samples("sess-1", 10)
            .expect("recent query should succeed");
        let remaining_ids = remaining
            .iter()
            .map(|row| row.sample_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(remaining_ids, vec!["sample-4", "sample-3", "sample-2"]);
    }

    #[test]
    fn persistence_failure_is_returned_without_panicking_probe_path() {
        let conn = setup_fixture_db();
        let repo = SqliteMetricsRepo::new(&conn);

        let mut invalid = sample(1);
        invalid.sample_id = "bad".to_string();
        invalid.session_id = "missing-session".to_string();
        invalid.baseline_ping_ms = -1.0;

        let append_error = repo.append_metric_sample(&invalid);
        assert!(matches!(
            append_error,
            Err(MetricsRepoError::InsertFailed { .. })
        ));

        let follow_up = repo.prune_old_samples("sess-1", 10);
        assert!(
            follow_up.is_ok(),
            "repo remains callable after insert error"
        );
    }

    #[test]
    fn query_helper_supports_recent_trend_retrieval() {
        let conn = setup_fixture_db();
        let repo = SqliteMetricsRepo::new(&conn);

        for index in 0..4 {
            repo.append_metric_sample(&sample(index))
                .expect("sample append should persist");
        }

        let trend = repo
            .recent_metric_samples("sess-1", 2)
            .expect("trend query should succeed");

        assert_eq!(trend.len(), 2);
        assert_eq!(trend[0].sample_id, "sample-3");
        assert_eq!(trend[1].sample_id, "sample-2");
    }
}
