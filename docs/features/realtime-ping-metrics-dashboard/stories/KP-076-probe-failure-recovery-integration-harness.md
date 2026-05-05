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
*   [x] Harness simulates intermittent probe failure and recovery.
*   [x] Degraded -> live recovery path is deterministic.
*   [x] Last-known values persist during failure window.
*   [x] Recovery trace output is reproducible.

## Business Rules & Logic
*   Measurement resilience must be proven before release.

## Dependencies
*   Depends on: KP-070, KP-074

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
