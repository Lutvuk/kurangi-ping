use client_engine::db::{load_onboarding_checkpoint, save_onboarding_checkpoint};
use client_engine::onboarding::{
    apply_check_result_transition, evaluate_onboarding_completion, run_permission_check,
    run_relay_test_check, transition_onboarding_state, CheckStatus, OnboardingCompletionOutcome,
    OnboardingStateMachine, OnboardingStep, OnboardingTransition, PermissionCheckInput,
    RelayReadinessPolicy,
};
use client_engine::routing::{RelayHealthSnapshot, RelayHealthStatus, RoutingStateMachine, RoutingTrigger};
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OnboardingJourneyScenarioResult {
    scenario: String,
    completed: bool,
    checkpoint_restored: bool,
    recoverable: bool,
    emit_completed_event: bool,
    trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingJourneySuiteResult {
    happy_path: OnboardingJourneyScenarioResult,
    interruption_resume_path: OnboardingJourneyScenarioResult,
    failure_recovery_path: OnboardingJourneyScenarioResult,
}

pub fn run_onboarding_journey_suite() -> OnboardingJourneySuiteResult {
    OnboardingJourneySuiteResult {
        happy_path: run_happy_path_scenario(),
        interruption_resume_path: run_interruption_resume_scenario(),
        failure_recovery_path: run_failure_recovery_scenario(),
    }
}

fn run_happy_path_scenario() -> OnboardingJourneyScenarioResult {
    let mut trace = Vec::new();
    let machine = progress_until_first_connect("happy", &mut trace);

    let established_transition = routing_established_transition();
    let completion = evaluate_onboarding_completion(&machine, Some(&established_transition))
        .expect("happy path completion gate should evaluate");

    trace.push(format!(
        "completion_outcome={:?};reason={};emit={}",
        completion.outcome,
        completion.reason_code.as_code(),
        completion.should_emit_completed_event
    ));
    trace.push(format!(
        "terminal_onboarding_state={}",
        onboarding_state_name(&completion.next_state_machine)
    ));

    OnboardingJourneyScenarioResult {
        scenario: "happy_path".to_string(),
        completed: matches!(completion.outcome, OnboardingCompletionOutcome::Completed),
        checkpoint_restored: false,
        recoverable: true,
        emit_completed_event: completion.should_emit_completed_event,
        trace,
    }
}

fn run_interruption_resume_scenario() -> OnboardingJourneyScenarioResult {
    let mut trace = Vec::new();
    let mut machine = transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
        .expect("begin should succeed");
    trace.push(format!("state={}", onboarding_state_name(&machine)));

    machine = transition_onboarding_state(
        &machine,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::Welcome,
        },
    )
    .expect("welcome should complete");
    trace.push(format!("state={}", onboarding_state_name(&machine)));

    let permission_result = run_permission_check(PermissionCheckInput {
        has_admin_privileges: true,
        can_access_route_table: true,
    });
    machine = apply_check_result_transition(&machine, OnboardingStep::PermissionCheck, &permission_result)
        .expect("permission step should complete");
    trace.push(format!(
        "permission_status={:?};state={}",
        permission_result.status,
        onboarding_state_name(&machine)
    ));

    let conn = setup_checkpoint_fixture_db();
    save_onboarding_checkpoint(&conn, "inst-onboarding", &machine, "2026-08-01T00:00:02Z")
        .expect("checkpoint save should succeed");
    let restored = load_onboarding_checkpoint(&conn, "inst-onboarding")
        .expect("checkpoint load should succeed");
    trace.push(format!("checkpoint_restored_state={}", onboarding_state_name(&restored)));
    let checkpoint_restored = restored == machine;

    let relay_result = run_relay_test_check(
        &[RelayHealthSnapshot {
            relay_id: "sin-01".to_string(),
            status: RelayHealthStatus::Ok,
            latency_ms: 44,
        }],
        RelayReadinessPolicy::default(),
    );
    let after_relay =
        apply_check_result_transition(&restored, OnboardingStep::RelayTest, &relay_result)
            .expect("relay test should complete after resume");
    trace.push(format!(
        "relay_status={:?};state={}",
        relay_result.status,
        onboarding_state_name(&after_relay)
    ));

    let after_detection = transition_onboarding_state(
        &after_relay,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::GameDetectionTest,
        },
    )
    .expect("game detection should complete");
    trace.push(format!("state={}", onboarding_state_name(&after_detection)));

    let established_transition = routing_established_transition();
    let completion = evaluate_onboarding_completion(&after_detection, Some(&established_transition))
        .expect("resume completion gate should evaluate");
    trace.push(format!(
        "completion_outcome={:?};reason={};emit={}",
        completion.outcome,
        completion.reason_code.as_code(),
        completion.should_emit_completed_event
    ));

    OnboardingJourneyScenarioResult {
        scenario: "interruption_resume_path".to_string(),
        completed: matches!(completion.outcome, OnboardingCompletionOutcome::Completed),
        checkpoint_restored,
        recoverable: true,
        emit_completed_event: completion.should_emit_completed_event,
        trace,
    }
}

