# KP-015: Rust DB Access Layer Skeleton (sqlite + migrations)

**Epic:** EPIC-FOUNDATION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Provide Rust-side DB initialization and migration execution hooks for client engine startup.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/db/mod.rs`, `crates/client-engine/src/db/migrate.rs`
*   **Functions/Classes:** `init_db()`, `run_migrations()`
*   **API Endpoints:** N/A
*   **Data Models:** core SQLite tables

## Acceptance Criteria (Technical)
*   [ ] Rust engine initializes SQLite connection with FK enforcement.
*   [ ] Migration runner can be invoked from engine startup path.
*   [ ] Initialization fails fast with clear errors on migration failure.
*   [ ] Module boundaries remain aligned with architecture docs.

## Business Rules & Logic
*   DB boot must be deterministic and safe for repeated startup.
*   No business feature logic beyond DB lifecycle.

## Dependencies
*   Depends on: KP-010, KP-011, KP-012

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
