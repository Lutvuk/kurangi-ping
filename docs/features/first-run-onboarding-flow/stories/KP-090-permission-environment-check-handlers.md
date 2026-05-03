# KP-090: Permission & Environment Check Handlers

**Epic:** EPIC-ONBOARDING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement onboarding check handlers for permission, relay test readiness, and environment constraints.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/onboarding/checks.rs`
*   **Functions/Classes:** `run_permission_check()`, `run_environment_check()`, `run_relay_test_check()`
*   **API Endpoints:** N/A
*   **Data Models:** Check result model

## Acceptance Criteria (Technical)
*   [ ] Permission checks produce actionable outcome codes.
*   [ ] Environment checks identify blocking/non-blocking conditions.
*   [ ] Relay readiness checks integrate with existing relay health sources.
*   [ ] Check results feed onboarding state machine transitions.

## Business Rules & Logic
*   Early failure clarity reduces setup confusion.

## Dependencies
*   Depends on: KP-089, KP-048

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
