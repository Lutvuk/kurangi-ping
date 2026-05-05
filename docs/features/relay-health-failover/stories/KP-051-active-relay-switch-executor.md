# KP-051: Active Relay Switch Executor

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Execute controlled active relay switch sequence when failover decision is approved.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/failover_executor.rs`
*   **Functions/Classes:** `switch_active_relay()`
*   **API Endpoints:** N/A
*   **Data Models:** Failover execution context model

## Acceptance Criteria (Technical)
*   [x] Switch sequence tears down old path safely before promoting new route.
*   [x] Failover switch respects retry budget and backoff.
*   [x] Partial switch failure returns deterministic fallback/terminal state.
*   [x] Execution logs produce non-sensitive diagnostics.

## Business Rules & Logic
*   Seamless continuity is prioritized, but bounded failure behavior is mandatory.

## Dependencies
*   Depends on: KP-050, KP-032

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