fn run_failure_recovery_scenario() -> OnboardingJourneyScenarioResult {
    let mut trace = Vec::new();
    let mut machine = transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
        .expect("begin should succeed");
    machine = transition_onboarding_state(
        &machine,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::Welcome,
        },
    )
    .expect("welcome should complete");
    trace.push(format!("state={}", onboarding_state_name(&machine)));

    let blocked_permission = run_permission_check(PermissionCheckInput {
        has_admin_privileges: false,
        can_access_route_table: false,
    });
    let blocked =
        apply_check_result_transition(&machine, OnboardingStep::PermissionCheck, &blocked_permission)
            .expect("blocked permission should transition");
    trace.push(format!(
        "permission_status={:?};state={};reason={}",
        blocked_permission.status,
        onboarding_state_name(&blocked),
        blocked_permission.primary_code.as_code()
    ));

    let resumed = transition_onboarding_state(&blocked, OnboardingTransition::ResumeFromBlocked)
        .expect("blocked state should be resumable");
    trace.push(format!("state={}", onboarding_state_name(&resumed)));

    let permission_ok = run_permission_check(PermissionCheckInput {
        has_admin_privileges: true,
        can_access_route_table: true,
    });
    let after_permission =
        apply_check_result_transition(&resumed, OnboardingStep::PermissionCheck, &permission_ok)
            .expect("permission retry should complete");
    trace.push(format!(
        "permission_retry_status={:?};state={}",
        permission_ok.status,
        onboarding_state_name(&after_permission)
    ));

    let relay_warning = run_relay_test_check(
        &[RelayHealthSnapshot {
            relay_id: "nrt-01".to_string(),
            status: RelayHealthStatus::Warn,
            latency_ms: 120,
        }],
        RelayReadinessPolicy::default(),
    );
    assert_eq!(relay_warning.status, CheckStatus::Warning);
    let after_relay =
        apply_check_result_transition(&after_permission, OnboardingStep::RelayTest, &relay_warning)
            .expect("warning relay should remain recoverable and complete");

    let after_detection = transition_onboarding_state(
        &after_relay,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::GameDetectionTest,
        },
    )
    .expect("game detection should complete");

    let failed_transition = routing_failed_transition();
    let failed_decision = evaluate_onboarding_completion(&after_detection, Some(&failed_transition))
        .expect("failed connect should evaluate");
    trace.push(format!(
        "first_connect_failure_outcome={:?};state={};emit={}",
        failed_decision.outcome,
        onboarding_state_name(&failed_decision.next_state_machine),
        failed_decision.should_emit_completed_event
    ));

    let retried = transition_onboarding_state(
        &failed_decision.next_state_machine,
        OnboardingTransition::RetryFromFailed,
    )
    .expect("failed state should be retryable");
    trace.push(format!("state={}", onboarding_state_name(&retried)));

    let established_transition = routing_established_transition();
    let completion = evaluate_onboarding_completion(&retried, Some(&established_transition))
        .expect("retry completion should evaluate");
    trace.push(format!(
        "completion_outcome={:?};state={};emit={}",
        completion.outcome,
        onboarding_state_name(&completion.next_state_machine),
        completion.should_emit_completed_event
    ));

    OnboardingJourneyScenarioResult {
        scenario: "failure_recovery_path".to_string(),
        completed: matches!(completion.outcome, OnboardingCompletionOutcome::Completed),
        checkpoint_restored: false,
        recoverable: true,
        emit_completed_event: completion.should_emit_completed_event,
        trace,
    }
}

fn progress_until_first_connect(scenario: &str, trace: &mut Vec<String>) -> OnboardingStateMachine {
    let mut machine = transition_onboarding_state(&OnboardingStateMachine::new(), OnboardingTransition::Begin)
        .expect("begin should succeed");
    trace.push(format!("{scenario}:state={}", onboarding_state_name(&machine)));

    machine = transition_onboarding_state(
        &machine,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::Welcome,
        },
    )
    .expect("welcome should complete");
    trace.push(format!("{scenario}:state={}", onboarding_state_name(&machine)));

    let permission_result = run_permission_check(PermissionCheckInput {
        has_admin_privileges: true,
        can_access_route_table: true,
    });
    machine = apply_check_result_transition(&machine, OnboardingStep::PermissionCheck, &permission_result)
        .expect("permission check should complete");
    trace.push(format!(
        "{scenario}:permission={:?};state={}",
        permission_result.status,
        onboarding_state_name(&machine)
    ));

    let relay_result = run_relay_test_check(
        &[RelayHealthSnapshot {
            relay_id: "sin-01".to_string(),
            status: RelayHealthStatus::Ok,
            latency_ms: 36,
        }],
        RelayReadinessPolicy::default(),
    );
    machine = apply_check_result_transition(&machine, OnboardingStep::RelayTest, &relay_result)
        .expect("relay check should complete");
    trace.push(format!(
        "{scenario}:relay={:?};state={}",
        relay_result.status,
        onboarding_state_name(&machine)
    ));

    machine = transition_onboarding_state(
        &machine,
        OnboardingTransition::CompleteStep {
            step: OnboardingStep::GameDetectionTest,
        },
    )
    .expect("game detection should complete");
    trace.push(format!("{scenario}:state={}", onboarding_state_name(&machine)));
    machine
}

