# KP-128: Relay Target Config Loader and Validation in Go Relay Controller

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement config loader for relay probe targets and timeout values from environment with validation and deterministic fallback behavior.

## Technical Specifications
*   **Proposed Files:** `apps/relay-controller/internal/http/health.go`, `apps/relay-controller/internal/http/health_test.go`
*   **Functions/Classes:** `loadRelayProbeConfig()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** `relayProbeConfig`

## Acceptance Criteria (Technical)
*   [x] Config loader parses relay target list from environment.
*   [x] Invalid or empty target input is handled with explicit fallback behavior.
*   [x] Probe timeout value is bounded and validated.
*   [x] Unit tests cover valid, invalid, and default config paths.

## Business Rules & Logic
*   Config validation must prevent malformed target data from crashing the health endpoint.

## Dependencies
*   Depends on: KP-127, KP-006

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
