# KP-042: Manual Rescan Command Handler

**Epic:** EPIC-DETECTION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Provide explicit rescan command path that triggers immediate detection refresh.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/detection/commands.rs`
*   **Functions/Classes:** `trigger_rescan()`
*   **API Endpoints:** N/A
*   **Data Models:** Rescan command/result model

## Acceptance Criteria (Technical)
*   [ ] Rescan bypasses periodic interval wait.
*   [ ] Command returns updated normalized state.
*   [ ] Concurrent rescans are debounced/serialized.
*   [ ] Failures surface actionable error codes.

## Business Rules & Logic
*   Manual control should feel immediate and reliable.

## Dependencies
*   Depends on: KP-041

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
