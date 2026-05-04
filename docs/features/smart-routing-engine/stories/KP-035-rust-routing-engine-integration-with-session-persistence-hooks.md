# KP-035: Rust Routing Engine Integration with Session Persistence Hooks

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Should

---

## Objective
Integrate orchestrator/state machine with DB session hooks for route session lifecycle tracking.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/integration.rs`, `crates/client-engine/src/db/route_session_repo.rs`
*   **Functions/Classes:** `start_route_session()`, `close_route_session()`
*   **API Endpoints:** N/A
*   **Data Models:** `route_session` mapping model

## Acceptance Criteria (Technical)
*   [x] Session start/end hooks persist consistent timestamps.
*   [x] Failed sessions capture end reason code.
*   [x] Persistence failures do not crash routing engine loop.
*   [x] Integration path is test-covered with DB mock/fixture.

## Business Rules & Logic
*   Persistence is supportive; routing loop stability remains highest priority.

## Dependencies
*   Depends on: KP-015, KP-034

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
