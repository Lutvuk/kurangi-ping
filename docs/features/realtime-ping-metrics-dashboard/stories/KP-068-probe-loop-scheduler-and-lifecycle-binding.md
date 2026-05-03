# KP-068: Probe Loop Scheduler and Lifecycle Binding

**Epic:** EPIC-METRICS
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement measurement loop scheduler bound to routing lifecycle start/stop hooks.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/metrics/probe_scheduler.rs`
*   **Functions/Classes:** `start_probe_loop()`, `stop_probe_loop()`
*   **API Endpoints:** N/A
*   **Data Models:** Probe scheduler context

## Acceptance Criteria (Technical)
*   [ ] Probe loop starts only when routing lifecycle is active.
*   [ ] Probe loop stops cleanly on OFF/disconnect.
*   [ ] Interval is configurable and bounded.
*   [ ] Concurrent loop duplication is prevented.

## Business Rules & Logic
*   Measurement cadence should be responsive without resource spikes.

## Dependencies
*   Depends on: KP-060, KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
