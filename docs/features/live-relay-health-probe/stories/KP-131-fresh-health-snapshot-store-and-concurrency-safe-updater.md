# KP-131: Fresh Health Snapshot Store and Concurrency-Safe Updater

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Should

---

## Objective
Provide in-memory latest probe snapshot storage with safe concurrent read/write behavior for health endpoint serving path.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health.go`, `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `relayHealthSnapshotStore`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `RelayHealthResponse`

## Acceptance Criteria (Technical)
*   [ ] Latest probe results can be stored and read atomically.
*   [ ] Concurrent endpoint reads do not race with snapshot updates.
*   [ ] Snapshot includes fresh `updated_at` timestamp values from latest probe cycle.
*   [ ] Tests cover repeated reads/updates without contract drift.

## Business Rules & Logic
*   Response freshness must come from latest successful probe cycle, not static literals.

## Dependencies
*   Depends on: KP-129, KP-130

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear

