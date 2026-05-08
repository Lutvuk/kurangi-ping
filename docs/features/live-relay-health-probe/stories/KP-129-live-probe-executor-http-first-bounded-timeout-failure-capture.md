# KP-129: Live Probe Executor (HTTP first, bounded timeout, failure capture)

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement real relay probing path to measure live latency and capture failure/timeout results for each configured target.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health.go`, `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `probeRelayTarget()`, `probeAllRelays()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `relayProbeResult`

## Acceptance Criteria (Technical)
*   [x] Each configured relay target is probed using real network call path.
*   [x] Probe execution enforces bounded timeout per target.
*   [x] Failures/timeouts are captured as structured probe results.
*   [x] Unit tests validate latency capture and timeout/failure paths.

## Business Rules & Logic
*   Probe execution must avoid unbounded blocking and must complete deterministically.

## Dependencies
*   Depends on: KP-128

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
