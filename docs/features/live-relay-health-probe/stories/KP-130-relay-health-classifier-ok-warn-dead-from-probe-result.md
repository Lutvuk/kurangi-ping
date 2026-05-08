# KP-130: Relay Health Classifier (`ok/warn/dead`) from Probe Result

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Map live probe outcomes into relay health status classes compatible with client failover semantics.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health.go`, `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `classifyRelayStatus()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `RelayHealthItem`

## Acceptance Criteria (Technical)
*   [x] Successful low-latency probe maps to `ok`.
*   [x] High-latency probe maps to `warn`.
*   [x] Timeout/failed probe maps to `dead`.
*   [x] Threshold mapping is deterministic and tested.

## Business Rules & Logic
*   Status semantics must stay aligned with relay-failover behavior in Rust engine.

## Dependencies
*   Depends on: KP-129, KP-049

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
