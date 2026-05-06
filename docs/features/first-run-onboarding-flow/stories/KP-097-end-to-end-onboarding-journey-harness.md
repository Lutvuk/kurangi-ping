# KP-097: End-to-End Onboarding Journey Harness

**Epic:** EPIC-ONBOARDING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 4
**Priority:** Must

---

## Objective
Validate complete onboarding journey including success, interruption, and failure recovery branches.

## Technical Specifications
*   **Proposed Files:** `tests/onboarding/e2e_onboarding_journey.rs`
*   **Functions/Classes:** `run_onboarding_journey_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** Journey scenario fixtures

## Acceptance Criteria (Technical)
*   [x] Happy path completes and sets onboarding completed state.
*   [x] Interruption path resumes correctly.
*   [x] Failure branch paths remain recoverable.
*   [x] Test traces are deterministic and debuggable.

## Business Rules & Logic
*   Onboarding reliability is a core adoption gate.

## Dependencies
*   Depends on: KP-092, KP-096

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
