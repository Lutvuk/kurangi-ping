use crate::onboarding::{OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep};
use rusqlite::{params, Connection};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum OnboardingRepoError {
    SerializeFailed {
        source: serde_json::Error,
    },
    SaveFailed {
        source: rusqlite::Error,
    },
    InstallationNotFound {
        installation_id: String,
    },
    LoadFailed {
        source: rusqlite::Error,
    },
}

impl Display for OnboardingRepoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeFailed { source } => {
                write!(f, "failed to serialize onboarding checkpoint: {source}")
            }
            Self::SaveFailed { source } => write!(f, "failed to save onboarding checkpoint: {source}"),
            Self::InstallationNotFound { installation_id } => write!(
                f,
                "installation not found while saving onboarding checkpoint: {installation_id}"
            ),
            Self::LoadFailed { source } => write!(f, "failed to load onboarding checkpoint: {source}"),
        }
    }
}

impl std::error::Error for OnboardingRepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SerializeFailed { source } => Some(source),
            Self::SaveFailed { source } => Some(source),
            Self::InstallationNotFound { .. } => None,
            Self::LoadFailed { source } => Some(source),
        }
    }
}

pub fn save_onboarding_checkpoint(
    conn: &Connection,
    installation_id: &str,
    state_machine: &OnboardingStateMachine,
    updated_at: &str,
) -> Result<(), OnboardingRepoError> {
    let checkpoint_json =
        serde_json::to_string(state_machine).map_err(|source| OnboardingRepoError::SerializeFailed { source })?;

    let updated = conn
        .execute(
            r#"
            UPDATE user_settings
            SET onboarding_state = ?1, updated_at = ?2
            WHERE installation_id = ?3
            "#,
            params![checkpoint_json, updated_at, installation_id],
        )
        .map_err(|source| OnboardingRepoError::SaveFailed { source })?;

    if updated == 0 {
        return Err(OnboardingRepoError::InstallationNotFound {
            installation_id: installation_id.to_string(),
        });
    }

    Ok(())
}

