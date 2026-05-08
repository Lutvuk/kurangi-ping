# KP-134: Offline Relay Scenario Harness (timeout/failure -> `dead`)

**Epic:** EPIC-RELAY
**Layer:** L5-integration
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Validate end-to-end behavior for unreachable relay targets so timeout/failure consistently maps to `dead` status in API response.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `TestOfflineRelayProbeMapsToDeadStatus`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `RelayHealthItem`

## Acceptance Criteria (Technical)
*   [ ] Timeout probe path marks relay as `dead`.
*   [ ] Immediate network failure path marks relay as `dead`.
*   [ ] Offline target handling does not crash or block full response generation.
*   [ ] Harness validates deterministic output across repeated runs.

## Business Rules & Logic
*   Offline detection must be explicit to support failover decision quality in client engine.

## Dependencies
*   Depends on: KP-132, KP-133

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear

