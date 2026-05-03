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
*   [ ] Degraded states do not trigger immediate oscillation.
*   [ ] Trigger conditions are deterministic and explainable.
*   [ ] Grace window is configurable.
*   [ ] Edge cases (rapid fluctuation) are test-covered.

## Business Rules & Logic
*   Failover reliability must not create instability loops.

## Dependencies
*   Depends on: KP-049, KP-033

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
