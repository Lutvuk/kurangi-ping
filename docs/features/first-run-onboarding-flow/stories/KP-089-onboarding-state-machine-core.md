# KP-089: Onboarding State Machine Core

**Epic:** EPIC-ONBOARDING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement deterministic onboarding state machine across defined setup steps.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/onboarding/state_machine.rs`
*   **Functions/Classes:** `OnboardingStateMachine`, `transition_onboarding_state()`
*   **API Endpoints:** N/A
*   **Data Models:** Onboarding step/state model

## Acceptance Criteria (Technical)
*   [ ] State machine supports canonical 5-step progression.
*   [ ] Invalid transitions are rejected.
*   [ ] Terminal states are explicit (completed, blocked, failed).
*   [ ] State model is serializable for UI sync.

## Business Rules & Logic
*   Onboarding behavior must be predictable and restart-safe.

## Dependencies
*   Depends on: KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
