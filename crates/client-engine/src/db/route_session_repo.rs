use rusqlite::{params, Connection};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSessionStartRecord {
    pub session_id: String,
    pub installation_id: String,
    pub game_id: String,
    pub relay_id: Option<String>,
    pub route_protocol: String,
    pub started_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSessionCloseRecord {
    pub session_id: String,
    pub ended_at: String,
    pub end_reason: Option<String>,
}

#[derive(Debug)]
pub enum RouteSessionRepoError {
    InsertFailed { source: rusqlite::Error },
    UpdateFailed { source: rusqlite::Error },
    SessionNotFound { session_id: String },
}

impl Display for RouteSessionRepoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsertFailed { source } => write!(f, "failed to insert route session: {source}"),
            Self::UpdateFailed { source } => write!(f, "failed to update route session: {source}"),
            Self::SessionNotFound { session_id } => {
                write!(f, "route session not found for close hook: {session_id}")
            }
        }
    }
}

impl std::error::Error for RouteSessionRepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InsertFailed { source } => Some(source),
            Self::UpdateFailed { source } => Some(source),
            Self::SessionNotFound { .. } => None,
        }
    }
}

pub struct SqliteRouteSessionRepo<'conn> {
    conn: &'conn Connection,
}

impl<'conn> SqliteRouteSessionRepo<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn start_route_session(
        &self,
        record: &RouteSessionStartRecord,
    ) -> Result<(), RouteSessionRepoError> {
        self.conn
            .execute(
                r#"
                INSERT INTO route_session(
                  session_id,
                  installation_id,
                  game_id,
                  relay_id,
                  route_protocol,
                  started_at,
                  ended_at,
                  end_reason
                ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL)
                "#,
                params![
                    record.session_id,
                    record.installation_id,
                    record.game_id,
                    record.relay_id,
                    record.route_protocol,
                    record.started_at
                ],
            )
            .map_err(|source| RouteSessionRepoError::InsertFailed { source })?;

        Ok(())
    }

    pub fn close_route_session(
        &self,
        record: &RouteSessionCloseRecord,
    ) -> Result<(), RouteSessionRepoError> {
        let updated = self
            .conn
            .execute(
                r#"
                UPDATE route_session
                SET ended_at = ?1, end_reason = ?2
                WHERE session_id = ?3
                "#,
                params![record.ended_at, record.end_reason, record.session_id],
            )
            .map_err(|source| RouteSessionRepoError::UpdateFailed { source })?;

        if updated == 0 {
            return Err(RouteSessionRepoError::SessionNotFound {
                session_id: record.session_id.clone(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{RouteSessionCloseRecord, RouteSessionStartRecord, SqliteRouteSessionRepo};
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

            INSERT INTO user_settings(installation_id) VALUES ('inst-1');
            INSERT INTO supported_games(game_id) VALUES ('ffxiv');
            INSERT INTO relay_node(relay_id) VALUES ('sin-01');
            "#,
        )
        .expect("fixture schema should be created");
        conn
    }

    #[test]
    fn start_and_close_persist_timestamps_and_reason() {
        let conn = setup_fixture_db();
        let repo = SqliteRouteSessionRepo::new(&conn);

        repo.start_route_session(&RouteSessionStartRecord {
            session_id: "sess-1".to_string(),
            installation_id: "inst-1".to_string(),
            game_id: "ffxiv".to_string(),
            relay_id: Some("sin-01".to_string()),
            route_protocol: "wireguard".to_string(),
            started_at: "2026-08-01T00:00:00Z".to_string(),
        })
        .expect("start hook should persist");

        repo.close_route_session(&RouteSessionCloseRecord {
            session_id: "sess-1".to_string(),
            ended_at: "2026-08-01T00:00:03Z".to_string(),
            end_reason: Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
        })
        .expect("close hook should persist");

        let (started_at, ended_at, end_reason): (String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT started_at, ended_at, end_reason FROM route_session WHERE session_id = 'sess-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("persisted route_session row should exist");

        assert_eq!(started_at, "2026-08-01T00:00:00Z");
        assert_eq!(ended_at.as_deref(), Some("2026-08-01T00:00:03Z"));
        assert_eq!(end_reason.as_deref(), Some("ROUTE_ALL_ATTEMPTS_FAILED"));
    }
}

