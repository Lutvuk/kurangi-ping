# KP-011: Core Tables Creation Migration (v1_init)

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 5
**Priority:** Must

---

## Objective
Create initial migration that defines core tables from `docs/erd/core-erd.md`.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0001_v1_init.sql`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** `user_settings`, `supported_games`, `relay_manifest`, `relay_node`, `route_session`, `ping_sample`, `telemetry_batch`, `telemetry_event`

## Acceptance Criteria (Technical)
*   [ ] All core tables are created on clean database.
*   [ ] Primary keys follow ERD contract.
*   [ ] Foreign keys reference correct parent tables.
*   [ ] Migration runs successfully via framework from KP-010.

## Business Rules & Logic
*   Table names and relationships must remain traceable to ERD.
*   No feature-specific extra tables in foundation migration.

## Dependencies
*   Depends on: KP-010

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
