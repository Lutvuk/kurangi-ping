# KP-133: Relay Health API Contract Regression Suite (shape unchanged, data dynamic)

**Epic:** EPIC-RELAY
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 2
**Priority:** Must

---

## Objective
Add integration tests to lock response contract shape while validating dynamic data source behavior from live probes.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `TestGetRelayHealthContractRegression`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `RelayHealthResponse`

## Acceptance Criteria (Technical)
*   [ ] Tests assert response envelope fields remain unchanged (`data`, `page`).
*   [ ] Tests assert item schema compatibility (`relay_id`, `status`, `latency_ms`, `updated_at`).
*   [ ] Tests assert data values originate from dynamic probe path, not fixed literals.
*   [ ] Regression suite fails if contract shape drifts.

## Business Rules & Logic
*   JSON compatibility is release-blocking because Rust engine depends on this shape.

## Dependencies
*   Depends on: KP-132

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear

