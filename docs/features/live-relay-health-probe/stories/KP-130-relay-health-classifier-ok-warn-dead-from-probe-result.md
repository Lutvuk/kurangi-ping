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
*   [ ] Successful low-latency probe maps to `ok`.
*   [ ] High-latency probe maps to `warn`.
*   [ ] Timeout/failed probe maps to `dead`.
*   [ ] Threshold mapping is deterministic and tested.

## Business Rules & Logic
*   Status semantics must stay aligned with relay-failover behavior in Rust engine.

## Dependencies
*   Depends on: KP-129, KP-049

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear

