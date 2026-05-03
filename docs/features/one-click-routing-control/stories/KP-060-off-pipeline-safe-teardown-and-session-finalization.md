# KP-060: OFF Pipeline Safe Teardown and Session Finalization

**Epic:** EPIC-TOGGLE
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement OFF flow that safely tears down route and finalizes session state.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/off_pipeline.rs`
*   **Functions/Classes:** `execute_off_pipeline()`
*   **API Endpoints:** N/A
*   **Data Models:** OFF result model

## Acceptance Criteria (Technical)
*   [ ] Active route teardown always attempted.
*   [ ] Session close hook is invoked with end reason.
*   [ ] Partial teardown failures move lifecycle to deterministic error state.
*   [ ] OFF from non-active state is safely idempotent.

## Business Rules & Logic
*   OFF must be safe and trusted as emergency stop.

## Dependencies
*   Depends on: KP-058, KP-035

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
