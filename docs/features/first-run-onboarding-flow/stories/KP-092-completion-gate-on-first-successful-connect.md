# KP-092: Completion Gate on First Successful Connect

**Epic:** EPIC-ONBOARDING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Mark onboarding complete only after first successful route activation.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/onboarding/completion_gate.rs`
*   **Functions/Classes:** `evaluate_onboarding_completion()`
*   **API Endpoints:** N/A
*   **Data Models:** Completion decision model

## Acceptance Criteria (Technical)
*   [x] Completion cannot occur before successful connect state.
*   [x] Successful connect updates onboarding state to completed.
*   [x] Failed connect keeps onboarding in non-complete state.
*   [x] Completion gate integrates with toggle/routing lifecycle.

## Business Rules & Logic
*   Completion must represent real product readiness, not just walkthrough completion.

## Dependencies
*   Depends on: KP-059, KP-091

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
