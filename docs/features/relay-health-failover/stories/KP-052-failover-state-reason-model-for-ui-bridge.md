# KP-052: Failover State/Reason Model for UI Bridge

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Should

---

## Objective
Expose normalized failover state and reason metadata for frontend consumption.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/failover_state.rs`
*   **Functions/Classes:** `build_failover_state_payload()`
*   **API Endpoints:** N/A
*   **Data Models:** Failover UI state payload model

## Acceptance Criteria (Technical)
*   [ ] Payload includes current state, previous relay, next relay, reason code.
*   [ ] Unknown reasons map to safe default explanation code.
*   [ ] Payload is serializable and versionable.
*   [ ] Mapping consistency is unit-tested.

## Business Rules & Logic
*   User messaging should be clear and non-technical by default.

## Dependencies
*   Depends on: KP-051, KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
