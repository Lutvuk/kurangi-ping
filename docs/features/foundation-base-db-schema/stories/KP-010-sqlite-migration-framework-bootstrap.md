# KP-010: SQLite Migration Framework Bootstrap

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Establish a deterministic SQLite migration framework and folder conventions for schema evolution.

## Technical Specifications
*   **Proposed Files:** `db/migrations/`, `db/migrations/README.md`, `scripts/db-migrate.ps1`
*   **Functions/Classes:** `Invoke-DbMigrate` (script entry)
*   **API Endpoints:** N/A
*   **Data Models:** `schema_migrations` metadata table

## Acceptance Criteria (Technical)
*   [x] Migration directory conventions are documented.
*   [x] Migration runner applies pending SQL files in lexical order.
*   [x] `schema_migrations` table tracks applied versions.
*   [x] Re-running migrations is idempotent.

## Business Rules & Logic
*   All schema changes must be migration-driven.
*   No manual out-of-band DB edits in baseline flow.

## Dependencies
*   Depends on: None

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
