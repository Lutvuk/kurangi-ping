# KP-059: ON Pipeline Gate Chain (Detection + Manifest + Route Precheck)

**Epic:** EPIC-TOGGLE
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement ON activation gate chain before routing starts.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/on_pipeline.rs`
*   **Functions/Classes:** `execute_on_pipeline()`
*   **API Endpoints:** N/A
*   **Data Models:** ON precheck result model

## Acceptance Criteria (Technical)
*   [ ] Detection gate required and enforced.
*   [ ] Manifest verification gate required and enforced.
*   [ ] Route prechecks run before activation attempt.
*   [ ] Failure returns normalized reason codes for UI.

## Business Rules & Logic
*   Activation must never bypass integrity/safety gates.

## Dependencies
*   Depends on: KP-044, KP-030, KP-058

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
