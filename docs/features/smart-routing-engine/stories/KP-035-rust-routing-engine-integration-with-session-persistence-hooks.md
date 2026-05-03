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
*   [ ] Session start/end hooks persist consistent timestamps.
*   [ ] Failed sessions capture end reason code.
*   [ ] Persistence failures do not crash routing engine loop.
*   [ ] Integration path is test-covered with DB mock/fixture.

## Business Rules & Logic
*   Persistence is supportive; routing loop stability remains highest priority.

## Dependencies
*   Depends on: KP-015, KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
