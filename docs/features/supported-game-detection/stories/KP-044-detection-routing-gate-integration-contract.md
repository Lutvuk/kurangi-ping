# KP-044: Detection-Routing Gate Integration Contract

**Epic:** EPIC-DETECTION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Integrate detection state contract as prerequisite gate for routing activation requests.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/detection_gate.rs`
*   **Functions/Classes:** `can_activate_routing()`
*   **API Endpoints:** N/A
*   **Data Models:** Detection gate decision model

## Acceptance Criteria (Technical)
*   [x] Routing activation denied when detection state is not `detected`.
*   [x] Gate response includes reason code for UI message mapping.
*   [x] Stale/error states are handled deterministically.
*   [x] Integration tests cover route activation gate behavior.

## Business Rules & Logic
*   Routing should never start on unknown/invalid game context.

## Dependencies
*   Depends on: KP-041, KP-032

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
