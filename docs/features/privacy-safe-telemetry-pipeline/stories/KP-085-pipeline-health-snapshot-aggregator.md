# KP-085: Pipeline Health Snapshot Aggregator

**Epic:** EPIC-TELEMETRY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Should

---

## Objective
Aggregate telemetry pipeline health stats for diagnostics and reliability monitoring.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/health_snapshot.rs`
*   **Functions/Classes:** `build_telemetry_health_snapshot()`
*   **API Endpoints:** N/A
*   **Data Models:** Pipeline health snapshot model

## Acceptance Criteria (Technical)
*   [ ] Snapshot includes queue depth, retry counts, drop counts, last delivery status.
*   [ ] Snapshot can be queried without blocking main event flow.
*   [ ] Values align with queue/delivery state machines.
*   [ ] Snapshot output excludes sensitive payload details.

## Business Rules & Logic
*   Operators need quick signal on telemetry subsystem health.

## Dependencies
*   Depends on: KP-084

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
