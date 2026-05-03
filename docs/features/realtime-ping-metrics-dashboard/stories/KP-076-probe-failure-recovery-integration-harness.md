# KP-076: Probe Failure Recovery Integration Harness

**Epic:** EPIC-METRICS
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Validate dashboard and backend recovery behavior under probe failure and resume scenarios.

## Technical Specifications
*   **Proposed Files:** `tests/metrics/probe_recovery_harness.rs`
*   **Functions/Classes:** `run_probe_recovery_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** Probe failure scenario fixtures

## Acceptance Criteria (Technical)
*   [ ] Harness simulates intermittent probe failure and recovery.
*   [ ] Degraded -> live recovery path is deterministic.
*   [ ] Last-known values persist during failure window.
*   [ ] Recovery trace output is reproducible.

## Business Rules & Logic
*   Measurement resilience must be proven before release.

## Dependencies
*   Depends on: KP-070, KP-074

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