fn routing_established_transition() -> client_engine::routing::RoutingTransition {
    let mut routing = RoutingStateMachine::new();
    routing
        .transition(RoutingTrigger::EnableRequested, None)
        .expect("off -> connecting should be legal");
    routing
        .transition(RoutingTrigger::ConnectionEstablished, None)
        .expect("connecting -> connected should be legal")
}

fn routing_failed_transition() -> client_engine::routing::RoutingTransition {
    let mut routing = RoutingStateMachine::new();
    routing
        .transition(RoutingTrigger::EnableRequested, None)
        .expect("off -> connecting should be legal");
    routing
        .transition(
            RoutingTrigger::ConnectionAttemptFailed,
            Some("ROUTE_ALL_ATTEMPTS_FAILED".to_string()),
        )
        .expect("connecting -> failed should be legal")
}

fn onboarding_state_name(machine: &OnboardingStateMachine) -> &'static str {
    match machine.state {
        client_engine::onboarding::OnboardingLifecycleState::NotStarted => "not_started",
        client_engine::onboarding::OnboardingLifecycleState::InProgress { .. } => "in_progress",
        client_engine::onboarding::OnboardingLifecycleState::Completed { .. } => "completed",
        client_engine::onboarding::OnboardingLifecycleState::Blocked { .. } => "blocked",
        client_engine::onboarding::OnboardingLifecycleState::Failed { .. } => "failed",
    }
}

fn setup_checkpoint_fixture_db() -> Connection {
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
          'inst-onboarding',
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

#[test]
fn happy_path_completes_and_sets_onboarding_completed_state() {
    let suite = run_onboarding_journey_suite();
    let happy = suite.happy_path;

    assert_eq!(happy.scenario, "happy_path");
    assert!(happy.completed);
    assert!(happy.emit_completed_event);
    assert!(
        happy.trace.iter().any(|line| line.contains("terminal_onboarding_state=completed")),
        "trace must include completed terminal state"
    );
}

#[test]
fn interruption_path_resumes_correctly() {
    let suite = run_onboarding_journey_suite();
    let interruption = suite.interruption_resume_path;

    assert_eq!(interruption.scenario, "interruption_resume_path");
    assert!(interruption.checkpoint_restored);
    assert!(interruption.completed);
    assert!(interruption.emit_completed_event);
    assert!(
        interruption
            .trace
            .iter()
            .any(|line| line.contains("checkpoint_restored_state=in_progress")),
        "trace should show resumed in_progress checkpoint"
    );
}

#[test]
fn failure_branch_paths_remain_recoverable() {
    let suite = run_onboarding_journey_suite();
    let failure = suite.failure_recovery_path;

    assert_eq!(failure.scenario, "failure_recovery_path");
    assert!(failure.recoverable);
    assert!(failure.completed);
    assert!(failure.emit_completed_event);
    assert!(
        failure
            .trace
            .iter()
            .any(|line| line.contains("first_connect_failure_outcome=Failed")),
        "trace should include failed first-connect branch before recovery"
    );
    assert!(
        failure
            .trace
            .iter()
            .any(|line| line.contains("completion_outcome=Completed")),
        "trace should include eventual successful recovery completion"
    );
}

#[test]
fn test_traces_are_deterministic_and_debuggable() {
    let first = run_onboarding_journey_suite();
    let second = run_onboarding_journey_suite();

    assert_eq!(first, second);
    assert!(!first.happy_path.trace.is_empty());
    assert!(!first.interruption_resume_path.trace.is_empty());
    assert!(!first.failure_recovery_path.trace.is_empty());

    assert!(
        first
            .happy_path
            .trace
            .iter()
            .any(|line| line.contains("completion_outcome=Completed")),
        "happy path trace should include completion checkpoint"
    );
    assert!(
        first
            .failure_recovery_path
            .trace
            .iter()
            .any(|line| line.contains("reason=permission_admin_required")),
        "recovery trace should include blocking reason code for debugging"
    );
}

