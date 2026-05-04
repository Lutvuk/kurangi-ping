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
*   [x] Rescan bypasses periodic interval wait.
*   [x] Command returns updated normalized state.
*   [x] Concurrent rescans are debounced/serialized.
*   [x] Failures surface actionable error codes.

## Business Rules & Logic
*   Manual control should feel immediate and reliable.

## Dependencies
*   Depends on: KP-041

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
