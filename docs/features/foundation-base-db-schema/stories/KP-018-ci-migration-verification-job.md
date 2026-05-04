# KP-018: CI Migration Verification Job

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** DevOps
**Estimation:** 2
**Priority:** Must

---

## Objective
Add CI job to verify DB migrations apply cleanly on fresh database and remain forward-compatible.

## Technical Specifications
*   **Proposed Files:** `.github/workflows/ci.yml` (or split workflow), `scripts/ci-verify-db.ps1`
*   **Functions/Classes:** CI job definitions
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] CI provisions clean SQLite DB and applies all migrations.
*   [x] CI fails when migration ordering or SQL syntax is invalid.
*   [x] Seed verification (optional stage) confirms idempotency.
*   [x] CI log includes migration summary artifacts.

## Business Rules & Logic
*   DB integrity must be enforced as merge gate.
*   Migration regressions are blockers.

## Dependencies
*   Depends on: KP-017

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
