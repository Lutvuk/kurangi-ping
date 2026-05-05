use client_engine::routing::{
    acquire_toggle_lock, dedupe_toggle_command, handle_toggle_command, RoutingState,
    RoutingStateMachine, RoutingTrigger, ToggleCommand, ToggleCommandGuard, ToggleDedupeDecision,
    ToggleResultCode,
};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HarnessDecision {
    Applied,
    Deduped,
    LockRejected,
    IllegalRejected,
}

impl Display for HarnessDecision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Applied => write!(f, "applied"),
            Self::Deduped => write!(f, "deduped"),
            Self::LockRejected => write!(f, "lock_rejected"),
            Self::IllegalRejected => write!(f, "illegal_rejected"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RapidToggleHarnessResult {
    scenario: String,
    total_commands: usize,
    applied_count: usize,
    deduped_count: usize,
    lock_rejected_count: usize,
    illegal_rejected_count: usize,
    final_state: RoutingState,
    trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardContentionResult {
    scenario: String,
    attempts: usize,
    accepted: usize,
    rejected: usize,
    trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RapidToggleSuiteResult {
    sequence_stress: RapidToggleHarnessResult,
    contention_stress: GuardContentionResult,
}

fn run_rapid_toggle_suite() -> RapidToggleSuiteResult {
    RapidToggleSuiteResult {
        sequence_stress: run_sequence_stress_scenario(),
        contention_stress: run_contention_stress_scenario(),
    }
}

fn run_sequence_stress_scenario() -> RapidToggleHarnessResult {
    let base = load_toggle_sequence_fixture("alternating_burst.seq");
    let mut commands = Vec::with_capacity(base.len() * 10);
    for _ in 0..10 {
        commands.extend(base.iter().copied());
    }
    commands.push(ToggleCommand::Off);

    let guard = ToggleCommandGuard::new();
    let mut machine = RoutingStateMachine::new();

    let mut applied_count = 0_usize;
    let mut deduped_count = 0_usize;
    let mut lock_rejected_count = 0_usize;
    let mut illegal_rejected_count = 0_usize;
    let mut trace = Vec::with_capacity(commands.len() * 2);

    for (index, command) in commands.iter().copied().enumerate() {
        let state_before = machine.state();
        let dedupe = dedupe_toggle_command(command, state_before);
        if matches!(dedupe, ToggleDedupeDecision::IdempotentNoOp { .. }) {
            deduped_count = deduped_count.saturating_add(1);
            trace.push(format!(
                "{index}:{}:{}:{state_before:?}",
                command.as_str(),
                HarnessDecision::Deduped
            ));
            continue;
        }

        let Ok(_lease) = acquire_toggle_lock(&guard) else {
            lock_rejected_count = lock_rejected_count.saturating_add(1);
            trace.push(format!(
                "{index}:{}:lock_rejected:{state_before:?}",
                command.as_str()
            ));
            continue;
        };

        let toggle = handle_toggle_command(&mut machine, command);
        let decision = if toggle.code == ToggleResultCode::Applied {
            applied_count = applied_count.saturating_add(1);
            HarnessDecision::Applied
        } else {
            illegal_rejected_count = illegal_rejected_count.saturating_add(1);
            HarnessDecision::IllegalRejected
        };

        trace.push(format!(
            "{index}:{}:{}:{:?}->{:?}",
            command.as_str(),
            decision,
            state_before,
            toggle.to_state
        ));

        if decision == HarnessDecision::Applied
            && command == ToggleCommand::On
            && machine.state() == RoutingState::Connecting
        {
            machine
                .transition(RoutingTrigger::ConnectionEstablished, None)
                .expect("harness must be able to complete ON transition");
            trace.push(format!("{index}:connection_established:{:?}", machine.state()));
        }
    }

    RapidToggleHarnessResult {
        scenario: "alternating_burst_x10".to_string(),
        total_commands: commands.len(),
        applied_count,
        deduped_count,
        lock_rejected_count,
        illegal_rejected_count,
        final_state: machine.state(),
        trace,
    }
}

fn run_contention_stress_scenario() -> GuardContentionResult {
    let attempts = 16_usize;
    let guard = Arc::new(ToggleCommandGuard::new());
    let barrier = Arc::new(Barrier::new(attempts + 1));
    let outcomes = Arc::new(Mutex::new(Vec::<(usize, HarnessDecision)>::with_capacity(attempts)));

    let mut workers = Vec::with_capacity(attempts);
    for worker_id in 0..attempts {
        let guard_ref = Arc::clone(&guard);
        let barrier_ref = Arc::clone(&barrier);
        let outcomes_ref = Arc::clone(&outcomes);
        workers.push(thread::spawn(move || {
            barrier_ref.wait();
            let decision = match acquire_toggle_lock(&guard_ref) {
                Ok(_lease) => {
                    thread::sleep(Duration::from_millis(25));
                    HarnessDecision::Applied
                }
                Err(_) => HarnessDecision::LockRejected,
            };
            outcomes_ref
                .lock()
                .expect("mutex should not be poisoned")
                .push((worker_id, decision));
        }));
    }

    barrier.wait();
    for worker in workers {
        worker.join().expect("worker should not panic");
    }

    let mut rows = outcomes
        .lock()
        .expect("mutex should not be poisoned")
        .clone();
    rows.sort_by_key(|(worker_id, _)| *worker_id);

    let accepted = rows
        .iter()
        .filter(|(_, decision)| *decision == HarnessDecision::Applied)
        .count();
    let rejected = rows
        .iter()
        .filter(|(_, decision)| *decision == HarnessDecision::LockRejected)
        .count();
    let trace = vec![
        format!("attempts={attempts}"),
        format!("accepted={accepted}"),
        format!("rejected={rejected}"),
        "policy=reject_when_busy".to_string(),
    ];

    GuardContentionResult {
        scenario: "parallel_lock_contention_16".to_string(),
        attempts,
        accepted,
        rejected,
        trace,
    }
}

fn load_toggle_sequence_fixture(name: &str) -> Vec<ToggleCommand> {
    let path = fixture_path(name);
    let raw = fs::read_to_string(&path).expect("toggle fixture should be readable");
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match line {
            "ON" => ToggleCommand::On,
            "OFF" => ToggleCommand::Off,
            other => panic!("unknown toggle command in fixture: {other}"),
        })
        .collect()
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("toggle_sequences")
        .join(name)
}

#[test]
fn high_frequency_sequences_do_not_corrupt_lifecycle_state() {
    let result = run_rapid_toggle_suite().sequence_stress;

    assert!(
        result.total_commands >= 100,
        "stress sequence must be high frequency"
    );
    assert_eq!(
        result.total_commands,
        result.applied_count
            + result.deduped_count
            + result.lock_rejected_count
            + result.illegal_rejected_count
    );
    assert_eq!(result.final_state, RoutingState::Off);
    assert_eq!(result.illegal_rejected_count, 0);
    assert_eq!(result.lock_rejected_count, 0);
}

#[test]
fn command_guard_behavior_is_validated_under_contention() {
    let result = run_rapid_toggle_suite().contention_stress;

    assert_eq!(result.attempts, 16);
    assert_eq!(result.accepted, 1);
    assert_eq!(result.rejected, 15);
}

#[test]
fn terminal_states_remain_deterministic() {
    let first = run_rapid_toggle_suite();
    let second = run_rapid_toggle_suite();
    assert_eq!(first.sequence_stress.final_state, second.sequence_stress.final_state);
    assert_eq!(first.contention_stress.accepted, second.contention_stress.accepted);
    assert_eq!(first.contention_stress.rejected, second.contention_stress.rejected);
}

#[test]
fn harness_outputs_reproducible_traces() {
    let first = run_rapid_toggle_suite();
    let second = run_rapid_toggle_suite();

    assert_eq!(first.sequence_stress.trace, second.sequence_stress.trace);
    assert_eq!(first.contention_stress.trace, second.contention_stress.trace);
    assert!(
        first
            .sequence_stress
            .trace
            .iter()
            .any(|row| row.contains("connection_established")),
        "trace should include lifecycle completion markers"
    );
}