pub fn load_onboarding_checkpoint(
    conn: &Connection,
    installation_id: &str,
) -> Result<OnboardingStateMachine, OnboardingRepoError> {
    let state_value = conn
        .query_row(
            "SELECT onboarding_state FROM user_settings WHERE installation_id = ?1",
            params![installation_id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|source| OnboardingRepoError::LoadFailed { source })?;

    Ok(parse_checkpoint(&state_value))
}

fn parse_checkpoint(raw: &str) -> OnboardingStateMachine {
    if let Ok(parsed) = serde_json::from_str::<OnboardingStateMachine>(raw) {
        return parsed;
    }

    // Compatibility with legacy enum-only values in older DB rows.
    match raw {
        "completed" => OnboardingStateMachine {
            state: OnboardingLifecycleState::Completed {
                completed_steps: vec![
                    OnboardingStep::Welcome,
                    OnboardingStep::PermissionCheck,
                    OnboardingStep::RelayTest,
                    OnboardingStep::GameDetectionTest,
                    OnboardingStep::FirstConnect,
                ],
            },
        },
        "in_progress" => OnboardingStateMachine {
            state: OnboardingLifecycleState::InProgress {
                current_step: OnboardingStep::Welcome,
                completed_steps: Vec::new(),
            },
        },
        "not_started" => OnboardingStateMachine::new(),
        _ => OnboardingStateMachine::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{load_onboarding_checkpoint, save_onboarding_checkpoint, OnboardingRepoError};
    use crate::onboarding::{
        transition_onboarding_state, OnboardingLifecycleState, OnboardingStateMachine, OnboardingStep,
        OnboardingTransition,
    };
    use rusqlite::Connection;

    fn setup_fixture_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            CREATE TABLE user_settings (
              installation_id TEXT PRIMARY KEY,
              preferred_region TEXT,
              auto_connect INTEGER NOT NULL CHECK (auto_connect IN (0, 1)),
              update_channel TEXT NOT NULL CHECK (update_channel IN ('beta', 'stable')),
              onboarding_state TEXT NOT NULL,
              updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z')
            );
            INSERT INTO user_settings(
              installation_id,
              preferred_region,
              auto_connect,
              update_channel,
              onboarding_state,
              updated_at
            ) VALUES(
              'inst-1',
              'auto',
              1,
              'beta',
              'not_started',
              '2026-08-01T00:00:00Z'
            );
            "#,
        )
        .expect("fixture schema should be created");
        conn
    }

    fn in_progress_machine_after_permission() -> OnboardingStateMachine {
        let start = transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
            .expect("begin should start");
        let welcome = transition_onboarding_state(
            &start,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::Welcome,
            },
        )
        .expect("welcome should complete");
        transition_onboarding_state(
            &welcome,
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::PermissionCheck,
            },
        )
        .expect("permission check should complete")
    }

    #[test]
    fn progress_checkpoint_persists_after_each_completed_step() {
        let conn = setup_fixture_db();

        let after_welcome = transition_onboarding_state(
            &transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
                .expect("begin should start"),
            OnboardingTransition::CompleteStep {
                step: OnboardingStep::Welcome,
            },
        )
        .expect("welcome should complete");

        save_onboarding_checkpoint(
            &conn,
            "inst-1",
            &after_welcome,
            "2026-08-01T00:00:01Z",
        )
        .expect("first checkpoint should persist");
        let loaded_1 = load_onboarding_checkpoint(&conn, "inst-1")
            .expect("checkpoint should load");
        match loaded_1.state {
            OnboardingLifecycleState::InProgress {
                current_step,
                completed_steps,
            } => {
                assert_eq!(current_step, OnboardingStep::PermissionCheck);
                assert_eq!(completed_steps, vec![OnboardingStep::Welcome]);
            }
            other => panic!("expected in_progress checkpoint, got: {other:?}"),
        }

        let after_permission = in_progress_machine_after_permission();
        save_onboarding_checkpoint(
            &conn,
            "inst-1",
            &after_permission,
            "2026-08-01T00:00:02Z",
        )
        .expect("second checkpoint should persist");
        let loaded_2 = load_onboarding_checkpoint(&conn, "inst-1")
            .expect("checkpoint should load");
        match loaded_2.state {
            OnboardingLifecycleState::InProgress {
                current_step,
                completed_steps,
            } => {
                assert_eq!(current_step, OnboardingStep::RelayTest);
                assert_eq!(
                    completed_steps,
                    vec![OnboardingStep::Welcome, OnboardingStep::PermissionCheck]
                );
            }
            other => panic!("expected in_progress checkpoint, got: {other:?}"),
        }
    }

    #[test]
    fn reload_restores_correct_next_step_on_restart() {
        let conn = setup_fixture_db();
        let machine = in_progress_machine_after_permission();

        save_onboarding_checkpoint(&conn, "inst-1", &machine, "2026-08-01T00:01:00Z")
            .expect("checkpoint should persist");
        let reloaded = load_onboarding_checkpoint(&conn, "inst-1")
            .expect("checkpoint should load");

        assert_eq!(reloaded, machine);
    }

    #[test]
    fn corrupt_or_unknown_state_falls_back_to_safe_restart_strategy() {
        let conn = setup_fixture_db();

        conn.execute(
            "UPDATE user_settings SET onboarding_state = 'corrupt-state' WHERE installation_id = 'inst-1'",
            [],
        )
        .expect("corrupt value should be written");

        let loaded = load_onboarding_checkpoint(&conn, "inst-1")
            .expect("corrupt checkpoint should still load with fallback");
        assert_eq!(loaded, OnboardingStateMachine::new());
    }

    #[test]
    fn persistence_failures_are_surfaced_without_crash() {
        let conn = setup_fixture_db();

        let missing_installation = save_onboarding_checkpoint(
            &conn,
            "missing-inst",
            &OnboardingStateMachine::new(),
            "2026-08-01T00:00:00Z",
        );
        assert!(matches!(
            missing_installation,
            Err(OnboardingRepoError::InstallationNotFound { .. })
        ));

        conn.execute_batch("DROP TABLE user_settings;")
            .expect("fixture table should drop");
        let load_error = load_onboarding_checkpoint(&conn, "inst-1");
        assert!(matches!(load_error, Err(OnboardingRepoError::LoadFailed { .. })));
    }
}
