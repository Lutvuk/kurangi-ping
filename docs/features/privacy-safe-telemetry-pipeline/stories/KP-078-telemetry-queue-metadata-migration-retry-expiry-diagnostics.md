# KP-078: Telemetry Queue Metadata Migration (retry/expiry diagnostics)

**Epic:** EPIC-TELEMETRY
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Extend telemetry queue persistence metadata for retry and expiry diagnostics.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0005_telemetry_queue_metadata.sql`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** `telemetry_batch`, `telemetry_event`

## Acceptance Criteria (Technical)
*   [ ] Migration adds needed diagnostic columns without breaking existing reads.
*   [ ] Migration is idempotent and forward-compatible.
*   [ ] Default values preserve existing pipeline behavior.
*   [ ] Schema docs updated.

## Business Rules & Logic
*   Telemetry diagnostics should improve observability without collecting sensitive data.

## Dependencies
*   Depends on: KP-011, KP-012

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
