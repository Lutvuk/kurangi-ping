# KP-046: Detection Simulation and Anti-False-Positive Test Harness

**Epic:** EPIC-DETECTION
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Create simulation harness validating detection correctness and low false-positive rate.

## Technical Specifications
*   **Proposed Files:** `tests/detection/detection_harness.rs`, `tests/fixtures/process_lists/`
*   **Functions/Classes:** `run_detection_scenarios()`
*   **API Endpoints:** N/A
*   **Data Models:** Synthetic process fixture sets

## Acceptance Criteria (Technical)
*   [ ] Harness validates detection for supported process sets.
*   [ ] Harness validates rejection of unsupported/near-match process names.
*   [ ] Stale/error state scenarios are covered.
*   [ ] Results are deterministic across runs.

## Business Rules & Logic
*   False positive control is critical for trust and anti-cheat safety.

## Dependencies
*   Depends on: KP-044

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
