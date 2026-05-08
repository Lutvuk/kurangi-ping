# KP-132: `GetRelayHealth` Dynamic Wiring with Backward-Compatible JSON Contract

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Replace hardcoded relay payload in `GetRelayHealth` with dynamic probe snapshot while preserving existing JSON response structure.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health.go`, `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `GetRelayHealth()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `RelayHealthResponse`, `RelayHealthItem`, `PageInfo`

## Acceptance Criteria (Technical)
*   [ ] `GetRelayHealth` returns `data[]` built from latest live probe snapshot.
*   [ ] `page.next_cursor` and `page.limit` behavior remains compatible with existing contract.
*   [ ] Field names and JSON types remain unchanged from current Rust-consumed schema.
*   [ ] Endpoint no longer returns hardcoded relay health values.

## Business Rules & Logic
*   Contract compatibility is mandatory to avoid breaking Rust client parsing and failover flow.

## Dependencies
*   Depends on: KP-131, KP-128

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear

