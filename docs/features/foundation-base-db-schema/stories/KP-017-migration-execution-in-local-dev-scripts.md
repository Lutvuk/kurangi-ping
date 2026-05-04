# KP-017: Migration Execution in Local Dev Scripts

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 2
**Priority:** Must

---

## Objective
Integrate DB migrate/seed execution into local development bootstrap scripts.

## Technical Specifications
*   **Proposed Files:** `scripts/dev.ps1`, `scripts/db-migrate.ps1`, `scripts/db-seed.ps1`
*   **Functions/Classes:** script orchestration entry points
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Local bootstrap includes migration execution.
*   [x] Optional seed step is available and documented.
*   [x] Script exits non-zero on migration failure.
*   [x] Logs clearly indicate applied migration versions.

## Business Rules & Logic
*   Local onboarding should be one-command friendly.
*   Migration failure visibility must be immediate.

## Dependencies
*   Depends on: KP-014, KP-015, KP-016

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
