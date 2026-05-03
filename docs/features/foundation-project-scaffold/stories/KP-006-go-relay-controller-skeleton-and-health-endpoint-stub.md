# KP-006: Go Relay Controller Skeleton and Health Endpoint Stub

**Epic:** EPIC-FOUNDATION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Create Go relay controller service skeleton with basic HTTP server and stubbed relay health endpoint.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/go.mod`, `apps/relay-controller/cmd/server/main.go`, `apps/relay-controller/internal/http/health.go`
*   **Functions/Classes:** `main()`, `registerRoutes()`, `GetRelayHealth()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** Relay health response DTO stub

## Acceptance Criteria (Technical)
*   [ ] Go module initializes and builds.
*   [ ] HTTP server starts and exposes `/v1/relay/health` stub endpoint.
*   [ ] Response schema shape aligns with `docs/api/contracts/relay-public.v1.yaml` baseline.
*   [ ] Logging excludes sensitive request payload details.

## Business Rules & Logic
*   Health endpoint is read-only in scaffold phase.
*   Contract-first consistency must be maintained.

## Dependencies
*   Depends on: KP-001, KP-002

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing (basic handler tests)
*   [ ] Lint/Type check clear
