# KP-016: Go Relay Controller DB Binding Stub

**Epic:** EPIC-FOUNDATION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Create Go-side DB binding stub to prepare relay controller for future persistence needs.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/db/sqlite.go`, `apps/relay-controller/internal/db/migrate.go`
*   **Functions/Classes:** `OpenDB()`, `Migrate()`
*   **API Endpoints:** N/A
*   **Data Models:** relay/manifest related tables (read-oriented baseline)

## Acceptance Criteria (Technical)
*   [x] Controller can open SQLite connection for local/dev mode.
*   [x] Migration invocation path exists (even if limited scope).
*   [x] DB bootstrap errors are surfaced through service startup logs.
*   [x] No write-heavy business logic added in scaffold stage.

## Business Rules & Logic
*   Keep controller data binding minimal until feature-specific implementation.

## Dependencies
*   Depends on: KP-010, KP-011

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
