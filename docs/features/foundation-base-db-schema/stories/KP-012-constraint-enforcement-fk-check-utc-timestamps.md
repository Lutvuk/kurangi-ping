# KP-012: Constraint Enforcement (FK/CHECK/UTC timestamps)

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Enforce key integrity and domain constraints at DB layer including enum/range checks and UTC timestamp policy.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0002_constraints.sql`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** core tables with FK/CHECK constraints

## Acceptance Criteria (Technical)
*   [x] `PRAGMA foreign_keys=ON` is enforced in DB initialization path.
*   [x] Enum-like fields have CHECK constraints.
*   [x] Numeric ranges (`packet_loss_pct`, retries) are constrained.
*   [x] Invalid inserts are rejected with deterministic DB errors.

## Business Rules & Logic
*   Constraint logic is a security and data-quality boundary.
*   UTC timestamp policy is mandatory for all persisted event/session times.

## Dependencies
*   Depends on: KP-011

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
