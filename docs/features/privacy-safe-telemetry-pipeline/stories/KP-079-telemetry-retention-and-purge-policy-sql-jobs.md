# KP-079: Telemetry Retention and Purge Policy SQL Jobs

**Epic:** EPIC-TELEMETRY
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Implement retention cleanup SQL paths for expired telemetry batches/events.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0006_telemetry_retention.sql`, `scripts/db-telemetry-prune.ps1`
*   **Functions/Classes:** `Invoke-TelemetryPrune`
*   **API Endpoints:** N/A
*   **Data Models:** `telemetry_batch`, `telemetry_event`

## Acceptance Criteria (Technical)
*   [ ] Expired telemetry rows are pruned by policy.
*   [ ] Active/retryable rows are preserved.
*   [ ] Purge operation is safe to re-run.
*   [ ] Purge summary metrics are logged.

## Business Rules & Logic
*   Storage growth must stay bounded over time.

## Dependencies
*   Depends on: KP-078

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
