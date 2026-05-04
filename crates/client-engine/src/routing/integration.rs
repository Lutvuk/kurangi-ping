//! Integration hooks between routing lifecycle and session persistence.

use super::{IllegalTransitionError, RoutingState, RoutingStateMachine, RoutingTransition, RoutingTrigger};
use crate::db::{RouteSessionCloseRecord, RouteSessionStartRecord};

pub trait RouteSessionPersistence {
    fn start_route_session(&self, record: &RouteSessionStartRecord) -> Result<(), String>;
    fn close_route_session(&self, record: &RouteSessionCloseRecord) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionHookStatus {
    Persisted,
    PersistenceFailed { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionHookResult {
    pub operation: &'static str,
    pub status: SessionHookStatus,
}

pub fn start_route_session<P: RouteSessionPersistence>(
    persistence: &P,
    record: &RouteSessionStartRecord,
) -> SessionHookResult {
    match persistence.start_route_session(record) {
        Ok(()) => SessionHookResult {
            operation: "start_route_session",
            status: SessionHookStatus::Persisted,
        },
        Err(message) => SessionHookResult {
            operation: "start_route_session",
            status: SessionHookStatus::PersistenceFailed { message },
        },
    }
}

pub fn close_route_session<P: RouteSessionPersistence>(
    persistence: &P,
    record: &RouteSessionCloseRecord,
) -> SessionHookResult {
    match persistence.close_route_session(record) {
        Ok(()) => SessionHookResult {
            operation: "close_route_session",
            status: SessionHookStatus::Persisted,
        },
        Err(message) => SessionHookResult {
            operation: "close_route_session",
            status: SessionHookStatus::PersistenceFailed { message },
        },
    }
}

pub fn transition_with_session_hooks<P: RouteSessionPersistence>(
    machine: &mut RoutingStateMachine,
    trigger: RoutingTrigger,
    failure_code: Option<String>,
    persistence: &P,
    start_record: Option<&RouteSessionStartRecord>,
    close_record: Option<&RouteSessionCloseRecord>,
) -> Result<(RoutingTransition, Vec<SessionHookResult>), IllegalTransitionError> {
    let transition = machine.transition(trigger, failure_code)?;
    let mut hooks = Vec::new();

    if transition.from == RoutingState::Off && transition.to == RoutingState::Connecting {
        if let Some(record) = start_record {
            hooks.push(start_route_session(persistence, record));
        }
    }

    if matches!(transition.to, RoutingState::Failed | RoutingState::Off) {
        if let Some(record) = close_record {
            hooks.push(close_route_session(persistence, record));
        }
    }

    Ok((transition, hooks))
}

#[cfg(test)]
mod tests {
    use super::{
        close_route_session, start_route_session, transition_with_session_hooks, RouteSessionPersistence,
        SessionHookStatus,
    };
    use crate::db::{RouteSessionCloseRecord, RouteSessionStartRecord, SqliteRouteSessionRepo};
    use crate::routing::{RoutingStateMachine, RoutingTrigger};
    use rusqlite::Connection;
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockSessionPersistence {
        fail_start: bool,
        fail_close: bool,
        calls: RefCell<Vec<String>>,
    }

    impl RouteSessionPersistence for MockSessionPersistence {
        fn start_route_session(&self, _record: &RouteSessionStartRecord) -> Result<(), String> {
            self.calls.borrow_mut().push("start".to_string());
            if self.fail_start {
                Err("start failed".to_string())
            } else {
                Ok(())
            }
        }

        fn close_route_session(&self, _record: &RouteSessionCloseRecord) -> Result<(), String> {
            self.calls.borrow_mut().push("close".to_string());
            if self.fail_close {
                Err("close failed".to_string())
            } else {
                Ok(())
            }
        }
    }

    fn sample_start_record() -> RouteSessionStartRecord {
        RouteSessionStartRecord {
            session_id: "sess-1".to_string(),
            installation_id: "inst-1".to_string(),
            game_id: "ffxiv".to_string(),
            relay_id: Some("sin-01".to_string()),
            route_protocol: "wireguard".to_string(),
            started_at: "2026-08-01T00:00:00Z".to_string(),
        }
    }

    fn sample_close_record() -> RouteSessionCloseRecord {
        RouteSessionCloseRecord {
            session_id: "sess-1".to_string(),
            ended_at: "2026-08-01T00:00:03Z".to_string(),
            end_reason: Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
        }
    }

    #[test]
    fn persistence_failures_do_not_crash_lifecycle_flow() {
        let persistence = MockSessionPersistence {
            fail_start: true,
            fail_close: true,
            calls: RefCell::new(Vec::new()),
        };

        let start_result = start_route_session(&persistence, &sample_start_record());
        assert!(matches!(
            start_result.status,
            SessionHookStatus::PersistenceFailed { .. }
        ));

        let close_result = close_route_session(&persistence, &sample_close_record());
        assert!(matches!(
            close_result.status,
            SessionHookStatus::PersistenceFailed { .. }
        ));
    }

    #[test]
    fn state_machine_integration_triggers_expected_hooks() {
        let persistence = MockSessionPersistence::default();
        let mut machine = RoutingStateMachine::new();

        let (_transition, hooks) = transition_with_session_hooks(
            &mut machine,
            RoutingTrigger::EnableRequested,
            None,
            &persistence,
            Some(&sample_start_record()),
            None,
        )
        .expect("enable transition should be legal");

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].operation, "start_route_session");

        let (_transition, hooks) = transition_with_session_hooks(
            &mut machine,
            RoutingTrigger::ConnectionAttemptFailed,
            Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
            &persistence,
            None,
            Some(&sample_close_record()),
        )
        .expect("failed transition should be legal");

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].operation, "close_route_session");
    }

    #[test]
    fn integration_path_is_covered_with_sqlite_fixture() {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE user_settings (installation_id TEXT PRIMARY KEY);
            CREATE TABLE supported_games (game_id TEXT PRIMARY KEY);
            CREATE TABLE relay_node (relay_id TEXT PRIMARY KEY);
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

        struct RepoAdapter<'a> {
            repo: SqliteRouteSessionRepo<'a>,
        }

        impl<'a> RouteSessionPersistence for RepoAdapter<'a> {
            fn start_route_session(&self, record: &RouteSessionStartRecord) -> Result<(), String> {
                self.repo
                    .start_route_session(record)
                    .map_err(|err| err.to_string())
            }

            fn close_route_session(&self, record: &RouteSessionCloseRecord) -> Result<(), String> {
                self.repo
                    .close_route_session(record)
                    .map_err(|err| err.to_string())
            }
        }

        let adapter = RepoAdapter {
            repo: SqliteRouteSessionRepo::new(&conn),
        };
        let mut machine = RoutingStateMachine::new();

        transition_with_session_hooks(
            &mut machine,
            RoutingTrigger::EnableRequested,
            None,
            &adapter,
            Some(&sample_start_record()),
            None,
        )
        .expect("enable transition should persist start record");

        transition_with_session_hooks(
            &mut machine,
            RoutingTrigger::ConnectionAttemptFailed,
            Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
            &adapter,
            None,
            Some(&sample_close_record()),
        )
        .expect("failed transition should persist close record");

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

