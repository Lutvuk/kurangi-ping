# KP-089: c

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
*   [x] State machine supports canonical 5-step progression.
*   [x] Invalid transitions are rejected.
*   [x] Terminal states are explicit (completed, blocked, failed).
*   [x] State model is serializable for UI sync.

## Business Rules & Logic
*   Onboarding behavior must be predictable and restart-safe.

## Dependencies
*   Depends on: KP-034

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
