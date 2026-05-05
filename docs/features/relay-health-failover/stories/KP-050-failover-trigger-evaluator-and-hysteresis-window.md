# KP-050: Failover Trigger Evaluator and Hysteresis Window

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Decide when failover is triggered using health transitions with grace window/hysteresis anti-flapping logic.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/failover_trigger.rs`
*   **Functions/Classes:** `should_failover()`, `apply_hysteresis_window()`
*   **API Endpoints:** N/A
*   **Data Models:** Failover decision model

## Acceptance Criteria (Technical)
*   [x] Degraded states do not trigger immediate oscillation.
*   [x] Trigger conditions are deterministic and explainable.
*   [x] Grace window is configurable.
*   [x] Edge cases (rapid fluctuation) are test-covered.

## Business Rules & Logic
*   Failover reliability must not create instability loops.

## Dependencies
*   Depends on: KP-049, KP-033

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
