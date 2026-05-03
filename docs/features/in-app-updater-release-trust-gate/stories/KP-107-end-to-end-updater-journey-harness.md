# KP-107: End-to-End Updater Journey Harness

**Epic:** EPIC-UPDATER
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Simulate full updater user journey from check to apply/restart across success and failure branches.

## Technical Specifications
*   **Proposed Files:** `tests/updater/e2e_updater_journey.rs`
*   **Functions/Classes:** `run_updater_journey_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** Updater journey fixtures

## Acceptance Criteria (Technical)
*   [ ] Success path from check to ready-to-restart validates.
*   [ ] Failure paths produce recoverable UI states.
*   [ ] Deferred restart path preserves update readiness state.
*   [ ] Journey traces are deterministic.

## Business Rules & Logic
*   Update UX should be robust before public launch.

## Dependencies
*   Depends on: KP-103, KP-104, KP-106

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
