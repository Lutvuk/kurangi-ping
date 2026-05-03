# KP-013: Baseline Indexing for Sessions and Telemetry

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Add baseline indexes for expected read/write paths in session and telemetry flows.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0003_indexes.sql`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** `route_session`, `ping_sample`, `telemetry_batch`, `telemetry_event`

## Acceptance Criteria (Technical)
*   [ ] Indexes exist for session lookup/filter fields.
*   [ ] Indexes exist for telemetry queue processing fields.
*   [ ] Query plan checks confirm indexed access on target paths.
*   [ ] No redundant or contradictory indexes are introduced.

## Business Rules & Logic
*   Optimize for reliability and predictable local performance.
*   Keep index set minimal and purposeful.

## Dependencies
*   Depends on: KP-012

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
