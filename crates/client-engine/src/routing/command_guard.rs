//! Concurrency and idempotency guard for rapid toggle commands.

use super::{RoutingState, ToggleCommand};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleLockPolicy {
    RejectWhenBusy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleLockErrorCode {
    Busy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleLockError {
    pub code: ToggleLockErrorCode,
    pub message: &'static str,
}

#[derive(Debug, Clone)]
pub struct ToggleCommandGuard {
    lock_flag: Arc<AtomicBool>,
    policy: ToggleLockPolicy,
}

impl ToggleCommandGuard {
    pub fn new() -> Self {
        Self {
            lock_flag: Arc::new(AtomicBool::new(false)),
            policy: ToggleLockPolicy::RejectWhenBusy,
        }
    }

    pub fn with_policy(policy: ToggleLockPolicy) -> Self {
        Self {
            lock_flag: Arc::new(AtomicBool::new(false)),
            policy,
        }
    }

    pub fn policy(&self) -> ToggleLockPolicy {
        self.policy
    }

    pub fn is_locked(&self) -> bool {
        self.lock_flag.load(Ordering::SeqCst)
    }
}

impl Default for ToggleCommandGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ToggleLockLease {
    lock_flag: Arc<AtomicBool>,
}

impl Drop for ToggleLockLease {
    fn drop(&mut self) {
        self.lock_flag.store(false, Ordering::SeqCst);
    }
}

pub fn acquire_toggle_lock(guard: &ToggleCommandGuard) -> Result<ToggleLockLease, ToggleLockError> {
    match guard.policy {
        ToggleLockPolicy::RejectWhenBusy => {
            let acquired = guard
                .lock_flag
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok();
            if acquired {
                Ok(ToggleLockLease {
                    lock_flag: guard.lock_flag.clone(),
                })
            } else {
                Err(ToggleLockError {
                    code: ToggleLockErrorCode::Busy,
                    message: "toggle command already in progress",
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleDedupeReasonCode {
    AlreadyOn,
    AlreadyOff,
}

impl ToggleDedupeReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AlreadyOn => "toggle_already_on",
            Self::AlreadyOff => "toggle_already_off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleDedupeDecision {
    Execute,
    IdempotentNoOp {
        reason_code: ToggleDedupeReasonCode,
    },
}

pub fn dedupe_toggle_command(command: ToggleCommand, state: RoutingState) -> ToggleDedupeDecision {
    match (command, state) {
        (ToggleCommand::On, RoutingState::Connecting)
        | (ToggleCommand::On, RoutingState::Connected)
        | (ToggleCommand::On, RoutingState::Degraded) => ToggleDedupeDecision::IdempotentNoOp {
            reason_code: ToggleDedupeReasonCode::AlreadyOn,
        },
        (ToggleCommand::Off, RoutingState::Off) => ToggleDedupeDecision::IdempotentNoOp {
            reason_code: ToggleDedupeReasonCode::AlreadyOff,
        },
        _ => ToggleDedupeDecision::Execute,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        acquire_toggle_lock, dedupe_toggle_command, ToggleCommandGuard, ToggleDedupeDecision,
        ToggleDedupeReasonCode, ToggleLockErrorCode,
    };
    use crate::routing::{RoutingState, ToggleCommand};
    use std::panic::AssertUnwindSafe;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn concurrent_commands_are_rejected_by_policy_when_lock_is_busy() {
        let guard = Arc::new(ToggleCommandGuard::new());
        let gate = Arc::new(Barrier::new(3));

        let mut workers = Vec::new();
        for _ in 0..2 {
            let guard_ref = guard.clone();
            let gate_ref = gate.clone();
            workers.push(thread::spawn(move || {
                gate_ref.wait();
                let acquired = acquire_toggle_lock(&guard_ref);
                if let Ok(_lease) = acquired {
                    thread::sleep(Duration::from_millis(30));
                    true
                } else {
                    false
                }
            }));
        }

        gate.wait();
        let outcomes = workers
            .into_iter()
            .map(|worker| worker.join().expect("worker should complete"))
            .collect::<Vec<_>>();

        assert_eq!(outcomes.iter().filter(|ok| **ok).count(), 1);
        assert_eq!(outcomes.iter().filter(|ok| !**ok).count(), 1);
    }

    #[test]
    fn duplicate_commands_are_idempotent() {
        assert_eq!(
            dedupe_toggle_command(ToggleCommand::On, RoutingState::Connected),
            ToggleDedupeDecision::IdempotentNoOp {
                reason_code: ToggleDedupeReasonCode::AlreadyOn
            }
        );
        assert_eq!(
            dedupe_toggle_command(ToggleCommand::Off, RoutingState::Off),
            ToggleDedupeDecision::IdempotentNoOp {
                reason_code: ToggleDedupeReasonCode::AlreadyOff
            }
        );
        assert_eq!(
            dedupe_toggle_command(ToggleCommand::On, RoutingState::Failed),
            ToggleDedupeDecision::Execute
        );
    }

    #[test]
    fn guard_release_is_guaranteed_after_success_and_failure_paths() {
        let guard = ToggleCommandGuard::new();

        {
            let _lease = acquire_toggle_lock(&guard).expect("first lock should succeed");
            assert!(guard.is_locked());
        }
        assert!(!guard.is_locked());

        let busy_error = {
            let _lease = acquire_toggle_lock(&guard).expect("second lock should succeed");
            acquire_toggle_lock(&guard).expect_err("nested lock should be rejected")
        };
        assert_eq!(busy_error.code, ToggleLockErrorCode::Busy);
        assert!(!guard.is_locked());
    }

    #[test]
    fn lock_is_released_even_if_handler_panics() {
        let guard = ToggleCommandGuard::new();

        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let _lease = acquire_toggle_lock(&guard).expect("lock should be acquired");
            panic!("simulated panic in command path");
        }));

        assert!(!guard.is_locked());
        let _lease = acquire_toggle_lock(&guard).expect("lock should be reacquirable");
    }
}
